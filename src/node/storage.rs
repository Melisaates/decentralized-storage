use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::path::Path;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

const NODE_STORAGE_PATH: &str = "./node_storage"; // Node depolama dizini
const MAX_STORAGE_SIZE: u64 = 5 * 1024 * 1024 * 1024; // 5 GB

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize)]
pub struct StorageResponse {
    pub status: String,
    pub message: String,
}

pub async fn check_storage_availability(file_size: u64) -> Result<bool, String> {
    let used_space = get_used_storage().await;
    if used_space + file_size <= MAX_STORAGE_SIZE {
        Ok(true)
    } else {
        Err("Yeterli alan yok!".to_string())
    }
}

pub async fn get_used_storage() -> u64 {
    let paths = tokio::fs::read_dir(NODE_STORAGE_PATH)
        .await
        .unwrap();
    
    let mut used_space = 0;
    for entry in paths {
        let entry = entry.unwrap();
        let metadata = entry.metadata().await.unwrap();
        used_space += metadata.len();
    }
    used_space
}

pub async fn save_file(file_data: Vec<u8>, file_name: &str) -> Result<FileInfo, String> {
    let storage_path = Path::new(NODE_STORAGE_PATH);
    if !storage_path.exists() {
        create_dir_all(storage_path).await.unwrap();
    }

    // Dosya için benzersiz bir ID oluşturuluyor
    let file_id = Uuid::new_v4().to_string();
    let file_path = format!("{}/{}.dat", NODE_STORAGE_PATH, file_id);

    // Dosya yazılıyor
    let mut file = File::create(file_path).await.unwrap();
    file.write_all(&file_data).await.unwrap();

    Ok(FileInfo {
        id: file_id,
        name: file_name.to_string(),
        size: file_data.len() as u64,
    })
}

pub async fn chunk_and_save_file(file_data: Vec<u8>, file_name: &str) -> Result<FileInfo, String> {
    const CHUNK_SIZE: usize = 1024 * 1024 * 10; // 10MB'lık parçalar

    let total_chunks = (file_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE; // Parça sayısını hesapla
    let mut file_info = Vec::new();

    for i in 0..total_chunks {
        let start = i * CHUNK_SIZE;
        let end = std::cmp::min(start + CHUNK_SIZE, file_data.len());
        let chunk = &file_data[start..end];

        // Her bir parçayı depola
        let chunk_file_name = format!("{}_part_{}", file_name, i);
        match save_file(chunk.to_vec(), &chunk_file_name).await {
            Ok(info) => file_info.push(info),
            Err(err) => return Err(err),
        }
    }

    Ok(file_info[0].clone()) // İlk dosyanın bilgisi döndürülür
}

