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

// Key and IV production
fn generate_key_iv() -> ([u8; 16], [u8; 16]) {
    let mut key = [0u8; 16];
    let mut iv = [0u8; 16];
    let mut rng = rand::thread_rng();

    key = rng.gen();
    iv = rng.gen();
    
    (key, iv)
}

// Encrypt file
fn encrypt_file(file_path: &str,output_path: &str ,key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    let mut file = File::open(file_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    let cipher = Aes128Cbc::new_from_slice(key, iv).unwrap();
    let ciphertext = cipher.encrypt_vec(&data);


    //Check file integrity with HMAC
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(&ciphertext);   
    let hmac_result = hmac.finalize().into_bytes();
    
    // Store both encrypted data and HMAC value 

    let mut output_file = OpenOptions::new().create(true).Write(true).open(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&hmac_result)?;

    Ok(())
}

