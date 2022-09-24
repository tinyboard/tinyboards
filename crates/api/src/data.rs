use porpl_db::{database::PgPool, Database};

/// The global context for the application
pub struct PorplContext {
    pub db: Database,
}

impl PorplContext {
    pub fn init() -> Self {
        Self {
            db: Database::connect(),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.db.pool
    }
}
