use aes::{Aes256, BlockEncrypt, NewBlockCipher};
use aes::cipher::{BlockDecrypt, KeyInit};
use rand::Rng;
use hex::{encode, decode};
use serde::{Serialize, Deserialize};
use std::fs::{File, remove_file};
use std::io::{Write, Read};

const AES_BLOCK_SIZE: usize = 16;

// Şifreleme anahtarı yapısı
#[derive(Serialize, Deserialize)]
pub struct EncryptionKey {
    pub key: String,
    pub iv: String,
}

// AES-256 şifreleme fonksiyonu
pub fn encrypt_file(file_path: &str, encryption_key: &EncryptionKey) -> Result<String, Box<dyn std::error::Error>> {
    // Şifreleme için AES anahtarı ve IV (Initialization Vector) alınıyor
    let key = decode(&encryption_key.key)?;
    let iv = decode(&encryption_key.iv)?;
    
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // AES şifrelemesi başlatılıyor
    let cipher = Aes256::new(&key.into());
    let mut buffer = data.clone();
    for chunk in buffer.chunks_mut(AES_BLOCK_SIZE) {
        cipher.encrypt_block(&mut chunk.try_into().unwrap());
    }

    // Şifrelenmiş veriyi Base64 veya Hex formatında döndür
    Ok(encode(&buffer))
}

// Anahtar üretme fonksiyonu
pub fn generate_key() -> EncryptionKey {
    let mut rng = rand::thread_rng();
    let key: Vec<u8> = (0..32).map(|_| rng.gen()).collect(); // 32 byte uzunluğunda AES-256 anahtarı
    let iv: Vec<u8> = (0..16).map(|_| rng.gen()).collect(); // 16 byte IV

    EncryptionKey {
        key: encode(key),
        iv: encode(iv),
    }
}
