use std::fs::{self, File};
use std::io::{Read, Write};

use crate::encryption::{encrypt_file_,decrypt_file_};
use crate::key_management::generate_key_iv;



/// Dosyayı şifrelenmiş şekilde depola
pub fn store_file(
    file_data: &[u8],
    node_storage_path: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let key_data = generate_key_iv();
    
    let encrypted_data = encrypt_file_(file_data ,&key_data.key,&key_data.iv)?;

    // Dosyayı node'un depolama dizinine kaydet
    let file_path = format!("{}/{}", node_storage_path, file_name);
    let mut file = File::create(file_path)?;
    file.write_all(&encrypted_data)?;

    Ok(())
}

/// Dosyayı indir ve şifresini çöz
pub fn retrieve_file(
    file_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut encrypted_file = File::open(file_path)?;
    let mut encrypted_data = Vec::new();
    encrypted_file.read_to_end(&mut encrypted_data)?;

    let decrypted_data = decrypt_file_(&encrypted_data, &generate_key_iv().key, &generate_key_iv().iv)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(())
}
