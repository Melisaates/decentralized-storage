use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use rand::Rng;
use serde::{Serialize, Deserialize};
use aes::{Aes256};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use sha2::{Sha256, Digest}; // For key derivation

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

type KeyStore = HashMap<String, Vec<u8>>; // file_id -> encrypted_key eşlemesi

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

// Define the file path as a constant
const KEY_FILE_PATH: &str = "keys/key_data.json";  // Update this path as needed

// Load the key store from the JSON file
pub fn load_key_store() -> io::Result<KeyStore> {
    match File::open(KEY_FILE_PATH) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()), // Dosya yoksa boş bir HashMap döner
        Err(e) => Err(e),
    }
}

// Save the key store to the JSON file
pub fn save_key_store(key_store: &KeyStore) -> io::Result<()> {
    let file = File::create(KEY_FILE_PATH)?;
    serde_json::to_writer(file, key_store).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// Key derivation function (KDF) for encryption
// Derive a 32-byte key from a password
pub fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    let mut derived_key = [0u8; 32];
    derived_key.copy_from_slice(&result[..]);
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
    let encrypted_key = cipher.encrypt_vec(&key_data.key);

    let mut result = Vec::new();
    result.extend_from_slice(&key_data.iv); // Prepend IV to encrypted data
    result.extend_from_slice(&encrypted_key);
    result
}

// Decrypt the key
pub fn decrypt_key_data(encrypted_key: &[u8], encryption_key: &[u8; 32]) -> KeyData {
    let iv: &[u8; 16] = encrypted_key[0..16].try_into().expect("Invalid IV length");
    let encrypted_key_data = &encrypted_key[16..];

    let cipher = Aes256Cbc::new_from_slices(encryption_key, iv).expect("Cipher initialization failed");
    let decrypted_key = cipher.decrypt_vec(encrypted_key_data).expect("Decryption failed");

    if decrypted_key.len() != 16 {
        panic!("Decrypted key has an unexpected size: {}", decrypted_key.len());
    }

    KeyData {
        key: decrypted_key.try_into().expect("Failed to convert decrypted key to KeyData"),
        iv: iv.clone(),
    }
}
// Save the encrypted key to the key store (HashMap)
pub fn save_encrypted_key_to_store(key_data: &KeyData, password: &str, file_id: &str) -> io::Result<()> {
    let encryption_key = derive_key(password);
    let encrypted_key: Vec<u8> = encrypt_key_data(key_data, &encryption_key);

    let mut key_store = load_key_store()?;

    // Check if the file_id already exists in the key store
    if let Some(existing_key) = key_store.get(file_id) {

        println!(" {:?} ",key_data);
        println!("Key already exists for file ID: '{}'. Skipping save.", file_id);
        return Ok(()); // If the key already exists, do nothing
    }

    // If the key does not exist, insert the new encrypted key
    key_store.insert(file_id.to_string(), encrypted_key);
    println!("Key saved for file ID: '{}'", file_id);
    println!(" {:?} ",key_store.get(file_id));

    // Save the updated key store to the file
    save_key_store(&key_store)
}

// Load and decrypt the key from the key store (HashMap)
pub fn load_and_decrypt_key(password: &str, file_id: &str) -> io::Result<KeyData> {
    let key_store = load_key_store()?;
    
    // Check if the file_id exists in the key store
    if let Some(encrypted_key) = key_store.get(file_id) {
        println!("Key found for file ID: '{}'. Decrypting key...", file_id);
        let encryption_key = derive_key(password);
        let key_data = decrypt_key_data(encrypted_key, &encryption_key);
        Ok(key_data)
    } else {
        // If the key is not found, return an error
        Err(io::Error::new(io::ErrorKind::NotFound, format!("File ID '{}' not found", file_id)))
    }
}
