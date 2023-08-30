use tinyboards_db::{
    models::{
        board::{board_person_bans::BoardPersonBan, boards::BoardSafe},
        message::message::{Message, MessageNotif},
        person::{person::PersonSafe, person_blocks::PersonBlock},
    },
    schema::{board_person_bans, boards, person, person_blocks, pm_notif, private_message},
    traits::{ToSafe, ViewToVec},
    utils::{get_conn, limit_and_offset, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use diesel::{
    dsl::{count, now},
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};

use diesel_async::RunQueryDsl;
use typed_builder::TypedBuilder;

use crate::structs::MessageView;

type MessageViewTuple = (
    Message,
    Option<MessageNotif>,
    PersonSafe,
    Option<PersonSafe>,
    Option<BoardSafe>,
    Option<BoardPersonBan>,
    Option<PersonBlock>,
);

impl MessageView {
    pub async fn read(
        pool: &DbPool,
        message_id: i32,
        person_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let person_alias = diesel::alias!(person as person_alias);

        let (message, notif, creator, recipient_user, recipient_board, banned, blocked) =
            private_message::table
                .find(message_id)
                .left_join(
                    pm_notif::table.on(private_message::id
                        .eq(pm_notif::pm_id)
                        .and(pm_notif::recipient_id.eq(person_id))),
                )
                .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
                .left_join(
                    person_alias.on(private_message::recipient_user_id
                        .eq(person_alias.fields(person::id.nullable()))),
                )
                .left_join(
                    boards::table.on(boards::id
                        .nullable()
                        .eq(private_message::recipient_board_id)),
                )
                .left_join(
                    board_person_bans::table.on(boards::id
                        .eq(board_person_bans::board_id)
                        .and(board_person_bans::person_id.eq(private_message::creator_id))
                        .and(
                            board_person_bans::expires
                                .is_null()
                                .or(board_person_bans::expires.gt(now)),
                        )),
                )
                .left_join(
                    person_blocks::table.on(private_message::creator_id
                        .eq(person_blocks::target_id)
                        .and(person_blocks::person_id.eq(person_id))),
                )
                .select((
                    private_message::all_columns,
                    pm_notif::all_columns.nullable(),
                    PersonSafe::safe_columns_tuple(),
                    person_alias.fields(PersonSafe::safe_columns_tuple().nullable()),
                    BoardSafe::safe_columns_tuple().nullable(),
                    board_person_bans::all_columns.nullable(),
                    person_blocks::all_columns.nullable(),
                ))
                .first::<MessageViewTuple>(conn)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Something went wrong while loading message view",
                    )
                })?;

        Ok(Self {
            message,
            notif,
            creator,
            recipient_user,
            recipient_board,
            creator_banned_from_board: banned.is_some(),
            creator_blocked: blocked.is_some(),
        })
    }

    pub async fn get_unread_count(pool: &DbPool, person_id: i32) -> Result<i64, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        pm_notif::table
            .filter(
                pm_notif::recipient_id
                    .eq(person_id)
                    .and(pm_notif::read.eq(false)),
            )
            .select(count(pm_notif::id))
            .first::<i64>(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to get unread message count")
            })
    }

    /// Marks all unread messages as read for a user
    pub async fn mark_all_messages_as_read(
        pool: &DbPool,
        person_id: i32,
    ) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(pm_notif::table)
            .filter(pm_notif::read.eq(false))
            .filter(pm_notif::recipient_id.eq(person_id))
            .set(pm_notif::read.eq(true))
            .execute(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to mark messages as read")
            })
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct MessageQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    #[builder(!default)]
    person_id: i32,
    board_id: Option<i32>,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct MessageQueryResponse {
    pub messages: Vec<MessageView>,
    pub count: i64,
    pub unread: i64,
}

impl<'a> MessageQuery<'a> {
    pub async fn list(self) -> Result<MessageQueryResponse, TinyBoardsError> {
        let conn = &mut get_conn(self.pool).await?;

        let person_alias = diesel::alias!(person as person_alias);

        let person_id_join = self.person_id;

        let mut query = private_message::table
            .left_join(
                pm_notif::table.on(private_message::id
                    .eq(pm_notif::pm_id)
                    .and(pm_notif::recipient_id.eq(person_id_join))),
            )
            .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
            .left_join(person_alias.on(
                private_message::recipient_user_id.eq(person_alias.fields(person::id.nullable())),
            ))
            .left_join(
                boards::table.on(boards::id
                    .nullable()
                    .eq(private_message::recipient_board_id)),
            )
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(private_message::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                person_blocks::table.on(private_message::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .select((
                private_message::all_columns,
                pm_notif::all_columns.nullable(),
                PersonSafe::safe_columns_tuple(),
                person_alias.fields(PersonSafe::safe_columns_tuple().nullable()),
                BoardSafe::safe_columns_tuple().nullable(),
                board_person_bans::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
            ))
            .into_boxed();

        let mut count_query = private_message::table
            .left_join(
                pm_notif::table.on(private_message::id
                    .eq(pm_notif::pm_id)
                    .and(pm_notif::recipient_id.eq(person_id_join))),
            )
            .left_join(person_alias.on(
                private_message::recipient_user_id.eq(person_alias.fields(person::id.nullable())),
            ))
            .left_join(
                boards::table.on(boards::id
                    .nullable()
                    .eq(private_message::recipient_board_id)),
            )
            .into_boxed();

        if let Some(board_id) = self.board_id {
            query = query.filter(private_message::recipient_board_id.eq(board_id));
            count_query = count_query.filter(private_message::recipient_board_id.eq(board_id));
        } else {
            query = query.filter(private_message::recipient_user_id.eq(person_id_join));
            count_query = count_query.filter(private_message::recipient_user_id.eq(person_id_join));
        }

        if let Some(unread_only) = self.unread_only {
            if unread_only {
                query = query.filter(pm_notif::read.eq(false));
                count_query = count_query.filter(pm_notif::read.eq(false));
            }
        }

        query = query.then_order_by(private_message::published.desc());

        let (limit, offset) = limit_and_offset(self.page, self.limit)
            .map_err(|e| TinyBoardsError::from_error_message(e, 400, "Invalid limit or page"))?;

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<MessageViewTuple>(conn)
            .await?;

        let messages = MessageView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;
        let unread = MessageView::get_unread_count(self.pool, person_id_join).await?;

        Ok(MessageQueryResponse {
            messages,
            count,
            unread,
        })
    }
}

impl ViewToVec for MessageView {
    type DbTuple = MessageViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                message: a.0,
                notif: a.1,
                creator: a.2,
                recipient_user: a.3,
                recipient_board: a.4,
                creator_banned_from_board: a.5.is_some(),
                creator_blocked: a.6.is_some(),
            })
            .collect::<Vec<Self>>()
    }
}
