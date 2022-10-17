use porpl_db::{models::secret::Secret, utils::DbPool};
use porpl_utils::settings::{structs::Settings, SETTINGS};
use reqwest_middleware::ClientWithMiddleware;


/// The global context for the application
pub struct PorplContext {
    pool: DbPool,
    client: ClientWithMiddleware,
    settings: Settings,
    master_key: Secret,
}

impl PorplContext {
    pub fn create(
        pool: DbPool,
        client: ClientWithMiddleware,
        settings: Settings,
        master_key: Secret,
    ) -> PorplContext {
        PorplContext {
            pool,
            client,
            settings,
            master_key,
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
}

impl Clone for PorplContext {
    fn clone(&self) -> Self {
        PorplContext {
            pool: self.pool.clone(),
            client: self.client.clone(),
            settings: self.settings.clone(),
            master_key: self.master_key.clone(),
        }
    }
}
