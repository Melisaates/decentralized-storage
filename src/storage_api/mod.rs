use crate::encryption::{decrypt_file_chunked, encrypt_file_chunked, split_file};
use crate::p2p::{find_available_node, Network, Node};
use crate::proof_of_spacetime::periodic_check;
use crate::storage::{self, can_store_file, store_chunk_on_node, store_file};
use actix_web::body::MessageBody;
use chrono::{Duration, Utc};
use libp2p::core::network;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use uuid::Uuid;

// Dosya metadata yapısı
#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub file_id: String,
    pub file_name: String,
    pub node_id: String,
    pub file_size: u64,
    chunks: Vec<ChunkInfo>,
    timestamp: u64, // Dosyanın yüklendiği zaman
    pub owner: String,
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
    // filemetadata ve dosya adı eşleştirmesi
    file_index: Arc<Mutex<HashMap<String, FileMetadata>>>,
    storage_path: String,
}

impl StorageAPI {
    pub async fn new(
        storage_path: &str,
        server_addr: SocketAddr,
        initial_peers: Vec<SocketAddr>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //create a new network
        let network = Arc::new(Network::new());

        // let node = Node {
        //     id: Uuid::new_v4().to_string(),
        //     storage_path: storage_path.to_string(),
        //     available_space: 1024 * 1024 * 1024, // 1GB storage space
        //     address: "127.0.0.1:8084".to_string(),
        // };

        // let node2 = Node {
        //     id: Uuid::new_v4().to_string(),
        //     storage_path: storage_path.to_string(),
        //     available_space: 1024 * 1024 * 1024, // 1GB storage space
        //     address: "127.0.0.1:8085".to_string()
        // };

        // let node3 = Node {
        //     id: Uuid::new_v4().to_string(),
        //     storage_path: storage_path.to_string(),
        //     available_space: 1024 * 1024 * 1024, // 1GB storage space
        //     address: "127.0.0.1:8086".to_string()
        // };
        // network.add_node(node);
        // network.add_node(node2);
        // network.add_node(node3);

        let network_clone = Arc::clone(&network);
        let network_clone2 = Arc::clone(&network);

        // Start the server
        tokio::spawn(async move {
            if let Err(e) = network_clone.start_server(server_addr).await {
                eprintln!("Failed to start server: {:?}", e);
            }
        });

        // Discover peers
        tokio::spawn(async move {
            network_clone2.periodic_peer_update(initial_peers).await;
        });

        // // Zamanlı kontrol işlemini başlat
        // let storage_path_clone = storage_path.to_string();
        // tokio::spawn(async move {
        //     periodic_check(&storage_path_clone).await;
        // });

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
        // Create a new file metadata
        let mut file = FileMetadata {
            file_id: Uuid::new_v4().to_string(),
            file_name: Path::new(file_path)
                .file_name()
                .ok_or("Failed to get file name")?
                .to_str()
                .ok_or("Failed to convert file name to string")?
                .to_string(),
            node_id: "".to_string(),
            file_size: std::fs::metadata(file_path)?.len(),
            chunks: Vec::new(),
            timestamp: Utc::now().timestamp() as u64,
            owner: owner.to_string(),
        };

        // // find available node
        let mut nodes = self.network.get_nodes().await;
        // let node_id = can_store_file(&mut nodes, file.file_size)
        //     .await
        //     .ok_or("No node found with enough storage space")?;
        // // selected node is the node that has enough space to store the file
        // let selected_node = nodes
        //     .iter()
        //     .find(|n| n.id == node_id)
        //     .ok_or("Node not found")?;

        // selected_node.clone().storage_path = format!("{}/{}", self.storage_path, selected_node.id);

        // Separate the file into chunks and encrypt
        let encrypted_path = "C:/Users/melisates/Documents/encrypted_file.mp4";
        // if !Path::new(&encrypted_path).exists() {
        //     return Err("Encrypted file was not created.".into());
        // }
        encrypt_file_chunked(
            &file.file_id,
            file_path,
            encrypted_path,
            encryption_password,
        )?;
        let chunks = split_file(&encrypted_path, 1024 * 1024); // 1MB chunk boyutu
        let mut chunk_infos = Vec::new();

        let mut chunk_count = 0;

        // 4. Save and share chunks on each node
        for chunk_data in chunks.iter() {
            chunk_count += 1;
            println!("chunk count: {:?}", chunk_count);
            // Get nodes in the network

            println!("chunkdata: {:?}", chunk_data.len());
            // Find a node that can store the chunk
            if let Some(node_id) = can_store_file(&mut nodes, chunk_data.len() as u64).await {
                // Find the selected node
                let selected_node = nodes.iter_mut().find(|node| node.id == node_id).unwrap();
                println!("selected node for every chunkdata: {:?}", selected_node.id);
                println!(
                    "selected node for every chunkdata: {:?}",
                    selected_node.storage_path
                );

                // fn sanitize_path_component(input: &str) -> String {
                //     input
                //         .chars()
                //         .filter(|c| c.is_alphanumeric() || *c == '_')
                //         .collect()
                // }
                // let sanitized_file_name = sanitize_path_component(&file.file_name.replace(" ", ""));
                // let sanitized_node_id =
                //     sanitize_path_component(&node_id.replace(".", "").replace(":", ""));

                // Store the chunk data on the selected node
                // store_file(
                //     chunk_data,
                //     &selected_node.storage_path,
                //     &format!("{}/{}", sanitized_file_name, sanitized_node_id),
                //     &node_id,
                // )?;
                // println!("file_name: {:?}", file.file_name);

                //selected_node.available_space -= chunk_data.len() as u64;

                // Share the chunk with the network
                if let Err(e) = store_chunk_on_node_with_retry(chunk_data, &selected_node, 3).await
                {
                    eprintln!(
                        "Failed to store chunk on node {} after retries: {:?}",
                        selected_node.id, e
                    );
                    return Err(e);
                }

                let chunk_info = ChunkInfo {
                    chunk_id: Uuid::new_v4().to_string(),
                    node_id: selected_node.id.clone(),
                    size: chunk_data.len() as u64,
                    hash: calculate_hash(chunk_data),
                };
                // Chunk bilgilerini sakla
                chunk_infos.push(chunk_info.clone());
                file.chunks.push(chunk_info);

                println!(
                    "CHUNK  Node ID: {}, Used: {}, Available: {}, Chunk Size: {}",
                    selected_node.id,
                    selected_node.total_space,
                    selected_node.available_space,
                    chunk_data.len() as u64
                );
            } else {
                println!("No suitable node found to store the chunk!");
                return Err("No suitable node found to store the chunk.".into());
            }
        }

        if chunk_count == chunks.len() {
            // Tüm parçalar başarıyla yüklendiyse, dosya indeksine ekle
            println!("All chunks uploaded successfully.");
            // Başarıyla dosya yüklemesi tamamlandı
            println!("File uploaded successfully: {:?}", file.file_name);
            println!("File ID: {:?}", file.file_id);
            self.file_index
                .lock()
                .await
                .insert(file.file_id.clone(), file.clone());
            Ok(file.file_id)
        } else {
            // Yükleme işlemi sırasında bir hata meydana geldi, gerekli işlemi yap
            eprintln!("Failed to upload all chunks, aborting file upload.");
            Err("Failed to upload all chunks.".into())
        }
    }

