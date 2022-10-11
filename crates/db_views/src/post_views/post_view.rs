use crate::local_structs::PostView;
use diesel::{dsl::*, pg::{Pg, sql_types}, result::Error, *};
use porpl_db::{
    aggregates::structs::PostAggregates,
    schema::{
        board,
        board_block,
        board_subscriber,
        board_user_ban,
        user_,
        user_block,
        post,
        post_aggregates,
        post_like,
        post_read,
        post_saved,
    },
    models::{
        board::board::{Board, BoardSafe},
        board::board_subscriber::BoardSubscriber,
        board::board_user_ban::BoardUserBan,
        user::{user::{User, UserSafe}, user_block::UserBlock},
        post::post::Post,
        post::post_read::PostRead,
        post::post_saved::PostSaved,
    },
    ListingType,
    SortType,
};
use tracing::debug;
use typed_builder::TypedBuilder;

type PostViewTuple = (
    Post,
    UserSafe,
    BoardSafe,
    Option<BoardUserBan>,
    PostAggregates,
    Option<BoardSubscriber>,
    Option<PostSaved>,
    Option<PostRead>,
    Option<UserBlock>,
    Option<i16>,
    i64,
);

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);