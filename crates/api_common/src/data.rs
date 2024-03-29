use tinyboards_db::{models::secret::Secret, utils::DbPool};
use tinyboards_utils::{settings::{structs::Settings, SETTINGS}, rate_limit::RateLimitCell};
use reqwest_middleware::ClientWithMiddleware;

/// The global context for the application
pub struct TinyBoardsContext {
    pool: DbPool,
    client: ClientWithMiddleware,
    settings: Settings,
    master_key: Secret,
    rate_limit_cell: RateLimitCell,
}

impl TinyBoardsContext {
    pub fn create(
        pool: DbPool,
        client: ClientWithMiddleware,
        settings: Settings,
        master_key: Secret,
        rate_limit_cell: RateLimitCell,
    ) -> TinyBoardsContext {
        TinyBoardsContext {
            pool,
            client,
            settings,
            master_key,
            rate_limit_cell,
        }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub fn client(&self) -> &ClientWithMiddleware {
        &self.client
    }

    pub fn settings(&self) -> &'static Settings {
        &SETTINGS
    }

    pub fn master_key(&self) -> &Secret {
        &self.master_key
    }

    pub fn rate_limit_cell(&self) -> &RateLimitCell {
        &&self.rate_limit_cell
    }
}

impl Clone for TinyBoardsContext {
    fn clone(&self) -> Self {
        TinyBoardsContext {
            pool: self.pool.clone(),
            client: self.client.clone(),
            settings: self.settings.clone(),
            master_key: self.master_key.clone(),
            rate_limit_cell: self.rate_limit_cell.clone(),
        }
    }
}
