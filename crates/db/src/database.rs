use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenvy::dotenv;
use r2d2::Pool;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

fn estabilish_connection() -> PgPool {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be specified");
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub fn connect() -> Self {
        Self {
            pool: estabilish_connection(),
        }
    }
}
