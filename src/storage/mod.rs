use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use crate::encryption::{encrypt_file};
use crate::key_management::generate_key_iv;



pub fn store_file(
    file_data: &[u8],
    node_storage_path: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let key_data = generate_key_iv();
    let encrypted_data = encrypt_file(file_data, &key_data.key, &key_data.iv)?;

    // Dosya yolu oluşturulmadan önce dizinin var olup olmadığını kontrol et
    let dir_path = Path::new(node_storage_path);
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }

    let file_path = format!("{}/{}", node_storage_path, file_name);
    let mut file = File::create(file_path)?;
    file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn can_store_file(node_storage_path: &str, file_size: u64) -> Result<bool, Box<dyn std::error::Error>> {
    let storage_dir = Path::new(node_storage_path);
    
    // Eğer dizin yoksa, yeni bir dosya oluşturulacak
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir)?;
    }

    // Dizin içindeki dosyaların toplam boyutunu hesapla
    let total_used = fs::read_dir(storage_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.metadata())
        .filter_map(Result::ok)
        .map(|metadata| metadata.len())
        .sum::<u64>();

    // Node kapasitesi (örneğin 5MB)
    let max_capacity = 5 * 1024 * 1024; // 5MB

    // Mevcut depolama alanı + yeni dosya boyutunun toplamı kapasiteyi aşmamalı
    Ok(total_used + file_size <= max_capacity)
}

