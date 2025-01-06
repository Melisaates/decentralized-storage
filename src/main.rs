use hex::{encode};
mod encryption;
use encryption::{generate_key_iv, encrypt_file, decrypt_file};
    
fn main() -> std::io::Result<()> {
    let (key, iv) = generate_key_iv();

    // Anahtar ve IV'yi Hex formatında göster
    let key_hex = encode(&key);
    let iv_hex = encode(&iv);
    println!("Key (hex): {}", key_hex);
    println!("IV (hex): {}", iv_hex);

    let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-12-08 at 13.57.41_8ca1a6fc.mp4";
    let encrypted_file_path = "C:/Users/melisates/Documents/encrypted.doc";  
    let decrypted_file_path = "C:/Users/melisates/Documents/decrypted.mp4";

    
    // Dosya şifrele
    encrypt_file(file_path, encrypted_file_path, &key, &iv)?;
    println!("Dosya şifrelendi: {}", encrypted_file_path);

    // Dosya şifresini çöz
    decrypt_file(encrypted_file_path, decrypted_file_path, &key, &iv)?;
    println!("Dosya çözüldü: {}", decrypted_file_path);

    Ok(())
}
