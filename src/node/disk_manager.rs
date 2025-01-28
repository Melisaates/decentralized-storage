use actix_web::cookie::time::format_description::well_known::iso8601::Config;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use reqwest::header;
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use uuid::Uuid;
use md5;
use sha2::{Sha256, Digest};


// Disk Management Structures
#[derive(Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    pub base_path: String,
    // max_capacity is the maximum amount of space that can be used
    pub max_capacity: u64,  // in bytes
    // reserved_space purpose is to prevent disk full errors
    pub reserved_space: u64, // minimum free space to maintain
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub file_name: String,
    pub node_id: String,
    pub size: u64,
    pub created_at: u64,
    pub checksum: String,

}
#[derive(Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub node_id: String,
    pub available_space: u64,
    pub total_space: u64,
    pub is_healthy: bool,
    pub last_checked: u64,
}

// Disk Manager
pub struct DiskManager {
    pub config: DiskConfig,
    pub files: Arc<RwLock<std::collections::HashMap<String, FileMetadata>>>,
}

impl DiskManager {
    pub async fn new(config: DiskConfig) -> std::io::Result<Self> {
        
        Ok(Self {
            config,
            files: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    pub async fn check_space(&self, required_space: u64) -> bool {
        let available = self.get_available_space().await;
        available >= required_space + self.config.reserved_space
    }

    pub async fn get_available_space(&self) -> u64 {
        let total_used: u64 = self.files.read()
            .await
            .values()
            .map(|meta| meta.size)
            .sum();
        self.config.max_capacity.saturating_sub(total_used)
    }

    // Check if there is enough space to store the file
    pub async fn free_space(&mut self, file_path: &str) -> Result<bool, std::io::Error> {
    
        let metadata = fs::metadata(file_path).await?;
        let file_size = metadata.len();
    
        fs::remove_file(file_path).await?;
        self.config.reserved_space -= file_size;
        self.config.max_capacity += file_size;
    
        println!(
            "File '{}' deleted. New max space: {}, reserved space: {}",
            file_path, self.config.max_capacity, self.config.reserved_space
        );
    
        Ok(true)
    }



    pub fn store_file(
        self,
        encrypted_data_content: &[u8], 
        node_storage_path: &str,
        file_name: &str,
        node_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check for empty parameters
        if node_storage_path.is_empty() || file_name.is_empty() {
            return Err("Storage path or file name cannot be empty.".into());
        }

        if !self.check_space(encrypted_data_content.len() as u64).await {
            return Err("Not enough space to store the file.".into());
        }
        
    
        // Create the directory path if it doesn't exist
        let dir_path = Path::new(node_storage_path);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir_path.display(), e));
        }
    
        println!("Storing file at: {}", dir_path.display());
        // Use Path manipulation for file path
        let file_path = dir_path.join(file_name);
         // Ensure parent directory exists
         if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e));
        }
    
    
        println!("Storing file at*****************: {}", file_path.display());
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


    
        // Store the file hash along with node_id in a registry 
        store_file_hash(file_hash, node_id);
    
        Ok(())
    }
    
 

    pub async fn read_file(&self, file_id: &str) -> std::io::Result<Vec<u8>> {
        let file_path = self.get_file_path(file_id);
        fs::read(&file_path).await
    }

    pub async fn delete_file(&self, file_id: &str) -> std::io::Result<()> {
        let file_path = self.get_file_path(file_id);
        fs::remove_file(&file_path).await?;
        self.files.write().await.remove(file_id);
        Ok(())
    }

    fn get_file_path(&self, file_id: &str) -> PathBuf {
        Path::new(&self.config.base_path).join(file_id)
    }
}

pub fn store_file_hash(file_hash: String, node_id: &str) {
    println!("Storing hash for file on node {}: {}", node_id, file_hash);
}

