use std::fs::DirBuilder;
use std::path::Path;
pub mod disk_manager;

use serde::{Deserialize, Serialize};
// use crate::file_system::file_operations::{change_file_permission, control_permission};
use crate::file_system::file_operations;
use crate::file_system::FileSystem;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::thread::available_parallelism;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, anyhow};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub node_id: String,
    pub available_space: u64,
    pub total_space: u64,
    pub is_healthy: bool,
    pub last_checked: u64,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
// address : IP address of the node
//storage_path : Path to the storage directory
pub struct StorageNode {
    pub node_id: String, // unique identifier
    pub storage_path: String, // path to the storage directory
    pub address: String,
    pub total_space: u64,
    pub available_space: u64,
    health_status: HealthStatus
}

impl StorageNode {
    // Initialize storage file for the node
    pub async fn initialize_storage_file(&self) -> Result<()> {
        let path = Path::new(&self.storage_path);

        if path.exists() {
            println!("Storage file already exists for node {} at {}", self.node_id, self.storage_path);
            return Ok(());
        }

        // Depolama alanını kontrol et
        if !disk_manager.check_space(0).await {
            return Err(anyhow!(
                "Not enough space available to initialize storage for node {}",
                self.node_id
            ));
        }
        

        // Check write permission
        if !file_operations::control_permission(path.to_str().unwrap()) {
            // Attempt to change permissions if needed
            println!("Attempting to change write permission...");
            file_operations::change_file_permission(path.to_str().unwrap())?;
            if !file_operations::control_permission(path.to_str().unwrap()) {
                return Err(anyhow!("No write permission for storage path: {}", self.storage_path));
            }
        }

        // let mut file = fs::File::create(&self.storage_path)?;
        // file.set_len(self.total_space)?;

        // println!(
        //     "Storage file created for node {}: {} ({} bytes)",
        //     self.node_id, self.storage_path, self.total_space
        // );

        // Create a directory with the node id name and set its capacity
        DirBuilder::new().recursive(true).create(&self.storage_path)?;
        let mut dir_file = fs::File::create(format!("{}/capacity.txt", self.storage_path))?;
        writeln!(dir_file, "Capacity: {}", self.total_space)?;
        Ok(())
    }

    pub fn new(node_id: String, total_space: u64) -> Result<Self> {

        // Validate capacity for the file system
        let file_system = FileSystem::detect_file_system().ok_or_else(|| anyhow!("Failed to detect file system"))?;
        if Self::validate_capacity(file_system, total_space).is_err() {
            return Err(anyhow!("Invalid capacity for the selected file system."));
        }

        let storage_path = format!("storage/{}", node_id);
        let node = StorageNode {
            node_id,
            storage_path,
            total_space,
            available_space: total_space,
            address: String::from(" "),
        };

        node.initialize_storage_file().await?;

        Ok(node)
    }

    // Validate capacity for the file system
    pub fn validate_capacity(file_system: FileSystem, capacity: u64) -> Result<()> {
        let max_size = file_system.max_file_size();
        if capacity as u128 > max_size {
            return Err(anyhow!(
                "The specified capacity ({}) exceeds the maximum allowed file size ({}) for the selected file system.",
                capacity, max_size
            ));
        }
        Ok(())
    }




    // Reduce available space after data is stored
    pub fn reduce_available_space(&mut self, file_size: u64) {
        if self.available_space >= file_size {
            self.available_space -= file_size;
            println!(
                "StorageNode {}: Space reduced. Remaining available space: {}",
                self.node_id, self.available_space
            );
        } else {
            println!(
                "StorageNode {}: Not enough space to store the file of size {}.",
                self.node_id, file_size
            );
        }
    }

    pub fn store_data(&mut self, data: &[u8]) -> Result<()> {
        if self.available_space < data.len() as u64 {
            return Err(anyhow!("Not enough available space to store data."));
        }
    
        let path = Path::new(&self.storage_path);
        if !path.exists() {
            // Dizin yoksa oluştur
            DirBuilder::new().recursive(true).create(&self.storage_path)?;
        }
    
        if path.is_dir() {
            // Dizine dosya kaydet
            let file_path = format!("{}/file_{}.dat", self.storage_path, uuid::Uuid::new_v4());
            fs::write(file_path, data)?;
        } else {
            // Dosyaya veri ekle
            let mut file = fs::OpenOptions::new().append(true).open(&self.storage_path)?;
            file.write_all(data)?;
        }
    
        self.available_space -= data.len() as u64;
        Ok(())
    }

    pub async fn free_up_space(&mut self, file_path: &str) -> Result<()> {
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
    
        fs::remove_file(file_path)?;
        self.available_space += file_size;
    
        println!(
            "StorageNode {}: File '{}' deleted. New available space: {}",
            self.node_id, file_path, self.available_space
        );
    
        Ok(())
    }
}






