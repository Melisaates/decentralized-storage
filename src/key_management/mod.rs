use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use rand::Rng;
use serde::{Serialize, Deserialize};
use aes::{Aes256, BlockCipher};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use sha2::{Sha256, Digest}; // For key derivation

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

// Key derivation function (KDF) for encryption
// Derive a 32-byte key from a password
pub fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    let mut derived_key = [0u8; 32];
    derived_key.copy_from_slice(&result);
    derived_key
}

// Generate key and IV for encryption
pub fn generate_key_iv() -> KeyData {
    let mut rng = rand::thread_rng();
    let key: [u8; 16] = rng.gen();
    let iv: [u8; 16] = rng.gen();
    KeyData { key, iv }
}

// Encrypt the key
// The key data is encrypted using AES-256 in CBC mode
// Key encrypted with the encryption key and IV
pub fn encrypt_key_data(key_data: &KeyData, encryption_key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256Cbc::new_from_slices(encryption_key, &key_data.iv).unwrap();
    cipher.encrypt_vec(&key_data.key)
}

// Decrypt the encrypted key
pub fn decrypt_key_data(encrypted_key: &[u8], encryption_key: &[u8; 32], iv: &[u8; 16]) -> KeyData {
    let cipher = Aes256Cbc::new_from_slices(encryption_key, iv).unwrap();
    let decrypted_key = cipher.decrypt_vec(encrypted_key).unwrap();
    KeyData {
        key: decrypted_key.try_into().unwrap(),
        iv: iv.to_vec().try_into().unwrap(),
    }
}

// Save the encrypted key to a file
pub fn save_encrypted_key(file_path: &str, encrypted_key: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(encrypted_key)?;
    Ok(())
}

// Load the encrypted key from a file
pub fn load_encrypted_key(file_path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut encrypted_key = Vec::new();
    file.read_to_end(&mut encrypted_key)?;
    Ok(encrypted_key)
}

// Decrypt the encrypted key and retrieve the file content
pub fn load_and_decrypt_key(file_path: &str, password: &str) -> std::io::Result<KeyData> {
    let encrypted_key = load_encrypted_key(file_path)?;
    let encryption_key = derive_key(password);
    let iv: &[u8; 16] = encrypted_key[0..16].try_into().expect("slice with incorrect length");
    let key_data = decrypt_key_data(&encrypted_key[16..], &encryption_key, iv);
    Ok(key_data)
}

// To store the key locally
pub fn save_key_locally(file_path: &str, key_data: &KeyData, password: &str) -> std::io::Result<()> {
    let encryption_key = derive_key(password);
    let encrypted_key = encrypt_key_data(key_data, &encryption_key);
    save_encrypted_key(file_path, &encrypted_key)
}
