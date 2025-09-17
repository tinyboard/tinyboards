use crate::settings::SETTINGS;
use argon2::{self, Config};
use uuid::Uuid;

pub fn hash_password(pwd: String) -> String {
    let settings = SETTINGS.to_owned();
    let environment = settings.environment;
    let mut _salt_uuid = String::new();
    if environment != String::from("dev") {
        // we want a random uuid here per new password
        _salt_uuid = Uuid::new_v4().to_string();
    } else {
        // we want a blank uuid here (so we can make re-usable accounts for dev purposes)
        _salt_uuid = String::from("");
    };
    let salt_suffix = settings.salt_suffix;
    let salt = &[_salt_uuid.as_bytes(), salt_suffix.as_bytes()].concat();
    let config = Config::default();
    let hashed_pwd = argon2::hash_encoded(pwd.as_bytes(), salt, &config).unwrap();

    hashed_pwd
}

pub fn verify_password(encoded: &str, pwd: &str) -> bool {
    let pwd_matched = argon2::verify_encoded(encoded, pwd.as_bytes()).unwrap();
    pwd_matched
}
