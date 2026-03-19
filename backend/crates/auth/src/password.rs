use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::errors::AuthError;

/// Hash a password using Argon2id with a cryptographically random salt.
///
/// Returns the hash in PHC string format, which embeds the salt and parameters.
/// No configurable salt, no environment checks — random salt every time.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id with safe default params
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashingFailed(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored Argon2 hash (PHC format string).
///
/// Returns Ok(true) if the password matches, Ok(false) if not.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AuthError::HashingFailed(format!("Invalid hash format: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Validate password length requirements.
pub fn validate_password_length(password: &str) -> Result<(), AuthError> {
    if !(10..=60).contains(&password.len()) {
        return Err(AuthError::InvalidPasswordLength);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("hashing should succeed");

        // Hash should be in PHC format (starts with $argon2)
        assert!(hash.starts_with("$argon2"), "Hash should be PHC format");

        // Verify correct password
        assert!(
            verify_password(password, &hash).expect("verify should succeed"),
            "Correct password should verify"
        );

        // Verify wrong password
        assert!(
            !verify_password("wrong_password_123", &hash).expect("verify should succeed"),
            "Wrong password should not verify"
        );
    }

    #[test]
    fn test_unique_salts() {
        let password = "same_password_123";
        let hash1 = hash_password(password).expect("hashing should succeed");
        let hash2 = hash_password(password).expect("hashing should succeed");

        // Two hashes of the same password should be different (random salts)
        assert_ne!(hash1, hash2, "Hashes should differ due to random salts");

        // Both should verify
        assert!(verify_password(password, &hash1).expect("verify should succeed"));
        assert!(verify_password(password, &hash2).expect("verify should succeed"));
    }

    #[test]
    fn test_password_length_validation() {
        assert!(validate_password_length("short").is_err());
        assert!(validate_password_length("123456789").is_err()); // 9 chars
        assert!(validate_password_length("1234567890").is_ok()); // 10 chars
        assert!(validate_password_length(&"x".repeat(60)).is_ok()); // 60 chars
        assert!(validate_password_length(&"x".repeat(61)).is_err()); // 61 chars
    }
}
