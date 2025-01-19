// api/storage_api.rs

use crate::encryption::encrypt_file_chunked;
use crate::p2p::{find_available_node, Network, Node};
use crate::storage::{can_store_file, store_file};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::encryption::{
    encrypt_file_chunked, 
    decrypt_file_chunked,
    encrypt_data_chunked,
    decrypt_data_chunked
};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    file_id: String,
    file_name: String,
    file_size: u64,
    chunks: Vec<ChunkInfo>,
    timestamp: u64,
    owner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkInfo {
    chunk_id: String,
    node_id: String,
    size: u64,
    hash: String,
}

pub struct StorageAPI {
    network: Arc<Network>,
    file_index: Arc<Mutex<HashMap<String, FileMetadata>>>,
    storage_path: String,
}

impl StorageAPI {
    pub fn new(network: Arc<Network>, storage_path: &str) -> Self {
        let api = Self {
            network,
            file_index: Arc::new(Mutex::new(HashMap::new())),
            storage_path: storage_path.to_string(),
        };
        
        // Start Proof of Spacetime checking
        api.start_proof_of_spacetime();
        
        api
    }

    // Proof of Spacetime başlatma
    fn start_proof_of_spacetime(&self) {
        let storage_path = self.storage_path.clone();
        task::spawn(async move {
            periodic_check(&storage_path).await;
        });
    }

    pub async fn upload_file(
        &self,
        file_path: &str,
        owner: &str,
        encryption_password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 1. File preparation
        let file_data = self.read_file(file_path)?;
        let file_size = file_data.len() as u64;
        let file_id = generate_file_id(); // UUID veya benzeri unique ID
        
        // 2. Node selection
        let nodes = self.network.get_nodes().await;
        let available_nodes = find_available_nodes(&nodes, file_size)
            .ok_or("Yeterli depolama alanına sahip node bulunamadı")?;

        // 3. Encryption and chunking
        let encrypted_path = format!("{}_encrypted", file_path);
        encrypt_file_chunked(file_path, &encrypted_path, encryption_password)?;
        let chunks = split_into_chunks(&encrypted_path)?;

        // 4. Distribute chunks to nodes
        let mut chunk_infos = Vec::new();
        for (chunk_data, node) in chunks.iter().zip(available_nodes.iter()) {
            let chunk_id = store_chunk_on_node(chunk_data, node).await?;
            chunk_infos.push(ChunkInfo {
                chunk_id,
                node_id: node.id.clone(),
                size: chunk_data.len() as u64,
                hash: calculate_hash(chunk_data),
            });
        }

        // 5. Update metadata
        let metadata = FileMetadata {
            file_id: file_id.clone(),
            file_name: Path::new(file_path).file_name()?.to_str()?.to_string(),
            file_size,
            chunks: chunk_infos,
            timestamp: get_current_timestamp(),
            owner: owner.to_string(),
        };

        let mut index = self.file_index.lock().await;
        index.insert(file_id.clone(), metadata);

        Ok(file_id)
    }

    pub async fn download_file(
        &self,
        file_id: &str,
        destination: &str,
        encryption_password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Get file metadata
        let index = self.file_index.lock().await;
        let metadata = index.get(file_id)
            .ok_or("Dosya bulunamadı")?;
// 1. Şifrelenmiş dosyayı al
let encrypted_path = format!("{}.encrypted", destination_path);
let file_data = self.get_file_data(file_id).await?;
std::fs::write(&encrypted_path, file_data)?;

// 2. Şifreyi çöz
decrypt_file_chunked(&encrypted_path, destination_path, encryption_password)?;
        

        // 2. Retrieve chunks from nodes
        let mut chunks = Vec::new();
        for chunk_info in &metadata.chunks {
            let chunk_data = retrieve_chunk_from_node(&chunk_info.node_id, &chunk_info.chunk_id).await?;
            
            // Verify chunk integrity
            let chunk_hash = calculate_hash(&chunk_data);
            if chunk_hash != chunk_info.hash {
                return Err("Chunk bütünlüğü doğrulanamadı".into());
            }
            
            chunks.push(chunk_data);
        }

        // 3. Merge chunks and decrypt
        let encrypted_path = format!("{}_encrypted", destination);
        merge_chunks(&chunks, &encrypted_path)?;
        decrypt_file(&encrypted_path, destination, encryption_password)?;

        Ok(())
    }

    pub async fn delete_file(
        &self,
        file_id: &str,
        owner: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Verify ownership
        let mut index = self.file_index.lock().await;
        let metadata = index.get(file_id)
            .ok_or("Dosya bulunamadı")?;

        if metadata.owner != owner {
            return Err("Bu dosyayı silme yetkiniz yok".into());
        }

        // 2. Delete chunks from nodes
        for chunk_info in &metadata.chunks {
            delete_chunk_from_node(&chunk_info.node_id, &chunk_info.chunk_id).await?;
        }

        // 3. Remove metadata
        index.remove(file_id);

        Ok(())
    }

    // Helper functions
    fn read_file(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}