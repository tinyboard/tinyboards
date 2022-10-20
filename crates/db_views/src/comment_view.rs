// use crate::structs::CommentView;
// use diesel::{dsl::*, result::Error, *};
// use porpl_db::{
//     aggregates::structs::CommentAggregates,
//     schema::{
//         comment,
//         comment_aggregates,
//         comment_like,
//         comment_saved,
//         board,
//         board_block,
//         board_subscriber,
//         board_user_ban,
//         user_,
//         user_block,
//         post,
//     },
//     models::{
//         comment::comment::Comment,
//         comment::comment_saved::CommentSaved,
//         board::board::BoardSafe,
//         board::board_subscriber::BoardSubscriber,
//         board::board_user_ban::BoardUserBan,
//         user::user::UserSafe,
//         user::user_block::UserBlock,
//         post::post::Post,
//     },
//     traits::{ToSafe, ViewToVec},
//     utils::{
//         functions::hot_rank,
//         fuzzy_search,
//         limit_and_offset_unlimited,
//     },
//     CommentSortType,
//     ListingType,
// };
// use typed_builder::TypedBuilder;

// type CommentViewTuple = (
//     Comment,
//     UserSafe,
//     Post,
//     BoardSafe,
//     CommentAggregates,
//     Option<BoardUserBan>,
//     Option<BoardSubscriber>,
//     Option<CommentSaved>,
//     Option<UserBlock>,
//     Option<i16>,
// );

// impl CommentView {
//     pub fn read(
//         conn: &mut PgConnection,
//         comment_id: i32,
//         user_id: Option<i32>,
//     ) -> Result<Self, Error> {
//         let user_id_join = user_id.unwrap_or(-1);

//         let (
//             comment,
//             creator,
//             post,
//             board,
//             counts,
//             creator_banned_from_board,
//             subscriber,
//             saved,
//             creator_blocked,
//             comment_like,
//         ) = comment::table
//             .find(comment_id)
//             .inner_join(user_::table)
//             .inner_join(post::table)
//             .inner_join(board::table.on(post::board_id.eq(board::id)))
//             .inner_join(comment_aggregates::table)
//             .left_join(
//                 board_user_ban::table.on(
//                     board::id
//                         .eq(board_user_ban::board_id)
//                         .and(board_user_ban::user_id.eq(comment::creator_id))
//                         .and(
//                             board_user_ban::expires
//                                 .is_null()
//                                 .or(board_user_ban::expires.gt(now)),
//                     ),
//                 ),
//             )
//             .left_join(
//                 board_subscriber::table.on(
//                     post::board_id
//                         .eq(board_subscriber::board_id)
//                         .and(board_subscriber::user_id.eq(user_id_join)),
//                 ),
//             )
//             .left_join(
//                 comment_saved::table.on(
//                     comment::id
//                         .eq(comment_saved::comment_id)
//                         .and(comment_saved::user_id.eq(user_id_join)),
//                 ),
//             )
//             .left_join(
//                 user_block::table.on(
//                     comment::creator_id
//                         .eq(user_block::target_id)
//                         .and(user_block::user_id.eq(user_id_join)),
//                 ),
//             )
//             .left_join(
//                 comment_like::table.on(
//                     comment::id
//                         .eq(comment_like::comment_id)
//                         .and(comment_like::user_id.eq(user_id_join)),
//                 ),
//             )
//             .select((
//                 comment::all_columns,
//                 UserSafe::safe_columns_tuple(),
//                 post::all_columns,
//                 BoardSafe::safe_columns_tuple(),
//                 comment_aggregates::all_columns,
//                 board_user_ban::all_columns.nullable(),
//                 board_subscriber::all_columns.nullable(),
//                 comment_saved::all_columns.nullable(),
//                 user_block::all_columns.nullable(),
//                 comment_like::all_columns.nullable(),
//             ))
//             .first::<CommentViewTuple>(conn)?;

//             let my_vote = if user_id.is_some() && comment_like.is_none() {
//                 Some(0)
//             } else {
//                 comment_like
//             };

//             Ok(CommentView {
//                 comment,
//                 post,
//                 creator,
//                 board,
//                 counts,
//                 creator_banned_from_board: creator_banned_from_board.is_some(),
//                 subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
//                 saved: saved.is_some(),
//                 creator_blocked: creator_blocked.is_some(),
//                 my_vote,
//             })
//     }
// }