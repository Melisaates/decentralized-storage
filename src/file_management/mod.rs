use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;


pub fn upload_file(file_path: &str, encrypted_file_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    // Dosya şifrele
    encrypt_file(file_path, encrypted_file_path, key, iv)?;
    println!("Dosya şifrelendi ve yüklendi: {}", encrypted_file_path);

    // Dosya hash'ini BSC'ye kaydet
    let file_hash = calculate_file_hash(file_path);
    println!("Dosya hash'ı: {}", file_hash);

    // BSC'ye dosya sahipliğini kaydetme
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        match record_file_ownership(&file_hash).await {
            Ok(_) => println!("Dosya sahipliği başarıyla BSC'ye kaydedildi."),
            Err(e) => eprintln!("Dosya sahipliği kaydetme hatası: {:?}", e),
        }
    });

    Ok(())
}

// Dosya hash hesaplama
fn calculate_file_hash(file_path: &str) -> String {
    // Basitçe dosyanın içeriğinin hash'ini hesapla
    // Gerçek uygulamada SHA256 veya benzeri bir algoritma kullanılabilir
    format!("dummy_hash_of_{}", file_path)
}

pub fn delete_file(file_path: &str) -> std::io::Result<()> {
    if Path::new(file_path).exists() {
        std::fs::remove_file(file_path)?;
        println!("Dosya silindi: {}", file_path);
    } else {
        println!("Dosya bulunamadı: {}", file_path);
    }
    Ok(())
}
