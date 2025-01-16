use crate::encryption::encrypt_file;
use crate::key_management::generate_key_iv;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn store_file(
    file_data: &[u8],
    node_storage_path: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let key_data = generate_key_iv();
    let encrypted_data = encrypt_file(file_data, &key_data.key, &key_data.iv)?;

    // Check if the directory exists before creating the file path
    let dir_path = Path::new(node_storage_path);
    if !dir_path.exists() {
        // Create the directory if it does not exist
        fs::create_dir_all(dir_path)?;
    }

    let file_path = format!("{}/{}", node_storage_path, file_name);
    let mut file = File::create(file_path)?;
    file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn can_store_file(
    node_storage_path: &str,
    file_size: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    let storage_dir = Path::new(node_storage_path);

    // If the directory does not exist, a new file will be created
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir)?;
    }

    // Calculate the total size of files in the directorys
    let total_used = fs::read_dir(storage_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.metadata())
        .filter_map(Result::ok)
        .map(|metadata| metadata.len())
        .sum::<u64>();

    // Node capacity (example: 5MB)
    let max_capacity = 5 * 1024 * 1024; // 5MB

    // The total of current storage usage and new file size should not exceed the capacity
    Ok(total_used + file_size <= max_capacity)
}
