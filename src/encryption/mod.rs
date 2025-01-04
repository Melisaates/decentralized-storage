use aes::Aes128;
use cipher::{BlockDecrypt, BlockEncrypt};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use generic_array::GenericArray;
use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use hex::{encode, decode};

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// Anahtar ve IV üretimi
pub fn generate_key_iv() -> ([u8; 16], [u8; 16]) {
    let mut key = [0u8; 16];
    let mut iv = [0u8; 16];
    let mut rng = rand::thread_rng();

    rng.fill(&mut key); 
    rng.fill(&mut iv);   
    
    (key, iv)
}

// Dosya şifreleme
pub fn encrypt_file(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let ciphertext = cipher.encrypt_vec(&data);

    // HMAC hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(&ciphertext);
    let hmac_result = hmac.finalize().into_bytes();
    
    // Şifrelenmiş veri ve HMAC dosyaya yazılır
    let mut output_file = OpenOptions::new().create(true).write(true).open(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&hmac_result)?;

    Ok(())
}

// Dosya şifre çözme
pub fn decrypt_file(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data too short to contain HMAC"));
    }

    // Son 32 bayt HMAC için ayrılır
    let data_len = encrypted_data.len() - 32;
    let hmac_received = &encrypted_data[data_len..];
    let encrypted_data = &encrypted_data[..data_len];

    // HMAC doğrulaması
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(&encrypted_data);
    let hmac_result = hmac.finalize().into_bytes();

    if hmac_result.as_slice() != hmac_received {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    // Şifre çözme
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let decrypted_data = cipher.decrypt_vec(encrypted_data).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
    })?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(())
}
