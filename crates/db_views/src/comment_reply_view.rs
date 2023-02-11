use crate::structs::CommentReplyView;
use diesel::{
    dsl::now,
    result::Error,
    BoolExpressionMethods,
    ExpressionMethods,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl, 
};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    schema::{
        comments,
        comment_aggregates,
        comment_votes,
        comment_reply,
        user_comment_save,
        boards,
        board_subscriptions,
        board_user_bans,
        users,
        user_blocks,
        posts,
    },
    models::{
        comment::comments::Comment,
        comment::user_comment_save::CommentSaved,
        comment::comment_reply::CommentReply,
        board::boards::BoardSafe,
        board::board_subscriptions::BoardSubscriber,
        board::board_user_bans::BoardUserBan,
        user::users::UserSafe,
        user::user_blocks::UserBlock,
        post::posts::Post,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, limit_and_offset, get_conn, DbPool},
    CommentSortType,
};
use typed_builder::TypedBuilder;
use diesel_async::RunQueryDsl;

type CommentReplyViewTuple = (
    CommentReply,
    Comment,
    UserSafe,
    Post,
    BoardSafe,
    UserSafe,
    CommentAggregates,
    Option<BoardUserBan>,
    Option<BoardSubscriber>,
    Option<CommentSaved>,
    Option<UserBlock>,
    Option<i16>,
);

