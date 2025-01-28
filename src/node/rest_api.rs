use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use reqwest::header;
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use uuid::Uuid;
use md5;

use bytes::BytesMut;

use super::disk_manager::{DiskManager, HealthStatus};


// REST API Structures
#[derive(Deserialize)]
pub struct UploadRequest {
    file_id: String,
}

#[derive(Serialize)]
struct SpaceResponse {
    available: u64,
    total: u64,
}



pub async fn upload_file(
    disk_manager: web::Data<Arc<DiskManager>>,
    req: web::Json<UploadRequest>,
    mut payload: Multipart,
) -> impl Responder {
    let mut body = BytesMut::new();

    // Multipart form'daki alanları iterasyonla kontrol et
    while let Some(field) = payload.next().await {
        match field {
            Ok(mut field) => {
                // Dosya alanını kontrol et
                if field.name() == Some("file") {
                    // Dosya parçalarını işleme
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(chunk) => {
                                // Disk alanı kontrolü
                                if !disk_manager.check_space(body.len() as u64 + chunk.len() as u64).await {
                                    return HttpResponse::InsufficientStorage().json("Not enough storage space");
                                }
                                body.extend_from_slice(&chunk);
                            }
                            Err(e) => return HttpResponse::BadRequest().body(format!("Upload error: {}", e)),
                        }
                    }
                }
            }
            Err(e) => return HttpResponse::BadRequest().body(format!("Error processing multipart: {}", e)),
        }
    }

    // Dosyayı kaydetme
    match disk_manager.store_file(&req.file_id, body.freeze()).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(e) => HttpResponse::InternalServerError().body(format!("Storage error: {}", e)),
    }
}




pub async fn download_file(
    disk_manager: web::Data<Arc<DiskManager>>,
    file_id: web::Path<String>,
) -> impl Responder {
    match disk_manager.read_file(&file_id).await {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
}

pub async fn delete_file(
    disk_manager: web::Data<Arc<DiskManager>>,
    file_id: web::Path<String>,
) -> impl Responder {
    match disk_manager.delete_file(&file_id).await {
        Ok(_) => HttpResponse::Ok().body("File deleted"),
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
}

pub async fn check_space(disk_manager: web::Data<Arc<DiskManager>>) -> impl Responder {
    let available = disk_manager.get_available_space().await;
    HttpResponse::Ok().json(SpaceResponse {
        available,
        total: disk_manager.config.max_capacity,
    })
}

pub async fn health_check(disk_manager: web::Data<Arc<DiskManager>>) -> impl Responder {
    let available = disk_manager.get_available_space().await;
    let status = HealthStatus {
        node_id: Uuid::new_v4().to_string(),
        available_space: available,
        total_space: disk_manager.config.max_capacity,
        is_healthy: true,
        last_checked: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    HttpResponse::Ok().json(status)
}

