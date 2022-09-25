use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Boards {
    id: i32,
    board_name: String,
    created_utc: i32,
    board_description: Nullable<String>,
    board_description_html: Nullable<String>,
    over_18: Bool,
    is_nsfl: Bool,
    is_banned: Bool,
    has_banner: Bool,
    has_profile: Bool,
    creator_id: i32,
    ban_reason: Nullable<String>,
    color: String,
    restricted_posting: Bool,
    disallowbots: Bool,
    hide_banner_data: Bool,
    profile_nonce: i32,
    banner_nonce: i32,
    is_private: Bool,
    color_nonce: i32,
    rank_trending: BigDecimal,
    stored_subscriber_count: i32,
    all_opt_out: Bool,
    is_locked_category: Bool,
    subcat_id: i32,
    secondary_color: String



}




diesel::table! {
    boards (id) {


        rank_trending -> Numeric,
        stored_subscriber_count -> Int4,
        all_opt_out -> Bool,
        is_locked_category -> Bool,
        subcat_id -> Int4,
        secondary_color -> Nullable<Varchar>,
        public_chat -> Bool,
        motd -> Varchar,
        css_nonce -> Int4,
        css -> Varchar,
    }
}
