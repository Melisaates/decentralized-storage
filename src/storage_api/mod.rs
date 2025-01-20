use crate::encryption::{decrypt_file_chunked, encrypt_file_chunked, split_file};
use crate::p2p::{Network, Node, find_available_node};
use crate::proof_of_spacetime::periodic_check;
use crate::storage::{can_store_file, store_chunk_on_node, store_file};
use std::path::Path;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{Duration, Utc};
use sha2::{Sha256, Digest};
// Dosya metadata yapısı
#[derive(Clone, Debug)]
pub struct FileMetadata {
    file_id: String,
    file_name: String,
    node_id: String,
    file_size: u64,
    chunks: Vec<ChunkInfo>,
    timestamp: u64,
    owner: String,
}

// Chunk bilgisi yapısı
#[derive(Clone, Debug)]
pub struct ChunkInfo {
    chunk_id: String,
    node_id: String,
    size: u64,
    hash: String,
}

fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// StorageAPI yapısı
pub struct StorageAPI {
    network: Arc<Network>,
    file_index: Arc<Mutex<HashMap<String, FileMetadata>>>,
    storage_path: String,
}

impl StorageAPI {
    // Yeni bir StorageAPI örneği oluşturur
    pub async fn new(storage_path: &str, server_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let network = Arc::new(Network::new());
        let addr: SocketAddr = server_addr.parse()?;
        
        // Ağı başlat
        let network_clone = Arc::clone(&network);
        tokio::spawn(async move {
            if let Err(e) = network_clone.start_server(addr).await {
                eprintln!("Server error: {:?}", e);
            }
        });

        // Zamanlı kontrol işlemini başlat
        let storage_path_clone = storage_path.to_string();
        tokio::spawn(async move {
            periodic_check(&storage_path_clone).await;
        });

        Ok(Self {
            network,
            file_index: Arc::new(Mutex::new(HashMap::new())),
            storage_path: storage_path.to_string(),
        })
    }

    // upload_file fonksiyonu, veriyi şifreler ve düğümlere yükler
    pub async fn upload_file(
        &self,
        file_path: &str,
        owner: &str,
        encryption_password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 1. Dosya bilgilerini al
        let file_size = std::fs::metadata(file_path)?.len();
        let file_id = Uuid::new_v4().to_string();
        let file_name = Path::new(file_path)
            .file_name()
            .ok_or("Failed to get file name")?
            .to_str()
            .ok_or("Failed to convert file name to string")?
            .to_string();
    
        // 2. Yeterli alana sahip düğümü bul
        let nodes = self.network.get_nodes().await;
        let node_id = can_store_file(&nodes, file_size)
            .await
            .ok_or("No node found with enough storage space")?;
        let selected_node = nodes.iter().find(|n| n.id == node_id).ok_or("Node not found")?;
    
        // 3. Dosyayı şifrele ve chunk'lara ayır
        let encrypted_path = format!("{}_encrypted", file_path);
        encrypt_file_chunked(file_path, &encrypted_path, encryption_password)?;
        let chunks = split_file(&encrypted_path, 1024 * 1024); // 1MB chunk boyutu
        let mut chunk_infos = Vec::new();
    
        // 4. Chunk'ları her bir düğüme kaydet ve paylaş
        for chunk_data in chunks.iter() {
            // Ağdaki düğümleri al
            let nodes = self.network.get_nodes().await;
    
            // Uygun bir düğüm bul
            if let Some(node_id) = can_store_file(&nodes, chunk_data.len() as u64).await {
                // Düğüm bilgilerini bul
                let selected_node = nodes.iter().find(|node| node.id == node_id).unwrap();
    
                // Chunk'ı düğüme kaydet
                store_file(chunk_data, &selected_node.storage_path, &file_name, &node_id)?;
    
                // Chunk verisini merkeziyetsiz ağda paylaş (retry mekanizması ile)
                if let Err(e) = store_chunk_on_node_with_retry(chunk_data, &selected_node, 3).await {
                    eprintln!("Failed to store chunk on node {} after retries: {:?}", selected_node.id, e);
                    return Err(e);
                }
    
                // Chunk bilgilerini sakla
                chunk_infos.push(ChunkInfo {
                    chunk_id: Uuid::new_v4().to_string(),
                    node_id: selected_node.id.clone(),
                    size: chunk_data.len() as u64,
                    hash: calculate_hash(chunk_data),
                });
            } else {
                println!("No suitable node found to store the chunk!");
                return Err("No suitable node found to store the chunk.".into());
            }
        }
        
        // Başarıyla dosya yüklemesi tamamlandı
        Ok(file_id)
    }
}

// store_chunk_on_node_with_retry: Chunk'ı node'a kaydetmek için retry mekanizması içerir
// Eğer başarısız olursa, belirli bir süre bekler ve tekrar dener
pub async fn store_chunk_on_node_with_retry(
    chunk_data: &[u8],
    selected_node: &Node,
    max_retries: u8,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut attempt = 0;
    let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

    while attempt < max_retries {
        attempt += 1;
        match store_chunk_on_node(chunk_data, selected_node, max_retries).await {
            Ok(_) => return Ok(()), // Success
            Err(e) => {
                // Box the error and ensure it implements Send and Sync
                last_error = Some(e.to_string().into());
                eprintln!("Attempt {}: Failed to store chunk on node {}: {:?}", attempt, selected_node.id, e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Err(last_error.unwrap_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Unknown error"))))
}





    // // Dosyayı ağdan indirir
    // pub async fn download_file(
    //     &self,
    //     file_id: &str,
    //     destination: &str,
    //     encryption_password: &str,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let index = self.file_index.lock().await;
    //     let metadata = index.get(file_id)
    //         .ok_or("File not found")?;

    //     // 2. Retrieve chunks from nodes
    //     let mut chunks = Vec::new();
    //     for chunk_info in &metadata.chunks {
    //         let chunk_data = retrieve_chunk_from_node(&chunk_info.node_id, &chunk_info.chunk_id).await?;
            
    //         // Verify chunk integrity
    //         let chunk_hash = calculate_hash(&chunk_data);
    //         if chunk_hash != chunk_info.hash {
    //             return Err("Chunk bütünlüğü doğrulanamadı".into());
    //         }
            
    //         chunks.push(chunk_data);
    //     }

    //     // 3. Merge chunks and decrypt
    //     let encrypted_path = format!("{}_encrypted", destination);
    //     merge_chunks(&chunks, &encrypted_path)?;
    //     decrypt_file_chunked(&encrypted_path, destination, encryption_password)?;

    //     Ok(())
    // }

    // // Düğüm listesini al
    // pub async fn list_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    //     Ok(self.network.get_nodes().await)
    // }

    // // Dosya listesini al
    // pub async fn list_files(&self) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
    //     let index = self.file_index.lock().await;
    //     Ok(index.values().cloned().collect())
    // }
