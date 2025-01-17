use pkcs7::encrypted_data_content;

use crate::encryption::encrypt_file;
use crate::key_management::generate_key_iv;
use crate::p2p::Node;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn store_file(
    encrypted_data_content: &[u8],
    node_storage_path: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check for empty parameters
    if node_storage_path.is_empty() || file_name.is_empty() {
        return Err("Storage path or file name cannot be empty.".into());
    }

    // Create the directory path if it doesn't exist
    let dir_path = Path::new(node_storage_path);
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path).map_err(|e| {
            format!(
                "Failed to create directory {}: {}",
                dir_path.display(),
                e
            )
        })?;
    }

    // Use Path manipulation for file path
    let file_path = dir_path.join(file_name);
    let mut file = File::create(&file_path).map_err(|e| {
        format!(
            "Failed to create file {}: {}",
            file_path.display(),
            e
        )
    })?;

    // Write the encrypted data to the file
    file.write_all(encrypted_data_content).map_err(|e| {
        format!(
            "Failed to write data to file {}: {}",
            file_path.display(),
            e
        )
    })?;

    // Ensure the file is fully written to disk
    file.sync_all().map_err(|e| {
        format!(
            "Failed to sync data to file {}: {}",
            file_path.display(),
            e
        )
    })?;

    println!("File successfully stored at {}", file_path.display());

    Ok(())
}


pub async fn can_store_file(
    nodes: &Vec<Node>, // Tüm düğümleri içeren liste
    file_size: u64,    // Dosya boyutu
) -> Option<String> {  // Depolayabileceği düğümün ID'sini döndür
    for node in nodes {
        let storage_dir = Path::new(&node.storage_path);
        println!("Checking storage for node: {}", node.id);

        // Eğer dizin yoksa, oluştur
        if !storage_dir.exists() {
            fs::create_dir_all(storage_dir).ok(); // Hata oluşursa devam et
        }

        // Düğümde kullanılan toplam alanı hesapla
        let total_used = fs::read_dir(storage_dir)
            .unwrap_or_else(|_| fs::read_dir("/dev/null").unwrap()) // Hata oluşursa boş döner
            .filter_map(Result::ok)
            .map(|entry| entry.metadata())
            .filter_map(Result::ok)
            .map(|metadata| metadata.len())
            .sum::<u64>();

        println!(
            "Node ID: {}, Used: {}, Available: {}, File Size: {}",
            node.id, total_used, node.available_space, file_size
        );

        // Dosya depolanabiliyorsa düğüm ID'sini döndür
        if total_used + file_size <= node.available_space {
            return Some(node.id.clone());
        }
    }
    None // Uygun düğüm bulunamadı
}
