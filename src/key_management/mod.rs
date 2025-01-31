use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use ethers::core::k256::elliptic_curve::rand_core::le;
use rand::Rng;
use serde::{Serialize, Deserialize};
use aes::{Aes256};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;

// Define AES-256 CBC type
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

// Define the key store type
type KeyStore = HashMap<String, Vec<u8>>; // file_id -> encrypted_key mapping

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

// Define the file path as a constant
const KEY_FILE_PATH: &str = "keys/key_data.json";
/*WINDOWS
$FilePath = "keys\key_data.json"
$Acl = Get-Acl $FilePath
$Acl.SetAccessRuleProtection($true, $false)  # Tüm kalıtımları devre dışı bırak
$Rule = New-Object System.Security.AccessControl.FileSystemAccessRule("Melisa", "FullControl", "Allow")
$Acl.SetAccessRule($Rule)
Set-Acl $FilePath $Acl
 */
/*LINUX
chmod 600 keys/key_data.json */
use std::env;

fn get_master_key() -> [u8; 32] {
    let key_str = env::var("MASTER_KEY").expect("MASTER_KEY is not set!");
    let key_bytes = key_str.as_bytes();
    
    let mut master_key = [0u8; 32];
    let len = key_bytes.len().min(32); // Eğer anahtar 32 bayttan küçükse, sadece o kısmı kullan
    master_key[..len].copy_from_slice(&key_bytes[..len]);

    master_key
}

// Load the key store from the JSON file
pub fn load_key_store() -> io::Result<KeyStore> {
    match File::open(KEY_FILE_PATH) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()), // If file doesn't exist, return an empty HashMap
        Err(e) => Err(e),
    }
}

// Save the key store to the JSON file
pub fn save_key_store(key_store: &KeyStore) -> io::Result<()> {
    let file = File::create(KEY_FILE_PATH)?;
    serde_json::to_writer(file, key_store).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// Generate a new key and IV
pub fn generate_key_iv() -> KeyData {
    let mut rng = rand::thread_rng();
    let key: [u8; 16] = rng.gen();
    let iv: [u8; 16] = rng.gen();
    KeyData { key, iv }
}

// Encrypt the key data
pub fn encrypt_key_data(key_data: &KeyData) -> Vec<u8> {
    let  master_key = get_master_key();
    let cipher = Aes256Cbc::new_from_slices(& master_key, &key_data.iv).unwrap();
    let encrypted_key = cipher.encrypt_vec(&key_data.key);

    let mut result = Vec::new();
    result.extend_from_slice(&key_data.iv); // Prepend IV to encrypted data
    result.extend_from_slice(&encrypted_key);
    result
}

// Decrypt the key data
pub fn decrypt_key_data(encrypted_key: &[u8]) -> KeyData {
    let iv: &[u8; 16] = encrypted_key[0..16].try_into().expect("Invalid IV length");
    let encrypted_key_data = &encrypted_key[16..];

    let  master_key = get_master_key();
    let cipher = Aes256Cbc::new_from_slices(& master_key, iv).expect("Cipher initialization failed");
    let decrypted_key = cipher.decrypt_vec(encrypted_key_data).expect("Decryption failed");

    if decrypted_key.len() != 16 {
        panic!("Decrypted key has an unexpected size: {}", decrypted_key.len());
    }

    KeyData {
        key: decrypted_key.try_into().expect("Failed to convert decrypted key to KeyData"),
        iv: *iv,
    }
}

// Save the encrypted key to the key store
pub fn save_encrypted_key_to_store(key_data: &KeyData, file_id: &str) -> io::Result<()> {
    let encrypted_key = encrypt_key_data(key_data);
    println!("Key encrypted. Saving to key store...");
    let mut key_store = load_key_store()?;

    if key_store.contains_key(file_id) {
        println!("Key already exists for file ID: '{}'. Skipping save.", file_id);
        return Ok(());
    }

    key_store.insert(file_id.to_string(), encrypted_key);
    println!("Key saved for file ID: '{}'.", file_id);

    save_key_store(&key_store)
}

// Load and decrypt the key from the key store
pub fn load_and_decrypt_key(file_id: &str) -> io::Result<KeyData> {
    let key_store = load_key_store()?;
    
    if let Some(encrypted_key) = key_store.get(file_id) {
        println!("Key found for file ID: '{}'. Decrypting...", file_id);
        let key_data = decrypt_key_data(encrypted_key);
        Ok(key_data)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("File ID '{}' not found", file_id)))
    }
}
