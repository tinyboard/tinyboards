use crate::structs::{ModAddBoardView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, moderator::mod_actions::ModAddBoard, user::user::UserSafe},
    schema::{boards, mod_add_board, users},
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type ModAddBoardViewTuple = (ModAddBoard, Option<UserSafe>, BoardSafe, UserSafe);

impl ModAddBoardView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let user_alias = diesel::alias!(users as user_1);
        let mod_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_add_board::mod_user_id
            .eq(users::id)
            .and(show_mod_names_expr.or(users::id.eq(mod_id_join)));

        let mut query = mod_add_board::table
            .left_join(users::table.on(mod_names_join))
            .inner_join(boards::table)
            .inner_join(user_alias.on(mod_add_board::other_user_id.eq(user_alias.field(users::id))))
            .select((
                mod_add_board::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
            ))
            .into_boxed();

        if let Some(mod_user_id) = params.mod_user_id {
            query = query.filter(mod_add_board::mod_user_id.eq(mod_user_id));
        };

        if let Some(board_id) = params.board_id {
            query = query.filter(mod_add_board::board_id.eq(board_id));
        };

        if let Some(other_user_id) = params.other_user_id {
            query = query.filter(user_alias.field(users::id).eq(other_user_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_add_board::when_.desc())
            .load::<ModAddBoardViewTuple>(conn)?;

        let results = Self::from_tuple_to_vec(res);
        Ok(results)
    }
}

impl ViewToVec for ModAddBoardView {
    type DbTuple = ModAddBoardViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_add_board: a.0,
                moderator: a.1,
                board: a.2,
                modded_user: a.3,
            })
            .collect::<Vec<Self>>()
    }
}
