use crate::p2p::Node;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use sha2::{Sha256, Digest};

pub fn store_file(
    encrypted_data_content: &[u8],
    node_storage_path: &str,
    file_name: &str,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check for empty parameters
    if node_storage_path.is_empty() || file_name.is_empty() {
        return Err("Storage path or file name cannot be empty.".into());
    }

    // Create the directory path if it doesn't exist
    let dir_path = Path::new(node_storage_path);
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create directory {}: {}", dir_path.display(), e))?;
    }

    // Use Path manipulation for file path
    let file_path = dir_path.join(file_name);
    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file {}: {}", file_path.display(), e))?;

    // Write the encrypted data to the file
    file.write_all(encrypted_data_content).map_err(|e| {
        format!(
            "Failed to write data to file {}: {}",
            file_path.display(),
            e
        )
    })?;

    // Ensure the file is fully written to disk
    file.sync_all()
        .map_err(|e| format!("Failed to sync data to file {}: {}", file_path.display(), e))?;

    // Generate a hash for the file for integrity check
    let mut hasher = Sha256::new();
    hasher.update(encrypted_data_content);
    let file_hash = hex::encode(hasher.finalize());

    println!("File successfully stored at {}, hash: {}", file_path.display(), file_hash);

    // Store the file hash along with node_id in a registry or database (optional)
    store_file_hash(file_hash, node_id);

    Ok(())
}

pub fn store_file_hash(file_hash: String, node_id: &str) {
    // In a real decentralized system, the hash and node_id should be stored in a distributed registry
    println!("Storing hash for file on node {}: {}", node_id, file_hash);
}

pub async fn can_store_file(
    nodes: &Vec<Node>, // List of all nodes
    file_size: u64,    // Size of the file
) -> Option<String> {
    // Return the ID of the node that can store the file
    for node in nodes {
        let storage_dir = Path::new(&node.storage_path);
        println!("Checking storage for node: {}", node.id);

        // If the directory doesn't exist, create it
        if !storage_dir.exists() {
            fs::create_dir_all(storage_dir).ok(); // Continue if there's an error
        }

        // Calculate the total used space in the node
        let total_used = fs::read_dir(storage_dir)
            .unwrap_or_else(|_| fs::read_dir("/dev/null").unwrap()) // Return empty if there's an error
            .filter_map(Result::ok)
            .map(|entry| entry.metadata())
            .filter_map(Result::ok)
            .map(|metadata| metadata.len())
            .sum::<u64>();

        println!(
            "Node ID: {}, Used: {}, Available: {}, File Size: {}",
            node.id, total_used, node.available_space, file_size
        );

        // Return the node ID if it can store the file
        if total_used + file_size <= node.available_space {
            return Some(node.id.clone());
        }
    }
    None // No suitable node found
}
