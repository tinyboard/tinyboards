use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Mods {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i32,
    accepted: Bool,
    invite_rescinded: Bool,
    perm_content: Bool,
    perm_appearance: Bool,
    perm_config: Bool,
    perm_access: Bool,
    perm_full: Bool,
}