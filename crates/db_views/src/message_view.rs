use tinyboards_db::{
    models::{
        board::{
            board_person_bans::BoardPersonBan,
            boards::{Board, BoardSafe},
        },
        message::message::Message,
        person::{
            person::{Person, PersonSafe},
            person_blocks::PersonBlock,
        },
    },
    schema::{board_person_bans, boards, person, person_blocks, private_message},
    traits::ToSafe,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use diesel::{
    dsl::now, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    PgExpressionMethods, QueryDsl,
};

use diesel_async::RunQueryDsl;

use crate::structs::MessageView;

type MessageViewTuple = (
    Message,
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

        let (message, creator, recipient_user, recipient_board, banned, blocked) =
            private_message::table
                .find(message_id)
                .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
                .left_join(
                    person_alias.on(private_message::recipient_board_id
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
            creator,
            recipient_user,
            recipient_board,
            creator_banned_from_board: banned.is_some(),
            creator_blocked: blocked.is_some(),
        })
    }
}
