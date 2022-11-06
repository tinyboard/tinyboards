use crate::structs::{ModRemoveCommentView, ModLogParams};
use diesel::{result::Error, *};
use tinyboards_db::{
    schema::{
        mod_remove_comment,
        user_,
        post,
        board,
        comment,
    },
    models::{
        moderator::mod_actions::ModRemoveComment,
        user::user::UserSafe,
        post::post::Post,
        comment::comment::Comment,
        board::board::BoardSafe,
    },
    traits::{ToSafe, ViewToVec},
    utils::limit_and_offset,
};

type ModRemoveCommentViewTuple = (
    ModRemoveComment,
    Option<UserSafe>,
    Comment,
    UserSafe,
    Post,
    BoardSafe,
);

impl ModRemoveCommentView {
    pub fn list(conn: &mut PgConnection, params: ModLogParams) -> Result<Vec<Self>, Error> {
        let user_alias = diesel::alias!(user_ as user_1);
        let mod_id_join = params.mod_user_id.unwrap_or(-1);
        let show_mod_names = !params.hide_modlog_names;
        let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

        let mod_names_join = mod_remove_comment::mod_user_id
            .eq(user_::id)
            .and(show_mod_names_expr.or(user_::id.eq(mod_id_join)));

        let mut query = mod_remove_comment::table
            .left_join(user_::table.on(mod_names_join))
            .inner_join(comment::table)
            .inner_join(user_alias.on(comment::creator_id.eq(user_alias.field(user_::id))))
            .inner_join(post::table.on(comment::post_id.eq(post::id)))
            .inner_join(board::table.on(post::board_id.eq(board::id)))
            .select((
                mod_remove_comment::all_columns,
                UserSafe::safe_columns_tuple().nullable(),
                comment::all_columns,
                user_alias.fields(UserSafe::safe_columns_tuple()),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
            ))
            .into_boxed();

        if let Some(board_id) = params.board_id {
            query = query.filter(post::board_id.eq(board_id));
        };

        if let Some(mod_user_id) = params.mod_user_id {
            query = query.filter(mod_remove_comment::mod_user_id.eq(mod_user_id));
        };

        if let Some(other_person_id) = params.other_user_id {
            query = query.filter(user_alias.field(user_::id).eq(other_person_id));
        };

        let (limit, offset) = limit_and_offset(params.page, params.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .order_by(mod_remove_comment::when_.desc())
            .load::<ModRemoveCommentViewTuple>(conn)?;

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