use actix_web::{web::{self, Query}, HttpResponse, Responder};
use async_std::stream::StreamExt;
use serde::Deserialize;
use actix_multipart::Multipart;
use axum::{extract::Path, routing::post, Json, Router, response::IntoResponse};
use std::{fs::File, io::Write};
use actix_web::Error;
use tower_http::limit::RequestBodyLimitLayer;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::node::StorageNode;


use actix_web::{App, HttpServer, post, get, delete};

#[derive(Deserialize)]
struct UploadRequest {
    node_id: String,
    file_id: String,
}

// #[post("/upload/{file_id}")]
// async fn upload_file(mut payload: Multipart, form: web::Json<UploadRequest>) -> Result<HttpResponse, Error> {
//     let node_id = &form.node_id;
//     let file_id = &form.file_id;

//     // Loop over the multipart form data (file parts)
//     while let Some(Ok(mut field)) = payload.next().await {
//         let content_disposition = field.content_disposition().unwrap();
//         let file_name = content_disposition.get_filename().unwrap_or("unknown");

//         // Store the file temporarily
//         let mut file_data = Vec::new();
//         while let Some(Ok(bytes)) = field.next().await {
//             file_data.extend_from_slice(&bytes);
//         }

//         // You can handle file encryption here if needed
//         // Store the file using the node's store_file method
//         let source_file_path = "temporary_path_to_file"; // Modify as per your setup
//         let result = node.store_file(file_id, source_file_path).await;

//         match result {
//             Ok(_) => {
//                 // Return success response if file was stored successfully
//                 return Ok(HttpResponse::Ok().json("File uploaded and stored successfully"));
//             }
//             Err(e) => {
//                 // Handle errors if storing the file fails
//                 return Ok(HttpResponse::InternalServerError().json(format!("Error storing file: {}", e)));
//             }
//         }
//     }

//     Err(actix_web::error::ErrorBadRequest("File upload failed"))
// }
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct FileUpload {
    pub file_id: String,
    pub file_path: String,
}
#[post("/upload/{file_id}")]
async fn upload_file(
    storage_node: web::Data<Arc<Mutex<StorageNode>>>, 
    form: web::Form<FileUpload>,  // web::Json yerine web::Form
    mut payload: Multipart,       // multipart veri
) -> impl Responder {
    println!("Entering upload_file endpoint...");



    let file_id = &form.file_id;
    let file_path = &form.file_path;

    while let Some(Ok(mut field)) = payload.next().await {
        println!("Form data received: {:?}", form);
        println!("File received: {:?}", field);
        println!("Processing field: {:?}", field);
        let mut file_data = Vec::new();
        while let Some(Ok(bytes)) = field.next().await {
            file_data.extend_from_slice(&bytes);
        }

        let content_disposition = field.content_disposition().unwrap();
        let file_name = content_disposition.get_filename().unwrap_or("unknown");

        let temp_file_path = format!("./{}", file_name);
        let mut temp_file = File::create(&temp_file_path).unwrap();
        //temp_file.write_all(&file_data).unwrap();
        if let Err(e) = temp_file.write_all(&file_data) {
            println!("Error writing file: {:?}", e);
            return HttpResponse::InternalServerError().json(format!("Error writing file: {:?}", e));
        }
        

        let mut storage_node = storage_node.lock().await;
        match storage_node.store_file(file_id, &temp_file_path).await {
            Ok(_) => {
                println!("File stored successfully");
                return HttpResponse::Ok().json("File uploaded and encrypted successfully.");
            }
            Err(e) => {
                println!("Error storing file: {}", e);
                return HttpResponse::InternalServerError().json(format!("Error: {}", e));
            }
        }
    }

    HttpResponse::BadRequest().json("Failed to upload file.")
}

#[delete("/delete/{file_id}")]
async fn delete_file(node: web::Data<Arc<Mutex<StorageNode>>>, file_id: web::Path<String>) -> impl Responder {
    let file_id = file_id.into_inner();
    
    let mut node = node.lock().await;
    match node.delete_file(&file_id) {
        Ok(_) => HttpResponse::Ok().body("File deleted successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Deletion failed: {}", e)),
    }
}

#[get("/download/{file_id}")]
async fn download_file(node: web::Data<Arc<Mutex<StorageNode>>>, file_id: web::Path<String>) -> impl Responder {
    let file_id = file_id.into_inner();
    let download_path = format!("downloads/{}.decrypted", file_id);
    
    let mut node = node.lock().await;
    match node.retrieve_file(&file_id, &download_path).await {
        Ok(_) => {
            match std::fs::read(&download_path) {
                Ok(data) => HttpResponse::Ok().body(data),
                Err(_) => HttpResponse::InternalServerError().body("Failed to read downloaded file"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Download failed: {}", e)),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file);
    cfg.service(download_file);
    cfg.service(delete_file);
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     let storage_node = StorageNode::new("node1".to_string(), 10_000_000_000).await.unwrap();
//     let storage_node_data = web::Data::new(storage_node);
    
//     HttpServer::new(move || {
//         App::new()
//             .app_data(storage_node_data.clone())
//             .configure(config)
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }


/*
#[tokio::main]
async fn main() {
//DOWNLOADDDDDDDDDDDDDDDDDDDDDD
let node = StorageNode {
        storage_path: "/mnt/storage".to_string(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(node.clone()))
            .wrap(Logger::default())  // Sunucu loglaması
            .route("/download/{file_id}", web::get().to(download_file)) // API endpoint
    })
    .bind("127.0.0.1:8080")?  // Port 8080'de dinlesin
    .run()
    .await


//uploaddddddddddddddddddddddddd
    let app = Router::new()
        .route("/upload", post(upload_file))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)); // 10MB limit

    println!("Server running at http://127.0.0.1:3000");
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


 /* HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(disk_manager.clone()))
            .route("/upload", web::post().to(upload_file))
            .route("/download/{file_id}", web::get().to(download_file))
            .route("/space", web::get().to(check_space))
            .route("/health", web::get().to(health_check))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await */