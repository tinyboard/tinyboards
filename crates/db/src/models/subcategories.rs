use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Subcategories {
    id: i32,
    cat_id: i32,
    subcat_name: String,
    subcat_description: String,
    _visible: bool,
}
