use crate::structs::{ModLogParams, ModRemoveCommentView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        board::boards::BoardSafe, comment::comments::Comment,
        moderator::mod_actions::ModRemoveComment, post::posts::Post, person::person::PersonSafe,
    },
    schema::{boards, comments, mod_remove_comment, posts, person},
    traits::{ToSafe, ViewToVec},
    utils::{limit_and_offset, DbPool, get_conn},
};
use diesel_async::RunQueryDsl;

type ModRemoveCommentViewTuple = (
    ModRemoveComment,
    Option<PersonSafe>,
    Comment,
    PersonSafe,
    Post,
    BoardSafe,
);

impl ModRemoveCommentView {
    pub async fn list(pool: &DbPool, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_alias = diesel::alias!(person as person_1);
        let mod_id_join = params.mod_person_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_remove_comment::mod_person_id
            .eq(person::id)
            .and(show_mod_names_expr.or(person::id.eq(mod_id_join)));

        let mut query = mod_remove_comment::table
            .left_join(person::table.on(mod_names_join))
            .inner_join(comments::table)
            .inner_join(person_alias.on(comments::creator_id.eq(person_alias.field(person::id))))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .select((
                mod_remove_comment::all_columns,
                PersonSafe::safe_columns_tuple().nullable(),
                comments::all_columns,
                person_alias.fields(PersonSafe::safe_columns_tuple()),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(board_id) = params.board_id {
            query = query.filter(posts::board_id.eq(board_id));
        };

        if let Some(mod_person_id) = params.mod_person_id {
            query = query.filter(mod_remove_comment::mod_person_id.eq(mod_person_id));
        };

        if let Some(other_person_id) = params.other_person_id {
            query = query.filter(person_alias.field(person::id).eq(other_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_remove_comment::when_.desc())
            .load::<ModRemoveCommentViewTuple>(conn)
            .await?;

        let results = Self::from_tuple_to_vec(res);

        Ok(results)
    }
}

impl ViewToVec for ModRemoveCommentView {
    type DbTuple = ModRemoveCommentViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                mod_remove_comment: a.0,
                moderator: a.1,
                comment: a.2,
                commenter: a.3,
                post: a.4,
                board: a.5,
            })
            .collect::<Vec<Self>>()
    }
}
