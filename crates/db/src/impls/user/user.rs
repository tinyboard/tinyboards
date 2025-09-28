use crate::{
    models::user::{User, UserForm},
    newtypes::DbUrl,
    schema::users,
    traits::Crud,
    utils::{fuzzy_search, get_conn, limit_and_offset, naive_now, DbPool},
    UserListingType, UserSortType,
};

use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

impl User {
    pub async fn get_by_name(pool: &DbPool, username: String) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        
        users::table
            .filter(
                users::name.ilike(
                    username
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
    }

    pub async fn get_by_id(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        
        users::table
            .filter(users::id.eq(id))
            .first::<Self>(conn)
            .await
    }

    pub async fn get_by_email(pool: &DbPool, email: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        
        users::table
            .filter(users::email.eq(email))
            .first::<Self>(conn)
            .await
    }

    pub async fn search_by_name(pool: &DbPool, query: &str) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        
        users::table
            .filter(users::name.ilike(fuzzy_search(query)))
            .load(conn)
            .await
    }

    pub async fn get_users_for_ids(pool: &DbPool, ids: Vec<i32>) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;

        users::table
            .filter(users::id.eq_any(ids))
            .load::<Self>(conn)
            .await
    }

    pub async fn get_users_with_counts_for_ids(pool: &DbPool, ids: Vec<i32>) -> Result<Vec<(Self, crate::aggregates::structs::UserAggregates)>, Error> {
        use crate::schema::{users, user_aggregates};
        let conn = &mut get_conn(pool).await?;

        users::table
            .inner_join(user_aggregates::table.on(users::id.eq(user_aggregates::user_id)))
            .filter(users::id.eq_any(ids))
            .select((users::all_columns, user_aggregates::all_columns))
            .load::<(User, crate::aggregates::structs::UserAggregates)>(conn)
            .await
    }

    pub async fn update_settings(
        pool: &DbPool,
        id_: i32,
        form: &UserForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(users::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_ban(
        pool: &DbPool,
        id_: i32,
        new_banned: bool,
        expires: Option<NaiveDateTime>,
    ) -> Result<Self, Error> {
        use crate::schema::users::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(users.find(id_))
            .set((
                is_banned.eq(new_banned),
                unban_date.eq(expires),
                updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_admin(pool: &DbPool, id_: i32, admin_level_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::users::dsl::*;
        diesel::update(users.find(id_))
            .set((
                is_admin.eq(admin_level_ > 0),
                admin_level.eq(admin_level_),
                updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn delete_account(pool: &DbPool, user_id: i32) -> Result<User, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(users::table.find(user_id))
            .set((
                users::display_name.eq::<Option<String>>(None),
                users::email.eq::<Option<String>>(None),
                users::avatar.eq::<Option<String>>(None),
                users::banner.eq::<Option<String>>(None),
                users::bio.eq::<Option<String>>(None),
                users::is_deleted.eq(true),
                users::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_default_avatar(
        pool: &DbPool,
        old_default_url: String,
        new_default_url: String,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::users::dsl::*;

        diesel::update(users)
            .filter(avatar.eq(old_default_url))
            .set((avatar.eq(new_default_url), updated.eq(naive_now())))
            .execute(conn)
            .await
    }

    pub async fn ban(
        &self,
        pool: &DbPool,
        unban_date_: Option<NaiveDateTime>,
    ) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::users::dsl::*;

        let is_permanent_ban = unban_date_.is_none();

        if is_permanent_ban {
            // clear profile
            diesel::update(users.find(self.id))
                .set((
                    is_banned.eq(true),
                    unban_date.eq(None::<NaiveDateTime>),
                    avatar.eq(None::<DbUrl>),
                    banner.eq(None::<DbUrl>),
                    profile_background.eq(None::<DbUrl>),
                    display_name.eq(None::<String>),
                    bio.eq(None::<String>),
                    bio_html.eq(None::<String>),
                    profile_music.eq(None::<DbUrl>),
                    profile_music_youtube.eq(None::<String>),
                    updated.eq(naive_now()),
                ))
                .execute(conn)
        } else {
            diesel::update(users.find(self.id))
                .set((
                    is_banned.eq(true),
                    unban_date.eq(unban_date_),
                    updated.eq(naive_now()),
                ))
                .execute(conn)
        }
        .await
        .map(|_| ())
    }

    pub async fn unban(&self, pool: &DbPool) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::users::dsl::*;

        diesel::update(users.find(self.id))
            .set((
                is_banned.eq(false),
                unban_date.eq(None::<NaiveDateTime>),
                updated.eq(naive_now()),
            ))
            .execute(conn)
            .await
            .map(|_| ())
    }

    /// Load a bunch of users with their aggregates/counts.
    pub async fn list_with_counts(
        pool: &DbPool,
        sort: UserSortType,
        limit: Option<i64>,
        page: Option<i64>,
        listing_type: UserListingType,
        search_term: Option<String>,
    ) -> Result<Vec<(Self, crate::aggregates::structs::UserAggregates)>, Error> {
        use crate::schema::{users, user_aggregates};
        let conn = &mut get_conn(pool).await?;

        let mut query = users::table
            .inner_join(user_aggregates::table.on(users::id.eq(user_aggregates::user_id)))
            .select((users::all_columns, user_aggregates::all_columns))
            .into_boxed();

        query = match sort {
            UserSortType::New => query.then_order_by(users::creation_date.desc()),
            UserSortType::Old => query.then_order_by(users::creation_date.asc()),
            UserSortType::MostRep => query.then_order_by((user_aggregates::post_score + user_aggregates::comment_score).desc()),
            UserSortType::MostPosts => query.then_order_by(user_aggregates::post_count.desc()),
            UserSortType::MostComments => query.then_order_by(user_aggregates::comment_count.desc()),
        };

        if let Some(ref search) = search_term {
            query = query.filter(
                users::name
                    .ilike(fuzzy_search(search))
                    .or(users::display_name.ilike(fuzzy_search(search))),
            );
        }

        query = match listing_type {
            UserListingType::All => query,
            UserListingType::NotBanned => query.filter(users::is_banned.eq(false)),
            UserListingType::Banned => query.filter(users::is_banned.eq(true)),
            UserListingType::Admins => query.filter(users::is_admin.eq(true)),
        };

        let (limit, offset) = limit_and_offset(page, limit)?;
        query = query.limit(limit).offset(offset);

        query.load::<(User, crate::aggregates::structs::UserAggregates)>(conn).await
    }
}

#[async_trait::async_trait]
impl Crud for User {
    type Form = UserForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        users::table.find(id_).first::<Self>(conn).await
    }
    
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(users::table.find(id_)).execute(conn).await
    }
    
    async fn create(pool: &DbPool, form: &UserForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_user = diesel::insert_into(users::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_user)
    }
    
    async fn update(pool: &DbPool, id_: i32, form: &UserForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(users::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

pub mod safe_type {
    use crate::{models::user::*, schema::users::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        display_name,
        is_banned,
        is_deleted,
        is_admin,
        admin_level,
        unban_date,
        avatar,
        banner,
        bio,
        bio_html,
        signature,
        profile_background,
        avatar_frame,
        profile_music,
        profile_music_youtube,
        bot_account,
        board_creation_approved,
        show_nsfw,
        show_bots,
        theme,
        default_sort_type,
        default_listing_type,
        interface_language,
        email_notifications_enabled,
        creation_date,
        updated,
    );

    impl ToSafe for UserSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                display_name,
                is_banned,
                is_deleted,
                is_admin,
                admin_level,
                unban_date,
                avatar,
                banner,
                bio,
                bio_html,
                signature,
                profile_background,
                avatar_frame,
                profile_music,
                profile_music_youtube,
                bot_account,
                board_creation_approved,
                show_nsfw,
                show_bots,
                theme,
                default_sort_type,
                default_listing_type,
                interface_language,
                email_notifications_enabled,
                creation_date,
                updated,
            )
        }
    }
}