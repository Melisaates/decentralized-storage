use aes::{Aes128, BlockEncrypt, BlockDecrypt, NewBlockCipher};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use hex::{encode, decode};

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// Anahtar ve IV üretimi
fn generate_key_iv() -> ([u8; 16], [u8; 16]) {
    let mut rng = rand::thread_rng();
    let key: [u8; 16] = rng.gen();
    let iv: [u8; 16] = rng.gen();
    (key, iv)
}

// Dosyayı şifrele
fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut data = Vec::new();
    input_file.read_to_end(&mut data)?;

    let cipher = Aes128Cbc::new_from_slices(key, iv).unwrap();
    let encrypted_data = cipher.encrypt_vec(&data);

    // HMAC ile dosya bütünlüğünü kontrol et
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(&encrypted_data);
    let hmac_result = hmac.finalize().into_bytes();

    // Hem şifreli veriyi hem de HMAC değerini sakla
    let mut output_file = OpenOptions::new().create(true).write(true).open(output_path)?;
    output_file.write_all(&encrypted_data)?;
    output_file.write_all(&hmac_result)?;

    Ok(())
}