    // Düğüm listesini al
    pub async fn list_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        Ok(self.network.get_nodes().await)
    }

    // Dosya listesini al
    pub async fn list_files(&self) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
        let index = self.file_index.lock().await;
        Ok(index.values().cloned().collect())
    }

    pub async fn delete_file(&self, file_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Dosya bilgilerini al
        let file = self
            .file_index
            .lock()
            .await
            .get(file_id)
            .ok_or("File not found")?
            .clone();

        // Dosya parçalarını sırayla sil
        let mut chunk_count = 0;
        for chunk in file.chunks.iter() {
            let mut stored_node = self
                .network
                .get_node_by_id(&chunk.node_id)
                .await
                .ok_or("Node not found")?;

            // Parçayı noddan sil
            if let Err(e) = delete_chunk_from_node(&stored_node, &chunk.chunk_id).await {
                eprintln!(
                    "Failed to delete chunk {} from node {}: {:?}",
                    chunk.chunk_id, stored_node.id, e
                );
                return Err(e);
            }

            // Kapasiteyi güncelle
            stored_node.free_up_space(chunk.size).await;
            chunk_count += 1;
        }

        // Tüm parçalar başarıyla silindiyse dosyayı indeksden sil
        if chunk_count == file.chunks.len() {
            println!("All chunks deleted successfully.");
            self.file_index.lock().await.remove(file_id);
            Ok(format!("File with ID {} deleted successfully", file_id))
        } else {
            // Bir veya daha fazla parça silinemedi, dosya silme işlemi başarısız
            eprintln!("Failed to delete all chunks, aborting file deletion.");
            Err("Failed to delete all chunks.".into())
        }
    }

    pub async fn download_file_for_reading(
        &self,
        file_id: &str,
        destination_path: &str,
        encryption_password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let file = self
            .file_index
            .lock()
            .await
            .get(file_id)
            .ok_or("File not found")?
            .clone();

        let mut full_file_data = Vec::new();

        for chunk in file.chunks.iter() {
            let selected_node = self
                .network
                .get_node_by_id(&chunk.node_id)
                .await
                .ok_or("Node not found")?;

            let chunk_data = download_chunk_for_reading(&selected_node, &chunk.chunk_id).await?;
            full_file_data.extend(chunk_data);
        }

        let decrypted_path = format!("{}.decrypted", destination_path);
        if let Err(e) = decrypt_file_chunked(
            &file.file_id,
            &decrypted_path,
            destination_path,
            encryption_password,
        ) {
            eprintln!("Failed to decrypt file: {:?}", e);
            return Err(e.into());
        }

        println!(
            "File downloaded for reading successfully to: {:?}",
            destination_path
        );
        Ok(format!(
            "File downloaded for reading successfully to: {}",
            destination_path
        ))
    }

    pub async fn download_file_and_remove(
        &mut self,
        file_id: &str,
        destination_path: &str,
        encryption_password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let file = self
            .file_index
            .lock()
            .await
            .get(file_id)
            .ok_or("File not found")?
            .clone();

        let mut full_file_data = Vec::new();

        for chunk in file.chunks.iter() {
            let mut selected_node = self
                .network
                .get_node_by_id(&chunk.node_id)
                .await
                .ok_or("Node not found")?;

            let chunk_data =
                download_chunk_and_remove_from_node(&mut selected_node, &chunk.chunk_id).await?;
            full_file_data.extend(chunk_data);
        }

        let decrypted_path = format!("{}.decrypted", destination_path);
        decrypt_file_chunked(
            &file.file_id,
            &decrypted_path,
            destination_path,
            encryption_password,
        )?;

        println!(
            "File downloaded and removed from network successfully to: {:?}",
            destination_path
        );
        Ok(format!(
            "File downloaded and removed from network successfully to: {}",
            destination_path
        ))
    }
}

