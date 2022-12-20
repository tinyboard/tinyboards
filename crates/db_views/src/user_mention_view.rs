use crate::structs::UserMentionView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    map_to_comment_sort_type,
    models::{
        board::board_subscriptions::BoardSubscriber, board::board_user_bans::BoardUserBan,
        board::boards::BoardSafe, comment::comments::Comment,
        comment::user_comment_save::CommentSaved, post::posts::Post, user::user::UserSafe,
        user::user_blocks::UserBlock, user::user_mention::UserMention,
    },
    schema::{
        board_subscriptions, board_user_bans, boards, comment, comment_aggregates, comment_saved,
        comment_vote, post, user_block, user_mention, users,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, limit_and_offset},
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

impl UserMentionView {
    pub fn read(
        conn: &mut PgConnection,
        user_mention_id: i32,
        user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_alias = diesel::alias!(users as user_1);

        let user_id_join = user_id.unwrap_or(-1);

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
        ) = user_mention::table
            .find(user_mention_id)
            .inner_join(comment::table)
            .inner_join(users::table.on(comment::creator_id.eq(users::id)))
            .inner_join(post::table.on(comment::post_id.eq(post::id)))
            .inner_join(boards::table.on(post::board_id.eq(boards::id)))
            .inner_join(user_alias)
            .inner_join(
                comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comment::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(post::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::user_id.eq(user_id_join))),
            )
            .left_join(
                user_block::table.on(comment::creator_id
                    .eq(user_block::target_id)
                    .and(user_block::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_vote::table.on(comment::id
                    .eq(comment_vote::comment_id)
                    .and(comment_vote::user_id.eq(user_id_join))),
            )
            .select((
                user_mention::all_columns,
                comment::all_columns,
                UserSafe::safe_columns_tuple(),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                user_block::all_columns.nullable(),
                comment_vote::score.nullable(),
            ))
            .first::<UserMentionViewTuple>(conn)?;

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
    pub fn get_unread_mentions(conn: &mut PgConnection, user_id: i32) -> Result<i64, Error> {
        use diesel::dsl::*;

        user_mention::table
            .inner_join(comment::table)
            .filter(user_mention::recipient_id.eq(user_id))
            .filter(user_mention::read.eq(false))
            .filter(comment::deleted.eq(false))
            .filter(comment::removed.eq(false))
            .select(count(user_mention::id))
            .first::<i64>(conn)
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct UserMentionQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    user_id: Option<i32>,
    recipient_id: Option<i32>,
    sort: Option<String>,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

impl<'a> UserMentionQuery<'a> {
    pub fn list(self) -> Result<Vec<UserMentionView>, Error> {
        use diesel::dsl::*;

        let user_alias = diesel::alias!(users as user_1);

        let user_id_join = self.user_id.unwrap_or(-1);

        let mut query = user_mention::table
            .inner_join(comment::table)
            .inner_join(users::table.on(comment::creator_id.eq(users::id)))
            .inner_join(post::table.on(comment::post_id.eq(post::id)))
            .inner_join(boards::table.on(post::board_id.eq(boards::id)))
            .inner_join(user_alias)
            .inner_join(
                comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comment::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(post::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::user_id.eq(user_id_join))),
            )
            .left_join(
                user_block::table.on(comment::creator_id
                    .eq(user_block::target_id)
                    .and(user_block::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_vote::table.on(comment::id
                    .eq(comment_vote::comment_id)
                    .and(comment_vote::user_id.eq(user_id_join))),
            )
            .select((
                user_mention::all_columns,
                comment::all_columns,
                UserSafe::safe_columns_tuple(),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
                user_alias.fields(UserSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                user_block::all_columns.nullable(),
                comment_vote::score.nullable(),
            ))
            .into_boxed();

        if let Some(recipient_id) = self.recipient_id {
            query = query.filter(user_mention::recipient_id.eq(recipient_id));
        }

        if self.unread_only.unwrap_or(false) {
            query = query.filter(user_mention::read.eq(false));
        }

        let sort = map_to_comment_sort_type(self.sort.as_deref());

        query = match sort {
            CommentSortType::Hot => query
                .then_order_by(
                    hot_rank(comment_aggregates::score, comment_aggregates::published).desc(),
                )
                .then_order_by(comment_aggregates::published.desc()),
            CommentSortType::New => query.then_order_by(comment::published.desc()),
            CommentSortType::Old => query.then_order_by(comment::published.asc()),
            CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
        };

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<UserMentionViewTuple>(self.conn)?;

        Ok(UserMentionView::from_tuple_to_vec(res))
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
