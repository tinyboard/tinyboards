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
        board
    }
};