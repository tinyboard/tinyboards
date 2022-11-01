// use crate::structs::UserMentionView;
// use diesel::{dsl::*, result::Error, *};
// use tinyboards_db::{
//     aggregates::structs::CommentAggregates,
//     schema::{
//         comment,
//         comment_aggregates,
//         comment_vote,
//         comment_saved,
//         board,
//         board_subscriber,
//         board_user_ban,
//         user_,
//         user_block,
//         user_mention,
//         post,
//     },
//     models::{
//         comment::comment::Comment,
//         comment::comment_saved::CommentSaved,
//         board::board::BoardSafe,
//         board::{board_user_ban::BoardUserBan, board_block},
//         board::board_subscriber::BoardSubscriber,
//         user::user::UserSafe,
//         user::user_block::UserBlock,
//         user::user_mention::UserMention,
//         post::post::Post,
//     },
//     traits::{ToSafe, ViewToVec},
//     utils::{functions::hot_rank, limit_and_offset},
//     CommentSortType,
// };
// use typed_builder::TypedBuilder;


// type UserMentionViewTuple = (
//     UserMention,
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

// impl UserMentionView {
//     pub fn read(
//         conn: &mut PgConnection,
//         user_mention_id: i32,
//         user_id: Option<i32>,
//     ) -> Result<Self, Error> {
//         let user_alias = diesel::alias!(user_ as user_1);

//         let user_id_join = user_id.unwrap_or(-1);

//         let (
//             user_mention,
//             comment,
//             creator,
//             post,
//             board,
//             recipient,
//             counts,
//             creator_banned_from_board,
//             subscribed,
//             saved,
//             creator_blocked,
//             my_vote,
//         ) = user_mention::table
//             .find(user_mention_id)
//             .inner_join(comment::table)
//             .inner_join(user_::table.on(comment::creator_id.eq(user_::id)))
//             .inner_join(post::table.on(comment::post_id.eq(post::id)))
//             .inner_join(board::table.on(post::board_id.eq(board::id)))
//             .inner_join(user_alias)
//             .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
//             .left_join(
//                 board_user_ban::table.on(
//                     board::id
//                         .eq(board_user_ban::board_id)
//                         .and(board_user_ban::user_id.eq(comment::creator_id))
//                         .and(
//                             board_user_ban::expires.is_null()
//                             .or(
//                                 board_user_ban::expires.gt(now)
//                         ),
//                     ),
//                 ),
//             )
//             .left_join(
//                 board_subscriber::table.on(
//                     post::board_id
//                         .eq(board_subscriber::board_id)
//                         .and(board_subscriber::user_id.eq(user_id_join)
//                     ),
//                 ),
//             )
//             .left_join(
//                 comment_saved::table.on(
//                     comment::id
//                         .eq(comment_saved::comment_id)
//                         .and(comment_saved::user_id.eq(user_id_join)
//                     ),
//                 ),
//             )
//             .left_join(
//                 user_block::table.on(
//                     comment::creator_id
//                         .eq(user_block::target_id)
//                         .and(user_block::user_id.eq(user_id_join)
//                     ),
//                 ),
//             )
//             .left_join(
//                 comment_vote::table.on(
//                     comment::id
//                         .eq(comment_vote::comment_id)
//                         .and(comment_vote::user_id.eq(user_id_join)
//                     ),
//                 ),
//             )
//             .select((
//                 user_mention::all_columns,
//                 comment::all_columns,
//                 UserSafe::safe_columns_tuple(),
//                 post::all_columns,
//                 BoardSafe::safe_columns_tuple(),
//                 user_alias.fields(UserSafe::safe_columns_tuple()),
//                 comment_aggregates::all_columns,
//                 board_user_ban::all_columns.nullable(),
//                 board_subscriber::all_columns.nullable(),
//                 comment_saved::all_columns.nullable(),
//                 user_block::all_columns.nullable(),
//                 comment_vote::all_columns.nullable(),
//             ))
//             .first::<UserMentionViewTuple>(conn)?;
        
//         Ok( UserMentionView {
//             user_mention,
//             comment,
//             creator,
//             post,
//             board,
//             recipient,
//             counts,
//             creator_banned_from_board: creator_banned_from_board.is_some(),
//             subscribed: BoardSubscriber::to_subscribed_type(&subscribed),
//             saved: saved.is_some(),
//             creator_blocked: creator_blocked.is_some(),
//             my_vote,
//         })
//     }
// }