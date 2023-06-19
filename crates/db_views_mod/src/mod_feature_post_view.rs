use crate::structs::{ModLogParams, ModFeaturePostView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        board::boards::BoardSafe, moderator::mod_actions::ModFeaturePost, post::posts::Post, person::person::PersonSafe,
    },
    schema::{boards, mod_feature_post, posts, person},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type ModFeaturePostViewTuple = (ModFeaturePost, Option<PersonSafe>, Post, BoardSafe);

impl ModFeaturePostView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_alias = diesel::alias!(person as person_1);
        let mod_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_feature_post::mod_person_id
            .eq(person::id)
            .and(show_mod_names_expr.or(person::id.eq(mod_id_join)));

        let mut query = mod_feature_post::table
            .left_join(person::table.on(mod_names_join))
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person_alias.on(posts::creator_id.eq(person_alias.field(person::id))))
            .select((
                mod_feature_post::all_columns,
                PersonSafe::safe_columns_tuple().nullable(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(board_id) = params.board_id {
            query = query.filter(posts::board_id.eq(board_id));
        };

        if let Some(mod_person_id) = params.mod_person_id {
            query = query.filter(mod_feature_post::mod_person_id.eq(mod_person_id));
        };

        if let Some(other_person_id) = params.other_person_id {
            query = query.filter(person_alias.field(person::id).eq(other_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_feature_post::when_.desc())
            .load::<ModFeaturePostViewTuple>(conn)
            .await?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for ModFeaturePostView {
    type DbTuple = ModFeaturePostViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_feature_post: a.0,
                moderator: a.1,
                post: a.2,
                board: a.3,
            })
            .collect::<Vec<Self>>()
    }
}
