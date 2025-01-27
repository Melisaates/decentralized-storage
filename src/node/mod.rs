use std::path::Path;

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize, Debug)]
// address : IP address of the node
//storage_path : Path to the storage directory
pub struct Node {
    pub id: String, // unique identifier
    pub storage_path: String,
    pub address: String,
    pub total_space: u64,
    pub available_space: u64,
}

impl Node {
    // Initialize storage file for the node
    pub fn initialize_storage_file(&self) -> Result<()> {
        let path = Path::new(&self.storage_path);

        if path.exists() {
            println!("Storage file already exists for node {} at {}", self.id, self.storage_path);
            return Ok(());
        }

        // Check write permission
        if !Self::control_permission(&self) {
            // Attempt to change permissions if needed
            println!("Attempting to change write permission...");
            self.change_file_permission()?;
            if !Self::control_permission(&self) {
                return Err(anyhow!("No write permission for storage path: {}", self.storage_path));
            }
        }

        let mut file = fs::File::create(&self.storage_path)?;
        file.set_len(self.total_space)?;

        println!(
            "Storage file created for node {}: {} ({} bytes)",
            self.id, self.storage_path, self.total_space
        );

        Ok(())
    }

    pub fn new(id: String, storage_path: String, total_space: u64) -> Result<Self> {

        // Validate capacity for the file system
        let file_system = FileSystem::detect_file_system().ok_or_else(|| anyhow!("Failed to detect file system"))?;
        if Self::validate_capacity(file_system, total_space).is_err() {
            return Err(anyhow!("Invalid capacity for the selected file system."));
        }

        let node = Node {
            id,
            storage_path,
            total_space,
            available_space: total_space,
            address: String::from(" "),
        };

        node.initialize_storage_file()?;

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

    fn control_permission(&self) -> bool {
        // 1. Dosyanın metadatasını al
        match fs::metadata(&self.storage_path) {
            Ok(meta) => {
                // 2. Eğer dosya yazılabilir değilse
                if meta.permissions().readonly() {
                    println!("Dosya yalnızca okunabilir. Yazma işlemi yapamıyorsunuz.");
                    return false;
                }
    
                // 3. Eğer dosya yazılabilir
                println!("Dosya yazılabilir. Yazma işlemi yapabilirsiniz.");
                return true;
            }
            Err(e) => {
                // 4. Dosya erişim hatası durumunda
                println!("Dosya erişim hatası: {}", e);
                return false;
            }
        }
    }
    

    // Check if the path is writable
    fn can_write_to_path(path: &str) -> bool {
        let path = Path::new(path);

        if path.is_dir() {
            let test_file = format!("{}/.test_write_permission", path.display());
            match fs::File::create(&test_file) {
                Ok(_) => {
                    // Test file created, remove it immediately
                    let _ = fs::remove_file(test_file);
                    true
                }
                Err(_) => false,
            }
        } else {
            match fs::OpenOptions::new().write(true).open(path) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    // Change file permissions (Windows/Unix)
    pub fn change_file_permission(&self) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("icacls")
                .arg(&self.storage_path)
                .arg("/grant")
                .arg("Everyone:(F)") // Full access for everyone
                .output()?;
            
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Permission change failed"));
            }
        }
        
        #[cfg(target_os = "unix")]
        {
            let output = Command::new("chmod")
                .arg("+w")
                .arg(&self.storage_path)
                .output()?;
            
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Permission change failed"));
            }
        }
        
        Ok(())
    }

    // Reduce available space after data is stored
    pub fn reduce_available_space(&mut self, file_size: u64) {
        if self.available_space >= file_size {
            self.available_space -= file_size;
            println!(
                "Node {}: Space reduced. Remaining available space: {}",
                self.id, self.available_space
            );
        } else {
            println!(
                "Node {}: Not enough space to store the file of size {}.",
                self.id, file_size
            );
        }
    }

    // Store data in the node's storage path
    pub fn store_data(&mut self, data: &[u8]) -> Result<()> {
        if self.available_space < data.len() as u64 {
            return Err(anyhow!("Not enough available space to store data."));
        }

        if Path::new(&self.storage_path).is_dir() {
            // If it's a directory, save data as a file
            let file_path = format!("{}/file_{}.dat", self.storage_path, uuid::Uuid::new_v4());
            fs::write(file_path, data)?;
        } else {
            // Otherwise, append data to the file
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(&self.storage_path)?;
            file.write_all(data)?;
        }

        self.available_space -= data.len() as u64;
        Ok(())
    }

    // Free up space in the node
    pub async fn free_up_space(&mut self, freed_space: u64) {
        self.available_space += freed_space;
        println!(
            "Node {}: Space freed. New available space: {}",
            self.id, self.available_space
        );
    }
}