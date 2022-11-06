use crate::structs::{ModBanFromBoardView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    schema::{
        mod_ban_from_board,
        user_,
        board,
    },
    models::{
        moderator::mod_actions::ModBanFromBoard,
        user::user::UserSafe,
        board::board::BoardSafe,
    },
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type ModBanFromBoardViewTuple = (
    ModBanFromBoard,
    Option<UserSafe>,
    BoardSafe,
    UserSafe,
);

impl ModBanFromBoardView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let user_alias = diesel::alias!(user_ as user_1);
        let mod_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_ban_from_board::mod_user_id
            .eq(user_::id)
            .and(show_mod_names_expr.or(user_::id.eq(mod_id_join)));

        let mut query = mod_ban_from_board::table
            .left_join(user_::table.on(mod_names_join))
            .inner_join(board::table)
            .inner_join(
                user_alias.on(mod_ban_from_board::other_user_id.eq(user_alias.field(user_::id))),
            )
            .select((
                mod_ban_from_board::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
            ))
            .into_boxed();


            if let Some(mod_user_id) = params.mod_user_id {
                query = query.filter(mod_ban_from_board::mod_user_id.eq(mod_user_id));
            };

            if let Some(board_id) = params.board_id {
                query = query.filter(mod_ban_from_board::board_id.eq(board_id));
            };
    
            if let Some(other_user_id) = params.other_user_id {
                query = query.filter(mod_ban_from_board::other_user_id.eq(other_user_id));
            };
    
            let (limit, offset) = limit_and_offset(params.page, params.limit)?;
    
            let res = query
                .limit(limit)
                .offset(offset)
                .order_by(mod_ban_from_board::when_.desc())
                .load::<ModBanFromBoardViewTuple>(conn)?;

            let results = Self::from_tuple_to_vec(res);

            Ok(results)
    }
}

impl ViewToVec for ModBanFromBoardView {
    type DbTuple = ModBanFromBoardViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_ban_from_board: a.0,
                moderator: a.1,
                board: a.2,
                banned_user: a.3,
            })
            .collect::<Vec<Self>>()
    }
}