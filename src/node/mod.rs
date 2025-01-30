use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use std::fs::DirBuilder;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_rt::System;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use winapi::shared::ntdef::PULARGE_INTEGER;
use crate::file_system::{file_operations, FileSystem};
use std::fs::metadata;

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt; // Unix için ekstra bilgi

#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt; // Windows için ekstra bilgi

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StorageNode {
    pub node_id: String,
    storage_path: String,
    pub total_space: u64,
    pub available_space: u64,
    pub health_status: bool,
    pub last_checked: u64,
}

impl StorageNode {
    pub async fn initialize_storage_file(&mut self) -> Result<()> {
        let path = Path::new(&self.storage_path);
    
        
        if path.exists() {
            println!("Storage directory already exists for node {} at {}", self.node_id, self.storage_path);
            // Burada mevcut dosyanın mevcut alanını güncelleyin
            println!("Updating available space...");
            let available_space = self.calculate_available_space()?;
            self.available_space = available_space;  // Available space güncelleniyor
            println!("Updated available space: {}", self.available_space);
        } else {
            println!("Creating storage directory for node {} at {}", self.node_id, self.storage_path);
            DirBuilder::new().recursive(true).create(&self.storage_path)?;
        }
        
        // Create capacity.txt with the specified capacity
        // let capacity_file_path = format!("{}/capacity.txt", self.storage_path);
        // if !Path::new(&capacity_file_path).exists() {
        //     let mut dir_file = fs::File::create(&capacity_file_path)?;
        //     writeln!(dir_file, "Capacity: {}", self.total_space)?;
        // }

        
        // Create a file to actually reserve the physical space on disk
        // storage_file.dat is created with the specified capacity
        let storage_file_path = format!("{}/storage_file.dat", self.storage_path);
        let mut storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&storage_file_path)
            .map_err(|e| anyhow!("Failed to open storage file {}: {}", storage_file_path, e))?;
        storage_file.set_len(self.total_space)
            .map_err(|e| anyhow!("Failed to set length for storage file {}: {}", storage_file_path, e))?;
        storage_file.flush()
            .map_err(|e| anyhow!("Failed to flush storage file {}: {}", storage_file_path, e))?;  // Önce veriyi yaz
        storage_file.sync_all()
            .map_err(|e| anyhow!("Failed to sync storage file {}: {}", storage_file_path, e))?; // Disk senkronizasyonu yap (Windows + Linux)
        // let mut storage_file = fs::File::create(&storage_file_path)?;
        // storage_file.set_len(self.total_space)?;  // Burada dosya boyutu ayarlanıyor
        println!("************StorageNode {}: Available space: {}", self.node_id, self.available_space);
    
        // Kontrolleri yap
        if !file_operations::control_permission(self.storage_path.as_str()) {
            println!("Attempting to change write permission...");
            file_operations::change_file_permission(self.storage_path.as_str())?;
            if !file_operations::control_permission(self.storage_path.as_str()) {
                return Err(anyhow!("No write permission for storage path: {}", self.storage_path));
            }
        }
    
        Ok(())
    }
    

    pub async fn new(node_id: String, total_space: u64) -> Result<Self> {
        let file_system = FileSystem::detect_file_system().ok_or_else(|| anyhow!("Failed to detect file system"))?;
        Self::validate_capacity(file_system, total_space)?;

        // Calculate available space
        let storage_path = format!("storage/{}", node_id);
        let mut node = StorageNode {
            node_id,
            storage_path: storage_path.clone(),
            total_space,
            available_space: total_space,
            health_status: true,
            last_checked: 0,
        };

        node.initialize_storage_file().await?;
        node.update_available_space()?;  // Update available space dynamically
        Ok(node)
    }

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

    // Update available space based on the current system state
    pub fn update_available_space(&mut self) -> Result<()> {
        let available_space = Self::calculate_available_space(self)?;
        self.available_space = available_space;
        println!(
            "Node {}: Available space updated to {} bytes.",
            self.node_id, self.available_space
        );
        Ok(())
    }

    // pub fn reduce_available_space(&mut self, file_size: u64) {
    //     if self.available_space >= file_size {
    //         self.available_space -= file_size;
    //         println!(
    //             "StorageNode {}: Space reduced. Remaining available space: {}",
    //             self.node_id, self.available_space
    //         );
    //     } else {
    //         println!(
    //             "StorageNode {}: Not enough space to store the file of size {}.",
    //             self.node_id, file_size
    //         );
    //     }
    // }

