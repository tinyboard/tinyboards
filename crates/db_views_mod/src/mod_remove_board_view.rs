use crate::structs::{ModRemoveBoardView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    schema::{
        mod_remove_board,
        user_,
        board,
    },
    models::{
        user::user::UserSafe,
        moderator::mod_actions::ModRemoveBoard,
        board::board::BoardSafe,
    },
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type ModRemoveBoardViewTuple = (
    ModRemoveBoard,
    Option<UserSafe>,
    BoardSafe,
);

impl ModRemoveBoardView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let mod_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_remove_board::mod_user_id
            .eq(user_::id)
            .and(show_mod_names_expr.or(user_::id.eq(mod_id_join))); 

        let mut query = mod_remove_board::table
            .left_join(user_::table.on(mod_names_join))
            .inner_join(board::table)
            .select((
                mod_remove_board::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(board_id) = params.board_id {
            query = query.filter(mod_remove_board::board_id.eq(board_id));
        };

        if let Some(mod_user_id) = params.mod_user_id {
            query = query.filter(mod_remove_board::mod_user_id.eq(mod_user_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_remove_board::when_.desc())
            .load::<ModRemoveBoardViewTuple>(conn)?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for ModRemoveBoardView {
    type DbTuple = ModRemoveBoardViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_remove_board: a.0,
                moderator: a.1,
                board: a.2,
            })
            .collect::<Vec<Self>>()
    }
}