use crate::{
    aggregates::structs::PersonAggregates,
    models::person::local_user::LocalUser,
    models::person::person::{Person, PersonForm},
    models::person::user::User,
    newtypes::{DbUrl, UserId},
    schema::{instance, person, person_aggregates},
    traits::{ApubActor, Crud},
    utils::{functions::lower, fuzzy_search, get_conn, limit_and_offset, naive_now, DbPool},
    UserListingType, UserSortType,
};
use diesel::pg::expression::dsl::any;

use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
//use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl Person {
    pub async fn get_by_name(pool: &DbPool, username: String) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::person;

        person::table
            .filter(
                person::name.ilike(
                    username
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
    }

    pub async fn get_user_by_id(pool: &DbPool, id: i32) -> Result<User, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{local_user, person, person_aggregates};

        person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(local_user::person_id.eq(person::id)))
            .filter(person::id.eq(id))
            .first::<(Self, PersonAggregates, Option<LocalUser>)>(conn)
            .await
            .map(User::from)
    }

    
    pub async fn get_user_for_name(pool: &DbPool, username: String) -> Result<User, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{local_user, person, person_aggregates};

        person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(local_user::person_id.eq(person::id)))
            .filter(
                person::name.ilike(
                    username
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<(Self, PersonAggregates, Option<LocalUser>)>(conn)
            .await
            .map(User::from)
    }

    pub async fn search_by_name(pool: &DbPool, query: &str) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::person::dsl::*;
        person
            .filter(name.ilike(fuzzy_search(query)))
            .load(conn)
            .await
    }

    pub async fn get_users_for_ids(pool: &DbPool, ids: Vec<i32>) -> Result<Vec<User>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{local_user, person, person_aggregates};

        person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(local_user::person_id.eq(person::id)))
            .filter(person::id.eq_any(ids))
            .load::<(Self, PersonAggregates, Option<LocalUser>)>(conn)
            .await
            .map(|res| res.into_iter().map(User::from).collect::<Vec<User>>())
    }

    pub async fn update_settings(
        pool: &DbPool,
        id_: i32,
        form: &PersonForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(person::table.find(id_))
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
        use crate::schema::person::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(person.find(id_))
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
        use crate::schema::person::dsl::*;
        diesel::update(person.find(id_))
            .set((
                is_admin.eq(admin_level_ > 0),
                admin_level.eq(admin_level_),
                updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn delete_account(pool: &DbPool, person_id: i32) -> Result<Person, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user;

        // Set the local user info to none
        diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
            .set((local_user::email.eq::<Option<String>>(None),))
            .execute(conn)
            .await?;

        diesel::update(person::table.find(person_id))
            .set((
                person::display_name.eq::<Option<String>>(None),
                person::avatar.eq::<Option<String>>(None),
                person::banner.eq::<Option<String>>(None),
                person::bio.eq::<Option<String>>(None),
                person::is_deleted.eq(true),
                person::updated.eq(naive_now()),
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
        use crate::schema::person::dsl::*;

        diesel::update(person)
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
        use crate::schema::person::dsl::*;

        let is_permanent_ban = unban_date_.is_none();

        if is_permanent_ban {
            // clear profile
            diesel::update(person.find(self.id))
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
            diesel::update(person.find(self.id))
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
        use crate::schema::person::dsl::*;

        diesel::update(person.find(self.id))
            .set((
                is_banned.eq(false),
                unban_date.eq(None::<NaiveDateTime>),
                updated.eq(naive_now()),
            ))
            .execute(conn)
            .await
            .map(|_| ())
    }

    /// Update or insert the person.
    ///
    /// necessary for federation because Apub does not distinguish between these actions
    pub async fn upsert(pool: &DbPool, form: &PersonForm) -> Result<Person, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(person::table)
            .values(form)
            .on_conflict(person::actor_id)
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    /// Load a bunch of people with their counts.
    pub async fn list_with_counts(
        pool: &DbPool,
        sort: UserSortType,
        limit: Option<i64>,
        page: Option<i64>,
        listing_type: UserListingType,
        search_term: Option<String>,
    ) -> Result<Vec<User>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{local_user, person, person_aggregates};

        let mut query = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(local_user::person_id.eq(person::id)))
            .select((
                person::all_columns,
                person_aggregates::all_columns,
                local_user::all_columns.nullable(),
            ))
            .into_boxed();

        query = match sort {
            UserSortType::New => query.then_order_by(person::creation_date.desc()),
            UserSortType::Old => query.then_order_by(person::creation_date.asc()),
            UserSortType::MostRep => query.then_order_by(person_aggregates::rep.desc()),
            UserSortType::MostPosts => query.then_order_by(person_aggregates::post_count.desc()),
            UserSortType::MostComments => {
                query.then_order_by(person_aggregates::comment_count.desc())
            }
        };

        if let Some(ref search) = search_term {
            query = query.filter(
                person::name
                    .ilike(fuzzy_search(search))
                    .or(person::display_name.ilike(fuzzy_search(search))),
            );
        }

        query = match listing_type {
            UserListingType::All => query,
            UserListingType::NotBanned => query.filter(person::is_banned.eq(false)),
            UserListingType::Banned => query.filter(person::is_banned.eq(true)),
            UserListingType::Admins => query.filter(person::is_admin.eq(true)),
        };

        let (limit, offset) = limit_and_offset(page, limit)?;

        query = query.limit(limit).offset(offset);

        query
            .load::<(Person, PersonAggregates, Option<LocalUser>)>(conn)
            .await
            .map(|res| res.into_iter().map(User::from).collect::<Vec<User>>())
    }
}

#[async_trait::async_trait]
impl Crud for Person {
    type Form = PersonForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        person::table.find(id_).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(person::table.find(id_)).execute(conn).await
    }
    async fn create(pool: &DbPool, form: &PersonForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_person = diesel::insert_into(person::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_person)
    }
    async fn update(pool: &DbPool, id_: i32, form: &PersonForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(person::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

pub mod safe_type {
    use crate::{models::person::person::*, schema::person::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        display_name,
        is_banned,
        local,
        actor_id,
        creation_date,
        updated,
        avatar,
        signature,
        is_deleted,
        unban_date,
        banner,
        bio,
        inbox_url,
        shared_inbox_url,
        bot_account,
        last_refreshed_date,
        is_admin,
        instance,
        admin_level,
        profile_background,
        avatar_frame,
        bio_html,
        profile_music,
        profile_music_youtube,
    );

    impl ToSafe for PersonSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                display_name,
                is_banned,
                local,
                actor_id,
                creation_date,
                updated,
                avatar,
                signature,
                is_deleted,
                unban_date,
                banner,
                bio,
                inbox_url,
                shared_inbox_url,
                bot_account,
                last_refreshed_date,
                is_admin,
                instance,
                admin_level,
                profile_background,
                avatar_frame,
                bio_html,
                profile_music,
                profile_music_youtube,
            )
        }
    }
}

#[async_trait::async_trait]
impl ApubActor for Person {
    async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        Ok(person::table
            .into_boxed()
            .filter(person::is_deleted.eq(false))
            .filter(person::actor_id.eq(object_id.to_string()))
            .first::<Person>(conn)
            .await
            .ok()
            .map(Into::into))
    }

    async fn read_from_name(
        pool: &DbPool,
        from_name: &str,
        include_deleted: bool,
    ) -> Result<Person, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut q = person::table
            .into_boxed()
            .filter(person::local.eq(true))
            .filter(lower(person::name).eq(from_name.to_lowercase()));
        if !include_deleted {
            q = q.filter(person::is_deleted.eq(false))
        }
        q.first::<Self>(conn).await
    }

    async fn read_from_name_and_domain(
        pool: &DbPool,
        person_name: &str,
        for_domain: &str,
    ) -> Result<Person, Error> {
        let conn = &mut get_conn(pool).await?;

        person::table
            .inner_join(instance::table)
            .filter(lower(person::name).eq(person_name.to_lowercase()))
            .filter(instance::domain.eq(for_domain))
            .select(person::all_columns)
            .first::<Self>(conn)
            .await
    }
}
