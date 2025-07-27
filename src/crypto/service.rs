use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use base64::{Engine as _, engine::general_purpose};
use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM},
    rand::{SecureRandom, SystemRandom},
};

use crate::core::config::CryptoConfig;
use crate::core::error::{AppError, AppResult};

pub struct CryptoService {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl CryptoService {
    pub fn new(config: &CryptoConfig) -> Result<Self> {
        // Convert the encryption key from base64 to bytes
        let key_bytes = general_purpose::STANDARD
            .decode(&config.encryption_key)
            .map_err(|e| AppError::Crypto(format!("Invalid encryption key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(AppError::Crypto("Encryption key must be 32 bytes".to_string()).into());
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|e| AppError::Crypto(format!("Failed to create key: {}", e)))?;
        
        let key = LessSafeKey::new(unbound_key);
        let rng = SystemRandom::new();

        Ok(Self { key, rng })
    }

    /// Encrypt plaintext data
    pub fn encrypt(&self, plaintext: &str) -> AppResult<String> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|e| AppError::Crypto(format!("Failed to generate nonce: {}", e)))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();

        self.key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| AppError::Crypto(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to encrypted data
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);

        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypt ciphertext data
    pub fn decrypt(&self, ciphertext: &str) -> AppResult<String> {
        let encrypted_data = general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| AppError::Crypto(format!("Invalid base64: {}", e)))?;

        if encrypted_data.len() < 12 {
            return Err(AppError::Crypto("Ciphertext too short".to_string()));
        }

        let (nonce_bytes, mut ciphertext_with_tag) = encrypted_data.split_at(12);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|e| AppError::Crypto(format!("Invalid nonce: {}", e)))?;

        let mut ciphertext_vec = ciphertext_with_tag.to_vec();
        let plaintext_bytes = self.key.open_in_place(nonce, Aad::empty(), &mut ciphertext_vec)
            .map_err(|e| AppError::Crypto(format!("Decryption failed: {}", e)))?;

        let plaintext = String::from_utf8(plaintext_bytes.to_vec())
            .map_err(|e| AppError::Crypto(format!("Invalid UTF-8: {}", e)))?;

        Ok(plaintext)
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Crypto(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Crypto(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate a random token
    pub fn generate_token(&self) -> AppResult<String> {
        let mut token_bytes = [0u8; 32];
        self.rng.fill(&mut token_bytes)
            .map_err(|e| AppError::Crypto(format!("Failed to generate token: {}", e)))?;

        Ok(general_purpose::STANDARD.encode(token_bytes))
    }

    /// Hash data using SHA-256
    pub fn hash_data(&self, data: &str) -> String {
        use ring::digest;
        let digest = digest::digest(&digest::SHA256, data.as_bytes());
        general_purpose::STANDARD.encode(digest.as_ref())
    }

    /// Generate a secure random ID
    pub fn generate_id(&self) -> AppResult<String> {
        let mut id_bytes = [0u8; 16];
        self.rng.fill(&mut id_bytes)
            .map_err(|e| AppError::Crypto(format!("Failed to generate ID: {}", e)))?;

        Ok(general_purpose::STANDARD.encode(id_bytes))
    }
}