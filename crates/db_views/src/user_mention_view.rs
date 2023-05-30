use crate::structs::UserMentionView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber, board::board_user_bans::BoardUserBan,
        board::boards::BoardSafe, comment::comments::Comment,
        comment::user_comment_save::CommentSaved, post::posts::Post, user::user_blocks::UserBlock,
        user::user_mentions::UserMention, user::users::UserSafe,
    },
    schema::{
        board_subscriptions, board_user_bans, boards, comment_aggregates, comment_votes, comments,
        posts, user_blocks, user_comment_save, user_mentions, users,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, limit_and_offset, get_conn, DbPool},
    CommentSortType,
};
use typed_builder::TypedBuilder;

type UserMentionViewTuple = (
    UserMention,
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
use diesel_async::RunQueryDsl;

impl UserMentionView {
    pub async fn read(
        pool: &DbPool,
        user_mention_id: i32,
        person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let user_alias = diesel::alias!(users as user_1);

        let person_id_join = person_id.unwrap_or(-1);

        let (
            user_mention,
            comment,
            creator,
            post,
            board,
            recipient,
            counts,
            creator_banned_from_board,
            subscribed,
            saved,
            creator_blocked,
            my_vote,
        ) = user_mentions::table
            .find(user_mention_id)
            .inner_join(comments::table)
            .inner_join(users::table.on(comments::creator_id.eq(users::id)))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(user_alias)
            .inner_join(
                comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                user_comment_save::table.on(comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::person_id.eq(person_id_join))),
            )
            .left_join(
                user_blocks::table.on(comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                user_mentions::all_columns,
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
            .first::<UserMentionViewTuple>(conn)
            .await?;

        Ok(UserMentionView {
            user_mention,
            comment,
            creator,
            post,
            board,
            recipient,
            counts,
            creator_banned_from_board: creator_banned_from_board.is_some(),
            subscribed: BoardSubscriber::to_subscribed_type(&subscribed),
            saved: saved.is_some(),
            creator_blocked: creator_blocked.is_some(),
            my_vote,
        })
    }

    /// Gets count of unread mentions
    pub async fn get_unread_mentions(pool: &DbPool, person_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use diesel::dsl::*;

        user_mentions::table
            .inner_join(comments::table)
            .filter(user_mentions::recipient_id.eq(person_id))
            .filter(user_mentions::read.eq(false))
            .filter(comments::is_deleted.eq(false))
            .filter(comments::is_removed.eq(false))
            .select(count(user_mentions::id))
            .first::<i64>(conn)
            .await
    }

    /// Marks all unread as read for a user
    pub async fn mark_all_mentions_as_read(pool: &DbPool, person_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_mentions::table)
            .filter(user_mentions::read.eq(false))
            .filter(user_mentions::recipient_id.eq(person_id))
            .set(user_mentions::read.eq(true))
            .execute(conn)
            .await
    }

}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct UserMentionQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    person_id: Option<i32>,
    recipient_id: Option<i32>,
    sort: Option<CommentSortType>,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct UserMentionQueryResponse {
    pub mentions: Vec<UserMentionView>,
    pub count: i64,
    pub unread: i64,
}

impl<'a> UserMentionQuery<'a> {
    pub async fn list(self) -> Result<UserMentionQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        use diesel::dsl::*;

        let user_alias = diesel::alias!(users as user_1);

        let person_id_join = self.person_id.unwrap_or(-1);

        let mut query = user_mentions::table
            .inner_join(comments::table)
            .inner_join(users::table.on(comments::creator_id.eq(users::id)))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(user_alias)
            .inner_join(
                comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                user_comment_save::table.on(comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::person_id.eq(person_id_join))),
            )
            .left_join(
                user_blocks::table.on(comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                user_mentions::all_columns,
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

        let mut count_query = user_mentions::table
        .inner_join(comments::table)
        .inner_join(users::table.on(comments::creator_id.eq(users::id)))
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
        .inner_join(user_alias)
        .inner_join(
            comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
        )
        .into_boxed();
        
        if let Some(recipient_id) = self.recipient_id {
            query = query.filter(user_mentions::recipient_id.eq(recipient_id));
            count_query = count_query.filter(user_mentions::recipient_id.eq(recipient_id));
        }

        if self.unread_only.unwrap_or(false) {
            query = query.filter(user_mentions::read.eq(false));
            count_query = count_query.filter(user_mentions::read.eq(false));
        }

        query = match self.sort {
            Some(CommentSortType::Hot) => query
                .then_order_by(
                    hot_rank(comment_aggregates::score, comment_aggregates::creation_date).desc(),
                )
                .then_order_by(comment_aggregates::creation_date.desc()),
            Some(CommentSortType::New) => query.then_order_by(comments::creation_date.desc()),
            Some(CommentSortType::Old) => query.then_order_by(comments::creation_date.asc()),
            Some(CommentSortType::Top) => query.order_by(comment_aggregates::score.desc()),
            None => query
            .then_order_by(
                hot_rank(comment_aggregates::score, comment_aggregates::creation_date).desc(),
            )
            .then_order_by(comment_aggregates::creation_date.desc()),
        };

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<UserMentionViewTuple>(conn)
            .await?;

        let mentions = UserMentionView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;
        let unread = UserMentionView::get_unread_mentions(self.pool, person_id_join).await?;

        Ok(UserMentionQueryResponse { mentions, count, unread })
    }
}

impl ViewToVec for UserMentionView {
    type DbTuple = UserMentionViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                user_mention: a.0,
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
