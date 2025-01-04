
use hex::{encode, decode};
mod encryption;
use encryption::{generate_key_iv, encrypt_file, decrypt_file};
    
fn main() -> std::io::Result<()> {
    let (key, iv) = generate_key_iv();

    //Store keys in hex format
    let key_hex = encode(&key);
    let iv_hex = encode(&iv);
    print!("Key (hex): {}\n", key_hex);
    print!("IV (hex): {}\n", iv_hex);

    let file_path = "C:/Users/melisates/Downloads/hostes_sevval.doc";
    let encrypted_file_path = "encrypted.txt";  
    let decrypted_file_path = "decrypted.txt";

    // Encrypt file
    encrypt_file(file_path, encrypted_file_path, &key, &iv)?;

    // Decrypt file
    decrypt_file(encrypted_file_path, decrypted_file_path, &key, &iv)?;

    Ok(())
}



