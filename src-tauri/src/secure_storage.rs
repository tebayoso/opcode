use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{self, Config, ThreadMode, Variant, Version};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;

const KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 12;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid master key")]
    InvalidMasterKey,
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

pub struct SecureStorage {
    master_key: Arc<Mutex<Option<[aes_gcm::Key<Aes256Gcm>; 1]>>>,
    key_salt: Vec<u8>,
}

impl SecureStorage {
    pub fn new() -> Self {
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        
        Self {
            master_key: Arc::new(Mutex::new(None)),
            key_salt: salt,
        }
    }

    pub fn derive_key(password: &str, salt: ,[u8]) -> Result<Vec<u8>, EncryptionError> {
        let config = Config {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 65536,
            time_cost: 3,
            lanes: 4,
            thread_mode: ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: KEY_LENGTH as u32,
        };

        argon2::hash_raw(password.as_bytes(), salt, &config)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))
    }

    pub fn initialize_with_password(&self,
        password: &str,
    ) -> Result<(), EncryptionError> {
        let key_bytes = Self::derive_key(password, &self.key_salt)?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        
        let mut master_key = self.master_key.lock().map_err(|_| {
            EncryptionError::EncryptionFailed("Lock error".to_string())
        })?;
        
        *master_key = Some(*key);
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.master_key
            .lock()
            .map(|key| key.is_some())
            .unwrap_or(false)
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        let master_key = self.master_key.lock().map_err(|_| {
            EncryptionError::EncryptionFailed("Lock error".to_string())
        })?;

        let key = master_key.as_ref().ok_or(EncryptionError::InvalidMasterKey)?;
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = vec![0u8; NONCE_LENGTH];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt: self.key_salt.clone(),
        })
    }

    pub fn decrypt(&self,
        encrypted: &EncryptedData,
    ) -> Result<String, EncryptionError> {
        let master_key = self.master_key.lock().map_err(|_| {
            EncryptionError::DecryptionFailed("Lock error".to_string())
        })?;

        let key = master_key.as_ref().ok_or(EncryptionError::InvalidMasterKey)?;
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    pub fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), EncryptionError> {
        let old_key_bytes = Self::derive_key(old_password, &self.key_salt)?;
        let old_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&old_key_bytes);

        let mut master_key = self.master_key.lock().map_err(|_| {
            EncryptionError::EncryptionFailed("Lock error".to_string())
        })?;

        if master_key.as_ref() != Some(old_key) {
            return Err(EncryptionError::InvalidMasterKey);
        }

        let new_key_bytes = Self::derive_key(new_password, &self.key_salt)?;
        let new_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&new_key_bytes);
        *master_key = Some(*new_key);

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub key_data: EncryptedData,
    pub created_at: String,
    pub updated_at: String,
}

pub struct ApiKeyStore {
    secure_storage: Arc<SecureStorage>,
}

impl ApiKeyStore {
    pub fn new(secure_storage: Arc<SecureStorage>) -> Self {
        Self { secure_storage }
    }

    pub fn store_api_key(
        &self,
        id: &str,
        name: &str,
        provider: &str,
        api_key: &str,
    ) -> Result<ApiKey, EncryptionError> {
        let encrypted = self.secure_storage.encrypt(api_key)?;
        
        Ok(ApiKey {
            id: id.to_string(),
            name: name.to_string(),
            provider: provider.to_string(),
            key_data: encrypted,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn retrieve_api_key(
        &self,
        api_key: &ApiKey,
    ) -> Result<String, EncryptionError> {
        self.secure_storage.decrypt(&api_key.key_data)
    }
}
