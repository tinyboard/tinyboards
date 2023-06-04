use crate::structs::{RegistrationApplicationView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        site::registration_applications::RegistrationApplication, person::local_user::{LocalUserSafe, LocalUserSettings},
    },
    schema::{local_user, registration_applications},
    traits::{ViewToVec, ToSafe},
    utils::{limit_and_offset, get_conn, DbPool},
};
use typed_builder::TypedBuilder;
use diesel_async::RunQueryDsl;

type RegistrationApplicationViewTuple = (
    RegistrationApplication,
    LocalUserSettings,
    LocalUserSafe,
    Option<LocalUserSafe>,
);

impl RegistrationApplicationView {
    pub async fn read (
        pool: &DbPool,
        app_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let local_user_alias = diesel::alias!(local_user as local_user_alias);
        let (
            application,
            applicant_settings,
            applicant,
            admin,
        ) = registration_applications::table
        .find(app_id)
        .inner_join(local_user::table.on(registration_applications::person_id.eq(local_user::person_id)))
        .left_join(local_user_alias.on(registration_applications::admin_id.eq(local_user_alias.field(local_user::person_id).nullable())))
        .order_by(registration_applications::published.desc())
        .select((
            registration_applications::all_columns,
            LocalUserSettings::safe_columns_tuple(),
            LocalUserSafe::safe_columns_tuple(),
            local_user_alias.fields(LocalUserSafe::safe_columns_tuple()).nullable(),
        ))
        .first::<RegistrationApplicationViewTuple>(conn)
        .await?;

        Ok(RegistrationApplicationView {
            application,
            applicant_settings,
            applicant,
            admin,
        })
    }
}


#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct ApplicationQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct ApplicationQueryResponse {
    pub applications: Vec<RegistrationApplicationView>,
    pub count: i64,
}

impl<'a> ApplicationQuery<'a> {
    pub async fn list(self) -> Result<ApplicationQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        
        let user_alias = diesel::alias!(local_user as users_alias);
        let query = registration_applications::table
            .inner_join(local_user::table.on(registration_applications::person_id.eq(local_user::person_id)))
            .left_join(user_alias.on(registration_applications::admin_id.eq(user_alias.field(local_user::person_id).nullable())))
            .order_by(registration_applications::published.desc())
            .select((
                registration_applications::all_columns,
                LocalUserSettings::safe_columns_tuple(),
                LocalUserSafe::safe_columns_tuple(),
                user_alias.fields(LocalUserSafe::safe_columns_tuple()).nullable(),
            ))
            .into_boxed();


        let count_query = registration_applications::table
            .inner_join(local_user::table.on(registration_applications::person_id.eq(local_user::person_id)))
            .left_join(user_alias.on(registration_applications::admin_id.eq(user_alias.field(local_user::person_id).nullable())))
            .into_boxed();

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<RegistrationApplicationViewTuple>(conn)
            .await?;

        let applications = RegistrationApplicationView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(ApplicationQueryResponse { applications, count })
        
    }
}

impl ViewToVec for RegistrationApplicationView {
    type DbTuple = RegistrationApplicationViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                application: a.0,
                applicant_settings: a.1,
                applicant: a.2,
                admin: a.3,
            })
            .collect::<Vec<Self>>()
    }
}
