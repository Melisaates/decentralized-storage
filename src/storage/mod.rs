use crate::p2p::Node;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use sha2::{Sha256, Digest};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::time::Duration;
use tokio::time::timeout;
use anyhow::Error;

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

    // Store the file hash along with node_id in a registry 
    store_file_hash(file_hash, node_id);

    Ok(())
}

pub fn store_file_hash(file_hash: String, node_id: &str) {
    println!("Storing hash for file on node {}: {}", node_id, file_hash);
}

pub async fn can_store_file(
    nodes: &Vec<Node>, // List of all nodes
    file_size: u64,    // Size of the file
) -> Option<String> {
    // Return the ID of the node that can store the file
    for node in nodes {
        let storage_dir: &Path = Path::new(&node.storage_path);
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



// store_chunk_on_node fonksiyonu: Verilen chunk'ı seçilen node'a kaydeder
pub async fn store_chunk_on_node(
    chunk_data: &[u8],
    node: &Node,
    max_retries: u8, // Maksimum tekrar deneme sayısı başarısız olursa
    timeout_duration: u64, // Timeout süresi, örneğin saniye olarak
) -> anyhow::Result<()> {
    let node_address = node.address.clone();  // Örnek: "127.0.0.1:8080"
    
    // Bağlantı hatalarını ve tekrarları kontrol et
    let mut attempt = 0;
    let mut last_error: Option<anyhow::Error> = None;
    
    while attempt < max_retries {
        attempt += 1;

        //zaman aşımı süresi ile bağlantıyı kontrol et
        match timeout(Duration::from_secs(timeout_duration), TcpStream::connect(&node_address)).await {
            //tcp bağlantısı başarılı bir şekilde kuruldu hedef node'a chunk'ı gönder
            Ok(Ok(stream)) => {
                let mut writer = BufWriter::new(stream);
                
                match writer.write_all(chunk_data).await {
                    Ok(_) => {
                        // Veri yazma işlemi başarılı bir şekilde tamamlandı
                        writer.flush().await?;
                        println!("Chunk successfully stored on node: {}", node.id);
                        return Ok(()); // Başarılı bir şekilde veri gönderildi
                    }
                    Err(e) => {
                        eprintln!("Error writing to node {}: {:?}", node.id, e);
                        last_error = Some(Error::new(e));
                    }
                }
            },
            Ok(Err(e)) => {
                eprintln!("Error connecting to node {}: {:?}", node.id, e);
                last_error = Some(Error::from(e));  // Wrap the error into `anyhow::Error`
            },
            Err(_) => {
                eprintln!("Connection to node {} timed out.", node.id);
            },
        }

        // Hata durumunda beklemeden sonra tekrar dene
        //Eğer son deneme değilse 2 saniye bekle
        if attempt < max_retries {
            eprintln!("Retrying to store chunk on node: {} (Attempt {}/{})", node.id, attempt, max_retries);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    // Sonuçta, tüm denemeler başarısız olduysa hatayı döndür
    Err(last_error.unwrap_or_else(|| anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::Other, "Unknown error"))))
}
