use aes_gcm::aead::{Aead, KeyInit, OsRng, generic_array::GenericArray};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

pub const KEY_SIZE: usize = 32; // 256 bits for AES-256
pub const NONCE_SIZE: usize = 12; // Recommended size for AES-GCM

pub fn generate_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn encrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data.as_ref())?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    let (nonce, ciphertext) = data.split_at(NONCE_SIZE);
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
}