impl CommentReplyView {
    pub async fn read(
        pool: &DbPool,
        comment_reply_id: i32,
        user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let user_alias = diesel::alias!(users as users_alias);

        let user_id_join = user_id.unwrap_or(-1);

        let (
            comment_reply,
            comment,
            creator,
            post,
            board,
            recipient,
            counts,
            creator_banned_from_board,
            subscriber,
            saved,
            creator_blocked,
            my_vote,
        ) = comment_reply::table
            .find(comment_reply_id)
            .inner_join(comments::table)
            .inner_join(users::table.on(comments::creator_id.eq(users::id)))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(user_alias)
            .inner_join(comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)))
            .left_join(
                board_user_bans::table.on(
                    boards::id
                        .eq(board_user_bans::board_id)
                        .and(board_user_bans::user_id.eq(comments::creator_id))
                        .and(
                            board_user_bans::expires
                                .is_null()
                                .or(board_user_bans::expires.gt(now)),
                        ),
                ),
            )
            .left_join(
                board_subscriptions::table.on(
                    posts::board_id
                        .eq(board_subscriptions::board_id)
                        .and(board_subscriptions::user_id.eq(user_id_join))
                )
            )
            .left_join(
                user_comment_save::table.on(
                    comments::id
                        .eq(user_comment_save::comment_id)
                        .and(user_comment_save::user_id.eq(user_id_join))
                )
            )
            .left_join(
                user_blocks::table.on(
                    comments::creator_id
                        .eq(user_blocks::target_id)
                        .and(user_blocks::user_id.eq(user_id_join))
                )
            )
            .left_join(
                comment_votes::table.on(
                    comments::id
                        .eq(comment_votes::comment_id)
                        .and(comment_votes::user_id.eq(user_id_join))
                )
            )
            .select((
                comment_reply::all_columns,
                comments::all_columns,
                UserSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                user_comment_save::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .first::<CommentReplyViewTuple>(conn)
            .await?;

        Ok( CommentReplyView {
            comment_reply,
            comment,
            creator,
            post,
            board,
            recipient,
            counts,
            creator_banned_from_board: creator_banned_from_board.is_some(),
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            saved: saved.is_some(),
            creator_blocked: creator_blocked.is_some(),
            my_vote,
        })
    }

    /// Gets number of unread replies
    pub async fn get_unread_replies(pool: &DbPool, user_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use diesel::dsl::count;

        comment_reply::table
        .inner_join(comments::table)
        .filter(comment_reply::recipient_id.eq(user_id))
        .filter(comment_reply::read.eq(false))
        .filter(comments::is_deleted.eq(false))
        .filter(comments::is_removed.eq(false))
        .select(count(comment_reply::id))
        .first::<i64>(conn)
        .await
    }

    /// Marks all unread as read for a user
    pub async fn mark_all_replies_as_read(pool: &DbPool, user_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(comment_reply::table)
            .filter(comment_reply::read.eq(false))
            .filter(comment_reply::recipient_id.eq(user_id))
            .set(comment_reply::read.eq(true))
            .execute(conn)
            .await
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentReplyQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    user_id: Option<i32>,
    recipient_id: Option<i32>,
    sort: Option<CommentSortType>,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct CommentReplyQueryResponse {
    pub replies: Vec<CommentReplyView>,
    pub count: i64,
    pub unread: i64,
}

impl <'a> CommentReplyQuery<'a> {
    pub async fn list(self) -> Result<CommentReplyQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;

        let user_alias = diesel::alias!(users as user_alias);

        let user_id_join = self.user_id.unwrap_or(-1);

        let mut query = comment_reply::table
        .inner_join(comments::table)
        .inner_join(users::table.on(comments::creator_id.eq(users::id)))
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
        .inner_join(user_alias)
        .inner_join(comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)))
        .left_join(
            board_user_bans::table.on(
                boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    ),
            ),
        )
        .left_join(
            board_subscriptions::table.on(
                posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))
            )
        )
        .left_join(
            user_comment_save::table.on(
                comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::user_id.eq(user_id_join))
            )
        )
        .left_join(
            user_blocks::table.on(
                comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))
            )
        )
        .left_join(
            comment_votes::table.on(
                comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::user_id.eq(user_id_join))
            )
        )
        .select((
            comment_reply::all_columns,
            comments::all_columns,
            UserSafe::safe_columns_tuple(),
            posts::all_columns,
            BoardSafe::safe_columns_tuple(),
            user_alias.fields(UserSafe::safe_columns_tuple()),
            comment_aggregates::all_columns,
            board_user_bans::all_columns.nullable(),
            board_subscriptions::all_columns.nullable(),
            user_comment_save::all_columns.nullable(),
            user_blocks::all_columns.nullable(),
            comment_votes::score.nullable(),
        ))
        .into_boxed();

        let mut count_query = comment_reply::table
        .inner_join(comments::table)
        .inner_join(users::table.on(comments::creator_id.eq(users::id)))
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
        .inner_join(user_alias)
        .inner_join(comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)))
        .into_boxed();

        if let Some(recipient_id) = self.recipient_id {
            query = query.filter(comment_reply::recipient_id.eq(recipient_id));
            count_query = count_query.filter(comment_reply::recipient_id.eq(recipient_id));
        }

        if self.unread_only.unwrap_or(false) {
            query = query.filter(comment_reply::read.eq(false));
            count_query = count_query.filter(comment_reply::read.eq(false));
        }

        query = match self.sort.unwrap_or(CommentSortType::Hot) {
            CommentSortType::Hot => query
                .then_order_by(hot_rank(comment_aggregates::score, comment_aggregates::creation_date).desc())
                .then_order_by(comment_aggregates::creation_date.desc()),
            CommentSortType::New => query.then_order_by(comments::creation_date.desc()),
            CommentSortType::Old => query.then_order_by(comments::creation_date.asc()),
            CommentSortType::Top => query.then_order_by(comment_aggregates::score.desc()),
        };

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<CommentReplyViewTuple>(conn)
            .await?;
        
        let replies = CommentReplyView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;
        let unread = CommentReplyView::get_unread_replies(self.pool, user_id_join).await?;

        Ok(CommentReplyQueryResponse { replies, count, unread })
    }
}

impl ViewToVec for CommentReplyView {
    type DbTuple = CommentReplyViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                comment_reply: a.0,
                comment: a.1,
                creator: a.2,
                post: a.3,
                board: a.4,
                recipient: a.5,
                counts: a.6,
                creator_banned_from_board: a.7.is_some(),
                subscribed: BoardSubscriber::to_subscribed_type(&a.8),
                saved: a.9.is_some(),
                creator_blocked: a.10.is_some(),
                my_vote: a.11,
            })
            .collect::<Vec<Self>>()
    }
}