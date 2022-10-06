use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    id: i32,
    name: String,
    fedi_name: String,
    preferred_name: Option<String>,
    passhash: String,
    email: Option<String>,
    admin: bool,
    banned: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    theme: String,
    default_sort_type: i16,
    default_listing_type: i16,
    show_nsfw: bool,
}

