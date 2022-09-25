pub mod database;
pub mod models;

mod impls;
mod schema;

pub use database::Database;

pub use models::alts::*;
pub use models::badge_defs::*;