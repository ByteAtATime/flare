use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use keyring::Entry;
use std::sync::OnceLock;

const SERVICE_NAME: &str = "flare";
const KEY_NAME: &str = "encryption-key";
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;

static CIPHER: OnceLock<Result<XChaCha20Poly1305, String>> = OnceLock::new();

fn get_or_create_key() -> Result<[u8; KEY_SIZE], String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME).map_err(|e| e.to_string())?;

    match entry.get_password() {
        Ok(stored) => {
            let bytes = hex::decode(&stored).map_err(|e| e.to_string())?;
            bytes
                .try_into()
                .map_err(|_| "Invalid key length in keyring".to_string())
        }
        Err(keyring::Error::NoEntry) => {
            let key = XChaCha20Poly1305::generate_key(&mut OsRng);
            let key_hex = hex::encode(&key);
            entry.set_password(&key_hex).map_err(|e| e.to_string())?;
            Ok(key.into())
        }
        Err(e) => Err(e.to_string()),
    }
}

fn get_cipher() -> Result<&'static XChaCha20Poly1305, String> {
    let result =
        CIPHER.get_or_init(|| get_or_create_key().map(|key| XChaCha20Poly1305::new(&key.into())));
    result.as_ref().map_err(|e| e.clone())
}

pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let cipher = get_cipher()?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let cipher = get_cipher()?;
    let combined = hex::decode(encrypted).map_err(|e| e.to_string())?;

    if combined.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = XNonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
