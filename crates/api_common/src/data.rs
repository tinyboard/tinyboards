use porpl_db::{database::PgPool, Database};
use std::env;

/// The global context for the application
pub struct PorplContext {
    db: Database,
    master_key: String,
}

impl PorplContext {
    pub fn init() -> Self {
        Self {
            db: Database::connect(),
            master_key: env::var("MASTER_KEY")
                .expect("The environment variable MASTER_KEY must be specified!"),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.db.pool
    }

    pub fn master_key(&self) -> &str {
        &self.master_key
    }
}
