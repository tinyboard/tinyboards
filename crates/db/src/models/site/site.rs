use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Site {
    id: i32,
    name: String,
    description: Option<String>,
    creator_id: i32,
    published: chrono::NaiveDateTime,
    updated: Option<chrono::NaiveDateTime>,
    enable_downvotes: bool,
    open_registration: bool,
    enable_nsfw: bool,
    require_application: bool,
    application_question: Option<String>,
    private_instance: bool,
}