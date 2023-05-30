use crate::structs::{AdminPurgeCommentView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{moderator::admin_actions::AdminPurgeComment, local_user::users::UserSafe},
    schema::{admin_purge_comment, users},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type AdminPurgeCommentViewTuple = (AdminPurgeComment, Option<UserSafe>);

impl AdminPurgeCommentView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let admin_person_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let admin_names_join = admin_purge_comment::admin_id
            .eq(users::id)
            .and(show_mod_names_expr.or(users::id.eq(admin_person_id_join)));

        let mut query = admin_purge_comment::table
            .left_join(users::table.on(admin_names_join))
            .select((
                admin_purge_comment::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
            ))
            .into_boxed();

        if let Some(admin_person_id) = params.mod_person_id {
            query = query.filter(admin_purge_comment::admin_id.eq(admin_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(admin_purge_comment::when_.desc())
            .load::<AdminPurgeCommentViewTuple>(conn)
            .await?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for AdminPurgeCommentView {
    type DbTuple = AdminPurgeCommentViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                admin_purge_comment: a.0,
                admin: a.1,
            })
            .collect::<Vec<Self>>()
    }
}
