use argon2::{self, Config};

pub fn hash_password(pwd: &str) -> String {
    
    let salt = std::env::var("PASS_SALT").expect("ERROR: You need to set PASS_SALT in your .env file!");
    let config = Config::default();
    let hashed_pwd = argon2::hash_encoded(pwd.as_bytes(), salt.as_bytes(), &config).unwrap();

    hashed_pwd
}

pub fn verify_password(encoded: &str, pwd: &str) -> bool {

    let pwd_matched = argon2::verify_encoded(encoded, pwd.as_bytes()).unwrap();
    
    pwd_matched
}