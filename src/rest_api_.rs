use axum::{extract::Multipart, routing::post, Router};
use std::{fs::File, io::Write, path::Path};
use tower_http::limit::RequestBodyLimitLayer;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage_node::StorageNode; // StorageNode'ı doğru şekilde dahil edin

// Dosya yükleme işlemi
async fn upload_file(mut multipart: Multipart) -> Result<String, axum::http::StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        
        // Storage path'ini oluşturun
        let storage_path = format!("./storage/{}", file_name);
        let mut file = File::create(&storage_path)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Dosya verisini yazın
        file.write_all(&data)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        
        println!("✅ File '{}' uploaded successfully!", file_name);
        
        // `StorageNode` ile dosyayı saklama
        let mut storage_node = StorageNode::new("storage".to_string(), 100 * 1024 * 1024); // 100MB kapasiteli node
        // `store_file` fonksiyonunu çağırarak dosyayı yükleyin
        storage_node
            .store_file(&file_name, &data)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        
        return Ok(format!("File '{}' uploaded and stored successfully!", file_name));
    }

    Err(axum::http::StatusCode::BAD_REQUEST)
}

// API handler
async fn delete_file_handler(
    Path(file_name): Path<String>,
    storage_node: Arc<Mutex<String>>, // Storage path'ini paylaşmak için
) -> Json<String> {
    let storage_path = storage_node.lock().await; // storage_path'i kilitle

    match delete_file(&storage_path, &file_name) {
        Ok(_) => Json(format!("File '{}' deleted successfully", file_name)),
        Err(_) => Json(format!("Error deleting file '{}'", file_name)),
    }
}


/*
#[tokio::main]
async fn main() {
//uploaddddddddddddddddddddddddd
    let app = Router::new()
        .route("/upload", post(upload_file))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)); // 10MB limit

    println!("🚀 Server running at http://127.0.0.1:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
        //deleteeeeeeeeeeeeeeeeeeeeeeeeeeee
        let storage_path = Arc::new(Mutex::new("/path/to/your/storage".to_string())); // Burada storage path'i belirtiyoruz

    // API router
    let app = Router::new()
        .route("/delete-file/:file_name", post(delete_file_handler))
        .layer(axum::AddExtensionLayer::new(storage_path));

    // Sunucu başlatma
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
 */