//! Password hashing utilities using Argon2id
//!
//! This module provides secure password hashing and verification
//! using the Argon2id algorithm, which is the recommended variant
//! for password hashing due to its resistance to GPU/ASIC attacks.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using Argon2id with default params.
/// Returns the encoded hash string (PHC format: $argon2id$v=19$...).
///
/// # Example
/// ```
/// use reverie_core::crypto::hash_password;
///
/// let hash = hash_password("my_secure_password").unwrap();
/// assert!(hash.starts_with("$argon2id$"));
/// ```
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::HashError(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against a hash.
/// Returns Ok(true) if password matches, Ok(false) if not.
///
/// # Example
/// ```
/// use reverie_core::crypto::{hash_password, verify_password};
///
/// let password = "my_secure_password";
/// let hash = hash_password(password).unwrap();
/// assert!(verify_password(password, &hash).unwrap());
/// assert!(!verify_password("wrong_password", &hash).unwrap());
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| CryptoError::ParseError(e.to_string()))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Errors that can occur during password hashing operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Failed to hash the password
    #[error("Failed to hash password: {0}")]
    HashError(String),
    /// Failed to parse the hash string
    #[error("Failed to parse hash: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password2").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_format() {
        let hash = hash_password("test").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_invalid_hash() {
        let result = verify_password("password", "invalid_hash");
        assert!(result.is_err());
    }
}
