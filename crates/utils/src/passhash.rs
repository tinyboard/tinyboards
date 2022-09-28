use argon2::{self, Config};
use uuid::Uuid;

pub fn hash_password(pwd: String) -> String {
    let salt_uuid = Uuid::new_v4();
    let salt_suffix = std::env::var("SALT_SUFFIX")
        .expect("ERROR: You need to set SALT_SUFFIX in your .env file!");
    let salt = &[salt_uuid.as_bytes(), salt_suffix.as_bytes()].concat();
    let config = Config::default();
    let hashed_pwd = argon2::hash_encoded(pwd.as_bytes(), salt, &config).unwrap();

    hashed_pwd
}

pub fn verify_password(encoded: &str, pwd: &str) -> bool {
    let pwd_matched = argon2::verify_encoded(encoded, pwd.as_bytes()).unwrap();
    pwd_matched
}
