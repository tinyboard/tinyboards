use crate::structs::{AdminPurgeCommentView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    schema::{
        admin_purge_comment,
        user_,
        comment,
    },
    models::{
        moderator::admin_actions::AdminPurgeComment,
        user::user::UserSafe,
        comment::comment::Comment,
    },
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type AdminPurgeCommentViewTuple = (
    AdminPurgeComment,
    Option<UserSafe>,
    Comment,
);

impl AdminPurgeCommentView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let admin_user_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let admin_names_join = admin_purge_comment::admin_id
            .eq(user_::id)
            .and(show_mod_names_expr.or(user_::id.eq(admin_user_id_join)));
        
        let mut query = admin_purge_comment::table
            .left_join(user_::table.on(admin_names_join))
            .inner_join(comment::table)
            .select((
                admin_purge_comment::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                comment::all_columns,
            ))
            .into_boxed();

        if let Some(admin_user_id) = params.mod_user_id {
            query = query.filter(admin_purge_comment::admin_id.eq(admin_user_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(admin_purge_comment::when_.desc())
            .load::<AdminPurgeCommentViewTuple>(conn)?;

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
                comment: a.2,
            })
            .collect::<Vec<Self>>()
    }
}