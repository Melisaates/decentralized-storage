use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::fs::DirBuilder;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_rt::System;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use winapi::shared::ntdef::PULARGE_INTEGER;
use crate::file_system::{file_operations, FileSystem};

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
            let available_space = Self::calculate_available_space(&self.storage_path)?;
            self.available_space = available_space;  // Available space güncelleniyor
            println!("Updated available space: {}", self.available_space);
        } else {
            DirBuilder::new().recursive(true).create(&self.storage_path)?;
        }
        
        // Create capacity.txt with the specified capacity
        // let capacity_file_path = format!("{}/capacity.txt", self.storage_path);
        // if !Path::new(&capacity_file_path).exists() {
        //     let mut dir_file = fs::File::create(&capacity_file_path)?;
        //     writeln!(dir_file, "Capacity: {}", self.total_space)?;
        // }


        // Create a file to actually reserve the physical space on disk
        let storage_file_path = format!("{}/storage_file.dat", self.storage_path);
        let mut storage_file = fs::File::create(&storage_file_path)?;
        storage_file.set_len(self.total_space)?;  // Burada dosya boyutu ayarlanıyor
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
        node.update_available_space()?;  // Update available space dynamically

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
        let available_space = Self::calculate_available_space(&self.storage_path)?;
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
fn calculate_available_space(path: &str) -> Result<u64> {
    let path = Path::new(path);
    
    // Get available space for the specific path
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(path).map_err(|e| anyhow!("Failed to get metadata for path {}: {}", path.display(), e))?;
        let fs_stats = nix::sys::statvfs::statvfs(path)
            .map_err(|e| anyhow!("Failed to get filesystem stats for path {}: {}", path.display(), e))?;
        
        // Calculate available space in bytes
        let available = fs_stats.block_size() as u64 * fs_stats.blocks_available() as u64;
        Ok(available)
    }

    #[cfg(target_family = "windows")]
    {
        use winapi::um::fileapi::{GetDiskFreeSpaceExW};
        use std::os::windows::ffi::OsStrExt;
        
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut available: u64 = 0;
        let mut total: u64 = 0;
        let mut free: u64 = 0;
        
        let result = unsafe {
            GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut available as *mut u64 as PULARGE_INTEGER,
                &mut total as *mut u64 as PULARGE_INTEGER,
                &mut free as *mut u64 as PULARGE_INTEGER,
            )
        };
        
        if result == 0 {
            Err(anyhow!("Failed to get disk space for path: {}", path.display()))
        } else {
            Ok(available)
        }
    }

    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        Err(anyhow!("Unsupported platform for disk space calculation"))
    }
}


    pub async fn store_file(&mut self, file_id: &str, data: &[u8]) -> Result<()> {
        // Dynamically update available space each time a file is stored
        self.update_available_space()?;
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
        let available_space = Self::calculate_available_space(&self.storage_path)?;
        println!("**********store da  available space: {}", available_space);
        self.available_space -= file_size;
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

    async fn perform_health_check(&self) -> Result<bool> {
        let available = Self::calculate_available_space(&self.storage_path)?;
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