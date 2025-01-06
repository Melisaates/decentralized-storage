use std::collections::HashMap;
use rand::Rng;

#[derive(Clone)]
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

pub struct KeyManager {
    keys: HashMap<String, KeyData>, // Dosya ID'leri ve anahtarları
}

impl KeyManager {
    // Yeni bir anahtar ve IV üret
    pub fn generate_key_iv() -> KeyData {
        let mut rng = rand::thread_rng();
        let key: [u8; 16] = rng.gen();
        let iv: [u8; 16] = rng.gen();
        KeyData { key, iv }
    }

    // Anahtarları sakla
    pub fn store_key(&mut self, file_id: String, key_data: KeyData) {
        self.keys.insert(file_id, key_data);
    }

    // Anahtarı al
    pub fn get_key(&self, file_id: &str) -> Option<&KeyData> {
        self.keys.get(file_id)
    }
}
