use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use thiserror::Error;

#[allow(dead_code)]
pub trait SecretCipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8]) -> Result<String, SecretCipherError>;
    fn decrypt(&self, ciphertext: &str) -> Result<Vec<u8>, SecretCipherError>;
}

#[allow(dead_code)]
pub struct AesGcmSecretCipher {
    cipher: Aes256Gcm,
}

#[allow(dead_code)]
impl AesGcmSecretCipher {
    pub fn from_hex(key: &str) -> Result<Self, SecretCipherError> {
        let bytes = hex::decode(key).map_err(|_| SecretCipherError::InvalidKey)?;
        if bytes.len() != 32 {
            return Err(SecretCipherError::InvalidKey);
        }
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&bytes);
        Ok(Self {
            cipher: Aes256Gcm::new(key),
        })
    }
}

#[allow(dead_code)]
impl SecretCipher for AesGcmSecretCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<String, SecretCipherError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let encrypted = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| SecretCipherError::EncryptionFailed)?;
        let mut encoded = nonce_bytes.to_vec();
        encoded.extend(encrypted);
        Ok(hex::encode(encoded))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<Vec<u8>, SecretCipherError> {
        let encoded = hex::decode(ciphertext).map_err(|_| SecretCipherError::InvalidCiphertext)?;
        if encoded.len() < 12 {
            return Err(SecretCipherError::InvalidCiphertext);
        }
        let (nonce, encrypted) = encoded.split_at(12);
        self.cipher
            .decrypt(Nonce::from_slice(nonce), encrypted)
            .map_err(|_| SecretCipherError::DecryptionFailed)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SecretCipherError {
    #[error("secret encryption key must be 32 bytes encoded as hexadecimal")]
    InvalidKey,
    #[error("secret encryption failed")]
    EncryptionFailed,
    #[error("ciphertext is invalid")]
    InvalidCiphertext,
    #[error("ciphertext authentication failed")]
    DecryptionFailed,
}

#[cfg(test)]
mod tests {
    use super::{AesGcmSecretCipher, SecretCipher, SecretCipherError};

    const KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn encrypts_and_decrypts_without_reusing_ciphertext() {
        let cipher = AesGcmSecretCipher::from_hex(KEY).unwrap();
        let first = cipher.encrypt(b"contact-secret").unwrap();
        let second = cipher.encrypt(b"contact-secret").unwrap();
        assert_ne!(first, second);
        assert_eq!(cipher.decrypt(&first).unwrap(), b"contact-secret");
    }

    #[test]
    fn rejects_tampering_and_wrong_keys() {
        let cipher = AesGcmSecretCipher::from_hex(KEY).unwrap();
        let mut encrypted = cipher.encrypt(b"contact-secret").unwrap();
        encrypted.push('0');
        assert!(matches!(
            cipher.decrypt(&encrypted),
            Err(SecretCipherError::InvalidCiphertext | SecretCipherError::DecryptionFailed)
        ));
        let other = AesGcmSecretCipher::from_hex(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap();
        let original = cipher.encrypt(b"contact-secret").unwrap();
        assert_eq!(
            other.decrypt(&original),
            Err(SecretCipherError::DecryptionFailed)
        );
    }

    #[test]
    fn validates_key_format() {
        assert!(matches!(
            AesGcmSecretCipher::from_hex("00"),
            Err(SecretCipherError::InvalidKey)
        ));
        assert!(matches!(
            AesGcmSecretCipher::from_hex("not-hex"),
            Err(SecretCipherError::InvalidKey)
        ));
    }
}
