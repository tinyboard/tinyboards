use crate::structs::{RegistrationApplicationView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        site::registration_applications::RegistrationApplication, user::users::{UserSafe, UserSettings},
    },
    schema::{users, registration_applications},
    traits::{ViewToVec, ToSafe},
    utils::limit_and_offset,
};
use typed_builder::TypedBuilder;

type RegistrationApplicationViewTuple = (
    RegistrationApplication,
    UserSettings,
    UserSafe,
    Option<UserSafe>,
);

impl RegistrationApplicationView {
    pub fn read(
        conn: &mut PgConnection,
        app_id: i32,
    ) -> Result<Self, Error> {
        let user_alias = diesel::alias!(users as users_alias);
        let (
            application,
            applicant_settings,
            applicant,
            admin,
        ) = registration_applications::table
        .find(app_id)
        .inner_join(users::table.on(registration_applications::user_id.eq(users::id)))
        .left_join(user_alias.on(registration_applications::admin_id.eq(user_alias.field(users::id).nullable())))
        .order_by(registration_applications::published.desc())
        .select((
            registration_applications::all_columns,
            UserSettings::safe_columns_tuple(),
            UserSafe::safe_columns_tuple(),
            user_alias.fields(UserSafe::safe_columns_tuple()).nullable(),
        ))
        .first::<RegistrationApplicationViewTuple>(conn)?;

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
    conn: &'a mut PgConnection,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct ApplicationQueryResponse {
    pub applications: Vec<RegistrationApplicationView>,
    pub count: i64,
}

impl<'a> ApplicationQuery<'a> {
    pub fn list(self) -> Result<ApplicationQueryResponse, Error> {
        
        let user_alias = diesel::alias!(users as users_alias);
        let query = registration_applications::table
            .inner_join(users::table.on(registration_applications::user_id.eq(users::id)))
            .left_join(user_alias.on(registration_applications::admin_id.eq(user_alias.field(users::id).nullable())))
            .order_by(registration_applications::published.desc())
            .select((
                registration_applications::all_columns,
                UserSettings::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()).nullable(),
            ))
            .into_boxed();


        let count_query = registration_applications::table
            .inner_join(users::table.on(registration_applications::user_id.eq(users::id)))
            .left_join(user_alias.on(registration_applications::admin_id.eq(user_alias.field(users::id).nullable())))
            .into_boxed();

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<RegistrationApplicationViewTuple>(self.conn)?;

        let applications = RegistrationApplicationView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(self.conn)?;

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
