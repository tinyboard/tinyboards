use crate::structs::{ModLockPostView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        board::boards::BoardSafe, moderator::mod_actions::ModLockPost, post::posts::Post,
        user::user::UserSafe,
    },
    schema::{board, mod_lock_post, post, users},
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type ModLockPostViewTuple = (ModLockPost, Option<UserSafe>, Post, BoardSafe);

impl ModLockPostView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let user_alias = diesel::alias!(users as user_1);
        let mod_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_lock_post::mod_user_id
            .eq(users::id)
            .and(show_mod_names_expr.or(users::id.eq(mod_id_join)));

        let mut query = mod_lock_post::table
            .left_join(users::table.on(mod_names_join))
            .inner_join(post::table)
            .inner_join(board::table.on(post::board_id.eq(board::id)))
            .inner_join(user_alias.on(post::creator_id.eq(user_alias.field(users::id))))
            .select((
                mod_lock_post::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(mod_user_id) = params.mod_user_id {
            query = query.filter(mod_lock_post::mod_user_id.eq(mod_user_id));
        };

        if let Some(board_id) = params.board_id {
            query = query.filter(post::board_id.eq(board_id));
        };

        if let Some(other_user_id) = params.other_user_id {
            query = query.filter(user_alias.field(users::id).eq(other_user_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_lock_post::when_.desc())
            .load::<ModLockPostViewTuple>(conn)?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for ModLockPostView {
    type DbTuple = ModLockPostViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_lock_post: a.0,
                moderator: a.1,
                post: a.2,
                board: a.3,
            })
            .collect::<Vec<Self>>()
    }
}