// Updated disk space calculation method
// nasıl hesaplıyor? sözlü olarak: toplam alan - mevcut dosya boyutu


    pub fn calculate_available_space(&self) -> io::Result<u64> {
        let storage_dir = Path::new(&self.storage_path);

        if !storage_dir.exists() {
            return Ok(self.total_space); // Eğer dizin yoksa, tamamen boş kabul et
        }

        let mut used_space = 0;

        for entry in fs::read_dir(storage_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            used_space += metadata.len();
        }

        // Platform bağımsız olarak kullanılabilir alanı hesapla
        let available_space = self.total_space.saturating_sub(used_space);

        println!(
            "StorageNode {}: Total: {} | Used: {} | Available: {}",
            self.node_id, self.total_space, used_space, available_space
        );

        Ok(available_space)
    }



    pub async fn store_file(&mut self, file_id: &str, data: &[u8]) -> Result<()> {
        // Dynamically update available space each time a file is stored
        let file_size = data.len() as u64;
        if file_size > self.available_space {
            return Err(anyhow!("Insufficient storage space").into());
        }

        let file_path = self.get_file_path(file_id);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
            .map_err(|e| anyhow!("Error opening file: {}", e))?;

        file.write_all(data).map_err(|e| anyhow!("Error writing to file: {}", e))?;
        self.available_space -= file_size;
        
        self.update_available_space()?;
        println!("********** After storing file: Available space: {}", self.available_space);
        self.update_health_status().await?;
        

        Ok(())
    }

    pub async fn retrieve_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.get_file_path(file_id);
        let mut file = File::open(file_path).map_err(|e| anyhow!("Error opening file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| anyhow!("Error reading file: {}", e))?;
        Ok(buffer)
    }

    // Helper to construct file path
    fn get_file_path(&self, file_id: &str) -> PathBuf {
        Path::new(&self.storage_path).join(file_id)
    }

    // Health Check Methods
    pub async fn update_health_status(&mut self) -> Result<()> {
        self.health_status = self.perform_health_check().await?;
        self.last_checked = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }

    async fn perform_health_check(&mut self) -> Result<bool> {
        let available = self.calculate_available_space()?;
        if available == 0 {
            return Ok(false);
        }

        // Check write capability
        let test_file = self.get_file_path("health_check.tmp");
        if let Err(_) = fs::write(&test_file, b"health check") {
            return Ok(false);
        }
        fs::remove_file(test_file).map_err(|e| anyhow!("Error cleaning up health check file: {}", e))?;

        Ok(true)
    }

    pub fn free_up_space(&mut self, file_path: &str) -> Result<()> {
        let full_path = self.get_file_path(file_path); // Get the full path using get_file_path
        
        // Check if the file exists and delete it
        if let Ok(metadata) = fs::metadata(&full_path) {
            let file_size = metadata.len();
            println!("Deleting file '{}', size: {}", full_path.display(), file_size); // Debugging line
            
            fs::remove_file(&full_path)?;
            self.available_space += file_size;
    
            // Recalculate the available space based on the current disk status
            // self.available_space = Self::calculate_available_space(&self.storage_path)?;
            // println!(
            //     "********** After deletion: StorageNode {}: File '{}' deleted. New available space: {}",
            //     self.node_id, full_path.display(), self.available_space
            // );
        } else {
            println!(
                "StorageNode {}: File '{}' does not exist at path '{}'.",
                self.node_id, file_path, full_path.display()
            );
        }
        self.update_available_space()?;
        Ok(())
    }
    
    
    
}


/*use std::time::{SystemTime, UNIX_EPOCH};
use tokio; // Tokio async runtime
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let node_id = "node_1".to_string();
    let total_space = 100_000_000; // 100 MB örnek kapasite

    // StorageNode oluşturma
    let node = StorageNode::new(node_id, total_space).await?;

    println!("StorageNode oluşturuldu: {:?}", node);

    Ok(())
}
 */