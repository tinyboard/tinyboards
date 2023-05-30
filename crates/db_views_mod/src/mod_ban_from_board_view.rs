use crate::structs::{ModBanFromBoardView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        board::boards::BoardSafe, moderator::mod_actions::ModBanFromBoard, local_user::users::UserSafe,
    },
    schema::{boards, mod_ban_from_board, users},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type ModBanFromBoardViewTuple = (ModBanFromBoard, Option<UserSafe>, BoardSafe, UserSafe);

impl ModBanFromBoardView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let user_alias = diesel::alias!(users as user_1);
        let mod_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_ban_from_board::mod_person_id
            .eq(users::id)
            .and(show_mod_names_expr.or(users::id.eq(mod_id_join)));

        let mut query = mod_ban_from_board::table
            .left_join(users::table.on(mod_names_join))
            .inner_join(boards::table)
            .inner_join(
                user_alias.on(mod_ban_from_board::other_person_id.eq(user_alias.field(users::id))),
            )
            .select((
                mod_ban_from_board::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
            ))
            .into_boxed();

        if let Some(mod_person_id) = params.mod_person_id {
            query = query.filter(mod_ban_from_board::mod_person_id.eq(mod_person_id));
        };

        if let Some(board_id) = params.board_id {
            query = query.filter(mod_ban_from_board::board_id.eq(board_id));
        };

        if let Some(other_person_id) = params.other_person_id {
            query = query.filter(mod_ban_from_board::other_person_id.eq(other_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_ban_from_board::when_.desc())
            .load::<ModBanFromBoardViewTuple>(conn)
            .await?;

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
