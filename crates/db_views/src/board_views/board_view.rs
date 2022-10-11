use crate::actor_structs::{BoardModeratorView, BoardView, UserViewSafe};
use diesel::{result::Error, *};
use porpl_db::{
    aggregates::structs::BoardAggregates,
    schema::{
        board,
        board_aggregates,
        board_block,
        board_subscriber,
        user_,
    },
    models::board::{
        board::{Board, BoardSafe},
        board_subscriber::BoardSubscriber,
        board_block::BoardBlock,
    },
    traits::{ToSafe, ViewToVec},
};

type BoardViewTuple = (
    BoardSafe,
    BoardAggregates,
    Option<BoardSubscriber>,
    Option<BoardBlock>,
);

impl BoardView {
    pub fn read(
        conn: &mut PgConnection,
        board_id: i32,
        user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = user_id.unwrap_or(-1);

        let (board, counts, subscriber, blocked) = board::table
            .find(board_id)
            .inner_join(board_aggregates::table)
            .left_join(
                board_subscriber::table.on(
                    board::id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                board_block::table.on(
                    board::id
                        .eq(board_block::board_id)
                        .and(board_block::user_id.eq(user_id_join)),
                ),
            )
            .select((
                Board::safe_columns_tuple(),
                board_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                board_block::all_columns.nullable(),
            ))
            .first::<BoardViewTuple>(conn)?;
        Ok(BoardView {
            board,
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber), 
            blocked: blocked.is_some(),
            counts,
        })
    }
}