use crate::structs::PersonMentionView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber, board::board_person_bans::BoardPersonBan,
        board::boards::BoardSafe, comment::comments::Comment,
        comment::comment_saved::CommentSaved, post::posts::Post, person::person_blocks::PersonBlock,
        person::person_mentions::PersonMention, person::person::PersonSafe,
    },
    schema::{
        board_subscriptions, board_person_bans, boards, comment_aggregates, comment_votes, comments,
        posts, person_blocks, comment_saved, person_mentions, person,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, limit_and_offset, get_conn, DbPool},
    CommentSortType,
};
use typed_builder::TypedBuilder;

type PersonMentionViewTuple = (
    PersonMention,
    Comment,
    PersonSafe,
    Post,
    BoardSafe,
    PersonSafe,
    CommentAggregates,
    Option<BoardPersonBan>,
    Option<BoardSubscriber>,
    Option<CommentSaved>,
    Option<PersonBlock>,
    Option<i16>,
);
use diesel_async::RunQueryDsl;

impl PersonMentionView {
    pub async fn read(
        pool: &DbPool,
        user_mention_id: i32,
        person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_alias = diesel::alias!(person as person_1);

        let person_id_join = person_id.unwrap_or(-1);

        let (
            person_mention,
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
        ) = person_mentions::table
            .find(user_mention_id)
            .inner_join(comments::table)
            .inner_join(person::table.on(comments::creator_id.eq(person::id)))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person_alias)
            .inner_join(
                comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comments::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(comments::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                person_mentions::all_columns,
                comments::all_columns,
                PersonSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                person_alias.fields(PersonSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .first::<PersonMentionViewTuple>(conn)
            .await?;

        Ok(PersonMentionView {
            person_mention,
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

        person_mentions::table
            .inner_join(comments::table)
            .filter(person_mentions::recipient_id.eq(person_id))
            .filter(person_mentions::read.eq(false))
            .filter(comments::is_deleted.eq(false))
            .filter(comments::is_removed.eq(false))
            .select(count(person_mentions::id))
            .first::<i64>(conn)
            .await
    }

    /// Marks all unread as read for a user
    pub async fn mark_all_mentions_as_read(pool: &DbPool, person_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(person_mentions::table)
            .filter(person_mentions::read.eq(false))
            .filter(person_mentions::recipient_id.eq(person_id))
            .set(person_mentions::read.eq(true))
            .execute(conn)
            .await
    }

}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PersonMentionQuery<'a> {
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
pub struct PersonMentionQueryResponse {
    pub mentions: Vec<PersonMentionView>,
    pub count: i64,
    pub unread: i64,
}

impl<'a> PersonMentionQuery<'a> {
    pub async fn list(self) -> Result<PersonMentionQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        use diesel::dsl::*;

        let person_alias = diesel::alias!(person as person_1);

        let person_id_join = self.person_id.unwrap_or(-1);

        let mut query = person_mentions::table
            .inner_join(comments::table)
            .inner_join(person::table.on(comments::creator_id.eq(person::id)))
            .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person_alias)
            .inner_join(
                comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
            )
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comments::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(comments::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                person_mentions::all_columns,
                comments::all_columns,
                PersonSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                person_alias.fields(PersonSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .into_boxed();

        let mut count_query = person_mentions::table
        .inner_join(comments::table)
        .inner_join(person::table.on(comments::creator_id.eq(person::id)))
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
        .inner_join(person_alias)
        .inner_join(
            comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)),
        )
        .into_boxed();
        
        if let Some(recipient_id) = self.recipient_id {
            query = query.filter(person_mentions::recipient_id.eq(recipient_id));
            count_query = count_query.filter(person_mentions::recipient_id.eq(recipient_id));
        }

        if self.unread_only.unwrap_or(false) {
            query = query.filter(person_mentions::read.eq(false));
            count_query = count_query.filter(person_mentions::read.eq(false));
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
            .load::<PersonMentionViewTuple>(conn)
            .await?;

        let mentions = PersonMentionView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;
        let unread = PersonMentionView::get_unread_mentions(self.pool, person_id_join).await?;

        Ok(PersonMentionQueryResponse { mentions, count, unread })
    }
}

impl ViewToVec for PersonMentionView {
    type DbTuple = PersonMentionViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                person_mention: a.0,
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
