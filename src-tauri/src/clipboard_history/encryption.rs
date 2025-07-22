use crate::error::AppError;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};

const KEYRING_SERVICE: &str = "dev.byteatatime.flare";
const KEYRING_USERNAME: &str = "clipboard_history_key";

fn get_encryption_key_impl(entry: &keyring::Entry) -> Result<[u8; 32], AppError> {
    match entry.get_password() {
        Ok(hex_key) => {
            let key_bytes =
                hex::decode(hex_key).map_err(|e| AppError::ClipboardHistory(e.to_string()))?;
            Ok(key_bytes.try_into().unwrap())
        }
        Err(keyring::Error::NoEntry) => {
            let new_key: [u8; 32] = rand::random();
            let hex_key = hex::encode(new_key);
            entry.set_password(&hex_key)?;
            Ok(new_key)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn get_encryption_key() -> Result<[u8; 32], AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;
    get_encryption_key_impl(&entry)
}

pub fn encrypt(data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| AppError::ClipboardHistory(e.to_string()))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(hex::encode(result))
}

pub fn decrypt(hex_data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    let data = hex::decode(hex_data).map_err(|e| AppError::ClipboardHistory(e.to_string()))?;
    if data.len() < 12 {
        return Err(AppError::ClipboardHistory("Invalid encrypted data".into()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key.into());
    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::ClipboardHistory(e.to_string()))?;
    String::from_utf8(decrypted_bytes).map_err(|e| AppError::ClipboardHistory(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring::Entry;

    fn get_random_key() -> [u8; 32] {
        rand::random()
    }

    #[test]
    fn get_encryption_key_creates_new_key_if_none_exists() {
        // Given: a mock keyring entry that is empty
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).unwrap();
        let _ = entry.delete_credential();

        // When: we get the encryption key using the implementation function
        let key1 = get_encryption_key_impl(&entry).unwrap();

        // Then: A key is generated and returned
        assert_eq!(key1.len(), 32);

        // And: The key is stored in the keyring (verified on the same entry object)
        let stored_hex = entry.get_password().unwrap();
        assert_eq!(hex::decode(stored_hex).unwrap(), key1);
    }

    #[test]
    fn get_encryption_key_retrieves_existing_key() {
        // Given: a mock keyring entry with a pre-existing key
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).unwrap();
        let existing_key = get_random_key();
        entry.set_password(&hex::encode(existing_key)).unwrap();

        // When: We get the encryption key using the same entry object
        let retrieved_key = get_encryption_key_impl(&entry).unwrap();

        // Then: The retrieved key matches the existing key
        assert_eq!(retrieved_key, existing_key);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_works_for_simple_string() {
        // Given: a key and a plaintext string
        let key = get_random_key();
        let plaintext = "Hello, secure world!";

        // When: we encrypt the plaintext
        let encrypted = encrypt(plaintext, &key).unwrap();
        // And: we decrypt the ciphertext
        let decrypted = decrypt(&encrypted, &key).unwrap();

        // Then: the decrypted text matches the original plaintext
        assert_ne!(encrypted, plaintext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_works_for_empty_string() {
        // Given: a key and an empty plaintext string
        let key = get_random_key();
        let plaintext = "";

        // When: we encrypt and then decrypt
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        // Then: the result is the original empty string
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_works_for_unicode_string() {
        // Given: a key and a Unicode plaintext string
        let key = get_random_key();
        let plaintext = "你好, 世界! 🌍✨";

        // When: we encrypt and then decrypt
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        // Then: the decrypted text matches the original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        // Given: a plaintext encrypted with key1
        let key1 = get_random_key();
        let key2 = get_random_key();
        let plaintext = "some secret data";
        let encrypted = encrypt(plaintext, &key1).unwrap();

        // When: we try to decrypt with key2
        let result = decrypt(&encrypted, &key2);

        // Then: the decryption fails
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_with_tampered_data() {
        // Given: an encrypted string
        let key = get_random_key();
        let plaintext = "some secret data";
        let mut encrypted = encrypt(plaintext, &key).unwrap();
        let len = encrypted.len();

        // When: we tamper with the ciphertext (flip a bit in the last byte)
        let last_char_val = u8::from_str_radix(&encrypted[len - 1..], 16).unwrap();
        let new_last_char = format!("{:x}", last_char_val ^ 0x1);
        encrypted.replace_range(len - 1.., &new_last_char);

        let result = decrypt(&encrypted, &key);

        // Then: the decryption fails
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_with_invalid_hex() {
        // Given: a key and a non-hex string
        let key = get_random_key();
        let invalid_data = "this is not hex";

        // When: we try to decrypt
        let result = decrypt(invalid_data, &key);

        // Then: the decryption fails
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ClipboardHistory(_)));
    }

    #[test]
    fn decrypt_fails_with_data_too_short() {
        // Given: a key and data shorter than the nonce
        let key = get_random_key();
        let short_data = "123456";

        // When: we try to decrypt
        let result = decrypt(short_data, &key);

        // Then: the decryption fails with a specific error
        assert!(result.is_err());
        let err_string = result.unwrap_err().to_string();
        assert!(err_string.contains("Invalid encrypted data"));
    }
}
