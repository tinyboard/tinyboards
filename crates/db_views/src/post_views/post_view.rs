use crate::local_structs::PostView;
use diesel::{dsl::*, /*pg::{Pg, sql_types::*},*/ result::Error, *};
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
    traits::{ToSafe, ViewToVec},
    // ListingType,
    // SortType,
};
//use tracing::debug;
//use typed_builder::TypedBuilder;

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
);

// sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

impl PostView {
    pub fn read(
        conn: &mut PgConnection,
        post_id: i32,
        my_user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = my_user_id.unwrap_or(-1);
        let (
            post,
            creator,
            board,
            creator_banned_from_board,
            counts,
            subscriber,
            saved,
            read,
            creator_blocked,
            post_like,
        ) = post::table
            .find(post_id)
            .inner_join(user_::table)
            .inner_join(board::table)
            .left_join(
                board_user_ban::table.on(
                    post::board_id
                        .eq(board_user_ban::board_id)
                        .and(board_user_ban::user_id.eq(post::creator_id))
                        .and(
                            board_user_ban::expires
                                .is_null()
                                .or(board_user_ban::expires.gt(now))
                        ),
                    ),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(
                    post::board_id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join))
                )
            )
            .left_join(
                post_saved::table.on(
                    post::id
                        .eq(post_saved::post_id)
                        .and(post_saved::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_read::table.on(
                    post::id
                        .eq(post_read::post_id)
                        .and(post_read::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                user_block::table.on(
                    post::creator_id
                        .eq(user_block::target_id)
                        .and(user_block::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_like::table.on(
                    post::id
                        .eq(post_like::post_id)
                        .and(post_like::user_id.eq(user_id_join)),
                ),
            )
            .select((
                post::all_columns,
                User::safe_columns_tuple(),
                Board::safe_columns_tuple(),
                board_user_ban::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                user_block::all_columns.nullable(),
                post_like::score.nullable(),
            ))
            .first::<PostViewTuple>(conn)?;

            let my_vote = if my_user_id.is_some() && post_like.is_none() {
                Some(0)
            } else {
                post_like
            };

            Ok(PostView {
                post,
                creator,
                board,
                creator_banned_from_board: creator_banned_from_board.is_some(),
                counts,
                subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
                saved: saved.is_some(),
                read: read.is_some(),
                creator_blocked: creator_blocked.is_some(),
                my_vote,
            })
    }
}