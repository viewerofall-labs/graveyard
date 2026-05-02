use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use anyhow::Result;

pub fn encrypt(plaintext: &[u8]) -> Result<([u8; 12], Vec<u8>, [u8; 32])> {
    let key = Aes256Gcm::generate_key(OsRng);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let cipher = Aes256Gcm::new(&key);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| anyhow::anyhow!("AES-GCM encrypt failed"))?;

    Ok((nonce.into(), ciphertext, key.into()))
}

pub fn decrypt(nonce: &[u8; 12], ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let nonce = Nonce::from_slice(nonce);
    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("AES-GCM decrypt failed — wrong key or corrupted file"))
}
