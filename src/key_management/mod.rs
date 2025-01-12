use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

pub fn generate_key_iv() -> KeyData {
    let mut rng = rand::thread_rng();
    let key: [u8; 16] = rng.gen();
    let iv: [u8; 16] = rng.gen();
    KeyData { key, iv }
}

pub fn save_key_locally(file_path: &str, key_data: &KeyData) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    let data = serde_json::to_string(key_data).unwrap();
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn load_key_locally(file_path: &str) -> std::io::Result<KeyData> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let key_data: KeyData = serde_json::from_str(&data).unwrap();
    Ok(key_data)
} 


// use std::collections::HashMap;
// use rand::Rng;

// #[derive(Clone)]
// pub struct KeyData {
//     pub key: [u8; 16],
//     pub iv: [u8; 16],
// }

// pub struct KeyManager {
//     pub(crate) keys: HashMap<String, KeyData>, // Dosya ID'leri ve anahtarları
// }

// impl KeyManager {
    
//     pub fn generate_key_iv() -> KeyData {
//         let mut rng = rand::thread_rng();
//         let key: [u8; 16] = rng.gen();
//         let iv: [u8; 16] = rng.gen();
//         KeyData { key, iv }
//     }

//     pub fn store_key(&mut self, file_id: String, key_data: KeyData) {
//         self.keys.insert(file_id, key_data);
//     }

//     pub fn get_key(&self, file_id: &str) -> Option<&KeyData> {
//         self.keys.get(file_id)
//     }
// }



