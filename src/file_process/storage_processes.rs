use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use sha2::{Sha256, Digest};



/// Dosyayı şifrelenmiş şekilde depolama
pub fn store_file(file_path: &str, storage_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;

    let encrypted_data = crate::encryption::encrypt(&file_contents);

    fs::create_dir_all(storage_path)?;
    let encrypted_file_path = format!("{}/encrypted_{}", storage_path, file_path);
    let mut encrypted_file = File::create(&encrypted_file_path)?;
    encrypted_file.write_all(&encrypted_data)?;

    Ok(encrypted_file_path)
}

/// Dosya hash hesaplama
pub fn calculate_file_hash(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    Ok(format!("{:x}", hasher.finalize()))
}

/// Dosyayı indirip şifresini çözme
pub fn download_file(
    encrypted_file_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Şifrelenmiş dosyayı oku
    let mut encrypted_file = File::open(encrypted_file_path)?;
    let mut encrypted_data = Vec::new();
    encrypted_file.read_to_end(&mut encrypted_data)?;

    // Şifreyi çöz
    let decrypted_data = crate::encryption::decrypt(&encrypted_data);

    // Şifresi çözülmüş dosyayı kaydet
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    println!("Dosya başarıyla indirildi ve şifresi çözüldü: {}", output_path);
    Ok(())
}

/// Dosyayı silme
pub fn delete_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(file_path).exists() {
        fs::remove_file(file_path)?;
        println!("Dosya başarıyla silindi: {}", file_path);
    } else {
        println!("Dosya bulunamadı: {}", file_path);
    }
    Ok(())
}
