use argon2::{self, Config};
use uuid::Uuid;
use crate::settings::SETTINGS;

pub fn hash_password(pwd: String) -> String {
    let settings = SETTINGS.to_owned();
    let salt_uuid = Uuid::new_v4();
    let salt_suffix = settings.salt_suffix;
    let salt = &[salt_uuid.as_bytes(), salt_suffix.as_bytes()].concat();
    let config = Config::default();
    let hashed_pwd = argon2::hash_encoded(pwd.as_bytes(), salt, &config).unwrap();

    hashed_pwd
}

pub fn verify_password(encoded: &str, pwd: &str) -> bool {
    let pwd_matched = argon2::verify_encoded(encoded, pwd.as_bytes()).unwrap();
    pwd_matched
}


#[test]
fn most_secure_password() {
    temp_env::with_var("SALT_SUFFIX", Some("somesaltsuffix"),|| {

        let most_secure_hash = hash_password(String::from("hunter2"));
        let most_secure_verification = verify_password(&most_secure_hash, "hunter2");
        let not_most_secure_verification = verify_password(&most_secure_hash, "hunter3");
        assert_eq!(most_secure_verification, true);
        assert_eq!(not_most_secure_verification, false);
    })
}
