use crate::structs::{ModLogParams, ModRemoveBoardView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        board::boards::BoardSafe, moderator::mod_actions::ModRemoveBoard, person::person::PersonSafe,
    },
    schema::{boards, mod_remove_board, person},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type ModRemoveBoardViewTuple = (ModRemoveBoard, Option<PersonSafe>, BoardSafe);

impl ModRemoveBoardView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mod_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_remove_board::mod_person_id
            .eq(person::id)
            .and(show_mod_names_expr.or(person::id.eq(mod_id_join)));

        let mut query = mod_remove_board::table
            .left_join(person::table.on(mod_names_join))
            .inner_join(boards::table)
            .select((
                mod_remove_board::all_columns,
                PersonSafe::safe_columns_tuple().nullable(),
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(board_id) = params.board_id {
            query = query.filter(mod_remove_board::board_id.eq(board_id));
        };

        if let Some(mod_person_id) = params.mod_person_id {
            query = query.filter(mod_remove_board::mod_person_id.eq(mod_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_remove_board::when_.desc())
            .load::<ModRemoveBoardViewTuple>(conn)
            .await?;

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
