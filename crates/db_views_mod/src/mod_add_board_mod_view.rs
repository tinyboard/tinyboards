use crate::structs::{ModAddBoardModView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{moderator::mod_actions::ModAddBoardMod, person::person::PersonSafe},
    schema::{mod_add_board_mod, person},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type ModAddBoardModViewTuple = (ModAddBoardMod, Option<PersonSafe>, PersonSafe);

impl ModAddBoardModView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_alias = diesel::alias!(person as person_1);
        let mod_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_add_board_mod::mod_person_id
            .eq(person::id)
            .and(show_mod_names_expr.or(person::id.eq(mod_id_join)));

        let mut query = mod_add_board_mod::table
            .left_join(person::table.on(mod_names_join))
            .inner_join(
                person_alias.on(mod_add_board_mod::other_person_id.eq(person_alias.field(person::id))),
            )
            .select((
                mod_add_board_mod::all_columns,
                PersonSafe::safe_columns_tuple().nullable(),
                person_alias.fields(PersonSafe::safe_columns_tuple()),
            ))
            .into_boxed();

        if let Some(mod_person_id) = params.mod_person_id {
            query = query.filter(mod_add_board_mod::mod_person_id.eq(mod_person_id));
        };

        if let Some(other_person_id) = params.other_person_id {
            query = query.filter(mod_add_board_mod::other_person_id.eq(other_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_add_board_mod::when_.desc())
            .load::<ModAddBoardModViewTuple>(conn)
            .await?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for ModAddBoardModView {
    type DbTuple = ModAddBoardModViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_add_board_mod: a.0,
                moderator: a.1,
                modded_user: a.2,
            })
            .collect::<Vec<Self>>()
    }
}