async fn download_chunk_for_reading(
    node: &Node,
    chunk_id: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let chunk_path = format!("{}/{}", node.storage_path, chunk_id);
    let chunk_data = match tokio::fs::read(&chunk_path).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "Failed to read chunk {chunk_id} from node {node_address}: {error:?}",
                chunk_id = chunk_id,
                node_address = node.address,
                error = e
            );
            return Err(Box::new(e));
        }
    };
    println!("Chunk {} read from node {}", chunk_id, node.id);
    Ok(chunk_data)
}

async fn download_chunk_and_remove_from_node(
    node: &mut Node,
    chunk_id: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let chunk_path: String = format!("{}/{}", node.storage_path, chunk_id);
    let chunk_data = tokio::fs::read(&chunk_path).await?;

    delete_chunk_from_node(node, chunk_id).await?;

    // Kapasiteyi güncelle
    node.free_up_space(chunk_data.len() as u64).await;

    println!(
        "Chunk {} downloaded and removed from node {}",
        chunk_id, node.id
    );
    Ok(chunk_data)
}

// Yardımcı fonksiyon: Noddan bir parçayı sil
async fn delete_chunk_from_node(
    node: &Node,
    chunk_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parçayı noddan silme işlemi
    let chunk_path: String = format!("{}/{}", node.storage_path, chunk_id);
    if let Err(e) = tokio::fs::remove_file(chunk_path).await {
        eprintln!("Error deleting chunk: {:?}", e);
        return Err(Box::new(e));
    }
    println!("Chunk {} deleted from node {}", chunk_id, node.id);
    Ok(())
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
        match store_chunk_on_node(chunk_data, selected_node, max_retries, 120).await {
            Ok(_) => return Ok(()), // Success
            Err(e) => {
                last_error = Some(e.to_string().into());
                eprintln!(
                    "Attempt {}: Failed to store chunk on node {}: {:?}",
                    attempt, selected_node.id, e
                );
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Err(last_error.unwrap_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown error",
        ))
    }))
}

pub async fn wait_for_peers(
    storage_api: &StorageAPI,
    timeout_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(timeout_seconds);
    let min_peers = 2;

    while start_time.elapsed() < timeout_duration {
        let nodes = storage_api.list_nodes().await?;
        println!("Current connected peers: {} with details:", nodes.len());

        // Print details of each connected node
        for node in &nodes {
            println!("  - Node ID: {}, Address: {}", node.id, node.address);
        }

        if nodes.len() >= min_peers {
            println!("Successfully connected to {} peers", nodes.len());
            return Ok(());
        }

        // // Add some initial nodes if none are present
        // if nodes.is_empty() {
        //     println!("No nodes found, adding initial nodes...");
        //     let initial_nodes = vec![
        //         Node {
        //             id: Uuid::new_v4().to_string(),
        //             storage_path: format!("{}/node1", storage_api.storage_path),
        //             available_space: 1024 * 1024 * 1024, // 1GB
        //             address: "127.0.0.1:8084".to_string(),
        //         },
        //         Node {
        //             id: Uuid::new_v4().to_string(),
        //             storage_path: format!("{}/node2", storage_api.storage_path),
        //             available_space: 1024 * 1024 * 1024, // 1GB
        //             address: "127.0.0.1:8085".to_string(),
        //         },
        //     ];

        //     for node in initial_nodes {
        //         storage_api.network.add_node(node).await;
        //     }
        //}

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Instead of returning error, return Ok with a warning
    println!(
        "Warning: Timeout reached, but proceeding with {} available peers",
        storage_api.list_nodes().await?.len()
    );
    Ok(())
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
