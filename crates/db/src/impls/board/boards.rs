use crate::aggregates::structs::BoardAggregates;
use crate::models::board::board_mods::BoardModerator;
use crate::newtypes::DbUrl;
use crate::schema::{board_mods, board_person_bans, boards, instance};
use crate::utils::functions::{hot_rank, lower};
use crate::utils::{fuzzy_search, limit_and_offset};
use crate::{
    models::board::board_person_bans::{BoardPersonBan, BoardPersonBanForm},
    models::board::boards::{Board, BoardForm},
    traits::{ApubActor, Bannable, Crud},
    utils::{get_conn, naive_now, DbPool},
};
use crate::{ListingType, SortType};
use diesel::{dsl::*, prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

pub enum CollectionType {
    Moderators,
    Featured,
}

impl Board {
    /// Check if a board with a name exists
    pub async fn board_exists(pool: &DbPool, board_name: &str) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        let c = boards
            .filter(
                name.ilike(
                    board_name
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
            .optional();

        c.map(|b| b.is_some())
    }

    /// Takes a board id and an user id, and returns true if the user mods the board with the given id or is an admin
    pub async fn board_has_mod(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        let mod_id = board_mods::table
            .select(board_mods::id)
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::person_id.eq(person_id))
            .filter(board_mods::invite_accepted.eq(true))
            .first::<i32>(conn)
            .await
            .optional();

        mod_id.map(|opt| opt.is_some())
    }

    /// Takes a board id and an user id, and returns a mod relationship if one exists
    pub async fn board_get_mod(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<Option<BoardModerator>, Error> {
        let conn = &mut get_conn(pool).await?;
        board_mods::table
            .select(board_mods::all_columns)
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::person_id.eq(person_id))
            .filter(board_mods::invite_accepted.eq(true))
            .first::<BoardModerator>(conn)
            .await
            .optional()

        //mod_id.map(|opt| opt.is_some())
    }

    /// Takes a board id and an user id, and returns a mod invite if one exists
    pub async fn get_mod_invite(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<Option<BoardModerator>, Error> {
        let conn = &mut get_conn(pool).await?;
        board_mods::table
            .select(board_mods::all_columns)
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::person_id.eq(person_id))
            .filter(board_mods::invite_accepted.eq(false))
            .first::<BoardModerator>(conn)
            .await
            .optional()

        //mod_id.map(|opt| opt.is_some())
    }

    pub async fn get_by_collection_url(
        pool: &DbPool,
        url: &DbUrl,
    ) -> Result<(Board, CollectionType), Error> {
        use crate::schema::boards::dsl::{featured_url, moderators_url};
        use CollectionType::*;
        let conn = &mut get_conn(pool).await?;
        let res = boards::table
            .filter(moderators_url.eq(url))
            .first::<Self>(conn)
            .await;
        if let Ok(b) = res {
            return Ok((b, Moderators));
        }
        let res = boards::table
            .filter(featured_url.eq(url))
            .first::<Self>(conn)
            .await;
        if let Ok(b) = res {
            return Ok((b, Featured));
        }
        Err(diesel::NotFound)
    }

    pub async fn get_by_name(pool: &DbPool, board_name: &str) -> Result<Self, Error> {
        use crate::schema::boards::dsl::name;
        let conn = &mut get_conn(pool).await?;
        boards::table
            .filter(
                name.ilike(
                    board_name
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
    }

    /// Takes a board id and an user id, and returns true if the user is banned from the board with the given id
    pub async fn board_has_ban(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        let ban_id = board_person_bans::table
            .select(board_person_bans::id)
            .filter(board_person_bans::board_id.eq(board_id))
            .filter(board_person_bans::person_id.eq(person_id))
            .filter(
                board_person_bans::expires
                    .is_null()
                    .or(board_person_bans::expires.gt(now)),
            )
            .first::<i32>(conn)
            .await
            .optional();

        ban_id.map(|opt| opt.is_some())
    }

    pub async fn ban(&self, pool: &DbPool, reason: Option<&String>) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;

        diesel::update(boards.find(self.id))
            .set((is_removed.eq(true), ban_reason.eq(reason)))
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn unban(&self, pool: &DbPool) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;

        diesel::update(boards.find(self.id))
            .set(is_removed.eq(false))
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn update_deleted(
        pool: &DbPool,
        board_id: i32,
        new_is_deleted: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_deleted.eq(new_is_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_removed(
        pool: &DbPool,
        board_id: i32,
        new_is_removed: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_removed.eq(new_is_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_hidden(
        pool: &DbPool,
        board_id: i32,
        new_is_hidden: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_hidden.eq(new_is_hidden), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_banned(
        pool: &DbPool,
        board_id: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_removed.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn list_with_counts(
        pool: &DbPool,
        person_id_join: i32,
        limit: Option<i64>,
        page: Option<i64>,
        sort: SortType,
        listing_type: ListingType,
        search: Option<String>,
        search_title_and_desc: bool,
        banned_boards: bool,
    ) -> Result<Vec<(Self, BoardAggregates)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{board_aggregates, board_mods, board_subscriber, boards};

        let mut query = boards::table
            .inner_join(board_aggregates::table)
            .left_join(
                board_subscriber::table.on(board_subscriber::board_id
                    .eq(boards::id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                board_mods::table.on(board_mods::board_id
                    .eq(boards::id)
                    .and(board_mods::person_id.eq(person_id_join))),
            )
            .filter(boards::is_removed.eq(banned_boards))
            .select((boards::all_columns, board_aggregates::all_columns))
            .into_boxed();

        if let Some(search_query) = search {
            query = if search_title_and_desc {
                query.filter(
                    boards::name
                        .ilike(fuzzy_search(search_query.as_str()))
                        .or(boards::title.ilike(fuzzy_search(search_query.as_str())))
                        .or(boards::description.ilike(fuzzy_search(search_query.as_str()))),
                )
            } else {
                query.filter(boards::name.ilike(fuzzy_search(search_query.as_str())))
            }
        };

        query = match listing_type {
            // All except hidden boards
            ListingType::All => query.filter(
                boards::is_hidden
                    .eq(false)
                    .or(board_subscriber::id.is_not_null()),
            ),
            // Subscribed boards
            ListingType::Subscribed => query.filter(board_subscriber::id.is_not_null()),
            // Local: local boards only \ hidden boards
            ListingType::Local => query.filter(boards::local.eq(true)).filter(
                boards::is_hidden
                    .eq(false)
                    .or(board_subscriber::id.is_not_null()),
            ),
            // Mod feed: only boards that the user moderates
            ListingType::Moderated => query.filter(board_mods::id.is_not_null()),
        };

        query = match sort {
            SortType::New => query.order_by(boards::creation_date.desc()),
            SortType::TopAll => query.order_by(board_aggregates::subscribers.desc()),
            SortType::Hot => query
                .order_by(
                    hot_rank(
                        board_aggregates::subscribers,
                        board_aggregates::creation_date,
                    )
                    .desc(),
                )
                .then_order_by(board_aggregates::creation_date.desc()),
            SortType::Old => query.order_by(boards::creation_date.asc()),
            _ => query
                .order_by(
                    hot_rank(
                        board_aggregates::subscribers,
                        board_aggregates::creation_date,
                    )
                    .desc(),
                )
                .then_order_by(board_aggregates::creation_date.desc()),
        };

        let (limit, offset) = limit_and_offset(page, limit)?;

        query = query.limit(limit).offset(offset);

        query.load::<(Self, BoardAggregates)>(conn).await
    }
}

pub mod safe_type {
    use crate::{models::board::boards::BoardSafe, schema::boards::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        icon,
        banner,
        description,
        creation_date,
        updated,
        is_deleted,
        is_removed,
        is_nsfw,
        is_hidden,
        actor_id,
        subscribers_url,
        inbox_url,
        shared_inbox_url,
        moderators_url,
        featured_url,
        ban_reason,
        primary_color,
        secondary_color,
        hover_color,
        sidebar,
        sidebar_html,
    );

    impl ToSafe for BoardSafe {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                icon,
                banner,
                description,
                creation_date,
                updated,
                is_deleted,
                is_removed,
                is_nsfw,
                is_hidden,
                actor_id,
                subscribers_url,
                inbox_url,
                shared_inbox_url,
                moderators_url,
                featured_url,
                ban_reason,
                primary_color,
                secondary_color,
                hover_color,
                sidebar,
                sidebar_html,
            )
        }
    }
}

#[async_trait::async_trait]
impl Crud for Board {
    type Form = BoardForm;
    type IdType = i32;

    async fn read(pool: &DbPool, board_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        boards::table.find(board_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, board_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(boards::table.find(board_id))
            .execute(conn)
            .await
    }
    async fn create(pool: &DbPool, form: &BoardForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_board = diesel::insert_into(boards::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_board)
    }
    async fn update(pool: &DbPool, board_id: i32, form: &BoardForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(boards::table.find(board_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Bannable for BoardPersonBan {
    type Form = BoardPersonBanForm;

    async fn ban(pool: &DbPool, ban_form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_person_bans::dsl::{board_id, board_person_bans, person_id};
        insert_into(board_person_bans)
            .values(ban_form)
            .on_conflict((board_id, person_id))
            .do_update()
            .set(ban_form)
            .get_result::<Self>(conn)
            .await
    }

    async fn unban(pool: &DbPool, ban_form: &Self::Form) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_person_bans::dsl::{board_id, board_person_bans, person_id};
        diesel::delete(
            board_person_bans
                .filter(board_id.eq(ban_form.board_id))
                .filter(person_id.eq(ban_form.person_id)),
        )
        .execute(conn)
        .await
    }
}

#[async_trait::async_trait]
impl ApubActor for Board {
    async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        Ok(boards::table
            .filter(boards::actor_id.eq(object_id.to_string()))
            .first::<Board>(conn)
            .await
            .ok()
            .map(Into::into))
    }

    async fn read_from_name(
        pool: &DbPool,
        board_name: &str,
        include_deleted: bool,
    ) -> Result<Board, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut q = boards::table
            .into_boxed()
            .filter(boards::local.eq(true))
            .filter(lower(boards::name).eq(board_name.to_lowercase()));
        if !include_deleted {
            q = q.filter(boards::is_deleted.eq(false));
        }
        q.first::<Self>(conn).await
    }

    async fn read_from_name_and_domain(
        pool: &DbPool,
        board_name: &str,
        for_domain: &str,
    ) -> Result<Board, Error> {
        let conn = &mut get_conn(pool).await?;
        boards::table
            .inner_join(instance::table)
            .filter(lower(boards::name).eq(board_name.to_lowercase()))
            .filter(instance::domain.eq(for_domain))
            .select(boards::all_columns)
            .first::<Self>(conn)
            .await
    }
}
