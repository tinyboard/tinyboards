use crate::structs::{AdminPurgeBoardView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{moderator::admin_actions::AdminPurgeBoard, person::person::PersonSafe},
    schema::{admin_purge_board, person},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type AdminPurgeBoardViewTuple = (AdminPurgeBoard, Option<PersonSafe>);

impl AdminPurgeBoardView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let admin_person_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let admin_names_join = admin_purge_board::admin_id
            .eq(person::id)
            .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));

        let mut query = admin_purge_board::table
            .left_join(person::table.on(admin_names_join))
            .select((
                admin_purge_board::all_columns,
                PersonSafe::safe_columns_tuple().nullable(),
            ))
            .into_boxed();

        if let Some(admin_person_id) = params.mod_person_id {
            query = query.filter(admin_purge_board::admin_id.eq(admin_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(admin_purge_board::when_.desc())
            .load::<AdminPurgeBoardViewTuple>(conn)
            .await?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for AdminPurgeBoardView {
    type DbTuple = AdminPurgeBoardViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                admin_purge_board: a.0,
                admin: a.1,
            })
            .collect::<Vec<Self>>()
    }
}
