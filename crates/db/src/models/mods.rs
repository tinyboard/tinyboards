use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Mods {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i32,
    accepted: bool,
    invite_rescinded: bool,
    perm_content: bool,
    perm_appearance: bool,
    perm_config: bool,
    perm_access: bool,
    perm_full: bool,
}
