use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use std::{error::Error, io::Write, sync::PoisonError};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use anyhow::Result;
use std::collections::HashMap;

use crate::node::StorageNode;

// Request/Response structs
#[derive(Deserialize)]
struct CreateNodeRequest {
    node_id: String,
    total_space: u64,
}

#[derive(Serialize)]
struct NodeResponse {
    node_id: String,
    total_space: u64,
    available_space: u64,
    health_status: bool,
}

// State management for storage nodes
pub struct AppState {
    nodes: Mutex<HashMap<String, StorageNode>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }
}

// API Routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/nodes")
                    .route("", web::post().to(create_node))
                    .route("/{node_id}", web::get().to(get_node))
                    .route("/{node_id}", web::delete().to(delete_node))
                    .route("/{node_id}/health", web::get().to(check_node_health))
            )
            .service(
                web::scope("/files")
                    .route("/upload/{node_id}", web::post().to(upload_file))
                    .route("/download/{node_id}/{file_id}", web::get().to(download_file))
                    .route("/{node_id}/{file_id}", web::delete().to(delete_file))
            )
            .service(
                web::scope("/test")
                    .route("", web::get().to(test_endpoint))
            )
            
    );
}

// Node Management Handlers
async fn create_node(
    data: web::Data<AppState>,
    req: web::Json<CreateNodeRequest>,
) -> impl Responder {
    let mut nodes = data.nodes.lock().unwrap();
    
    match StorageNode::new(req.node_id.clone(), req.total_space).await {
        Ok(node) => {
            nodes.insert(req.node_id.clone(), node);
            HttpResponse::Created().json(NodeResponse {
                node_id: req.node_id.clone(),
                total_space: req.total_space,
                available_space: req.total_space,
                health_status: true,
            })
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_node(
    data: web::Data<AppState>,
    node_id: web::Path<String>,
) -> impl Responder {
    let nodes = data.nodes.lock().unwrap();
    
    match nodes.get(node_id.as_str()) {
        Some(node) => HttpResponse::Ok().json(NodeResponse {
            node_id: node.node_id.clone(),
            total_space: node.total_space,
            available_space: node.available_space,
            health_status: node.health_status,
        }),
        None => HttpResponse::NotFound().body("Node not found"),
    }
}

async fn delete_node(
    data: web::Data<AppState>,
    node_id: web::Path<String>,
) -> impl Responder {
    let mut nodes = data.nodes.lock().unwrap();
    
    if nodes.remove(&node_id.to_string()).is_some() {
        HttpResponse::Ok().body("Node deleted successfully")
    } else {
        HttpResponse::NotFound().body("Node not found")
    }
}

async fn check_node_health(
    data: web::Data<AppState>,
    node_id: web::Path<String>,
) -> impl Responder {
    let mut nodes = data.nodes.lock().unwrap();
    
    if let Some(node) = nodes.get_mut(&node_id.to_string()) {
        match node.update_health_status().await {
            Ok(_) => HttpResponse::Ok().json(node.health_status),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    } else {
        HttpResponse::NotFound().body("Node not found")
    }
}


use std::fs::File;

fn handle_poison_error<T>(_: PoisonError<T>) -> HttpResponse {
    HttpResponse::InternalServerError().body("Internal server error")
}

async fn upload_file(
    data: web::Data<AppState>,
    node_id: web::Path<String>,
    mut payload: Multipart,
) -> HttpResponse {
    println!("Starting file upload for node: {}", node_id);
    
    let mut nodes = match data.nodes.lock() {
        Ok(guard) => guard,
        Err(poison_err) => return handle_poison_error(poison_err),
    };

    let node = match nodes.get_mut(node_id.as_str()) {
        Some(n) => n,
        None => {
            println!("Node not found: {}", node_id);
            return HttpResponse::NotFound().body("Node not found");
        }
    };

    let temp_dir = "temp_uploads";
    if let Err(e) = std::fs::create_dir_all(temp_dir) {
        println!("Error creating temp directory: {}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to create temp directory: {}", e));
    }

    let file_future = async {
        // Dosyanın var olup olmadığını kontrol et
        if let Some(mut field) = payload.try_next().await.unwrap_or(None) {
            let content_disposition = field.content_disposition();
            let filename = match content_disposition.as_ref().and_then(|cd| cd.get_filename()) {
                Some(name) => name.to_string(),
                None => {
                    println!("No filename in field");
                    return Err("No filename in field".to_string());
                }
            };
            println!("Processing file: {}", filename);

            let temp_filepath = format!("{}/{}_{}", temp_dir, Uuid::new_v4(), filename);
            println!("Temp file path: {}", temp_filepath);

            let temp_filepath_clone = temp_filepath.clone();
            let file_result = web::block(move || std::fs::File::create(temp_filepath_clone)).await;
            let mut temp_file = match file_result {
                Ok(Ok(file)) => file,
                Ok(Err(e)) => {
                    println!("Failed to create temp file: {}", e);
                    return Err(format!("Failed to create temp file: {}", e));
                }
                Err(e) => {
                    println!("Block error: {}", e);
                    return Err(format!("Block error: {}", e));
                }
            };

            while let Ok(Some(chunk)) = field.try_next().await {
                let mut temp_file_clone = temp_file.try_clone().map_err(|e| format!("Failed to clone temp file: {}", e))?;
                if let Err(e) = web::block(move || {
                    temp_file_clone.write_all(&chunk)
                })
                .await
                {
                    println!("Error writing chunk: {}", e);
                    return Err(format!("Failed to write file chunk: {}", e));
                }
            }


            let unique_filename = format!("{}_{}", Uuid::new_v4(), filename);
            match node.store_file(&unique_filename, &temp_filepath).await {
                Ok(_) => {
                    println!("File stored successfully");
                    if let Err(e) = std::fs::remove_file(&temp_filepath) {
                        println!("Warning: Failed to remove temp file: {}", e);
                    }
                    return Ok(filename);
                }
                Err(e) => {
                    println!("Error storing file: {}", e);
                    if let Err(cleanup_err) = std::fs::remove_file(&temp_filepath) {
                        println!("Warning: Failed to remove temp file: {}", cleanup_err);
                    }
                    return Err(format!("Failed to store file: {}", e));
                }
            }
        } else {
            println!("No file found in request");
            return Err("No file found in request".to_string());
        }
    };


    match file_future.await {
        Ok(unique_filename) => HttpResponse::Ok().body(format!("File '{}' uploaded successfully", unique_filename)),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}





async fn download_file(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (node_id, file_id) = path.into_inner();
    let mut nodes = data.nodes.lock().unwrap();
    
    let node = match nodes.get_mut(&node_id) {
        Some(n) => n,
        None => return HttpResponse::NotFound().body("Node not found"),
    };

    // Create temporary directory for download
    let temp_dir = "temp_downloads";
    std::fs::create_dir_all(temp_dir).unwrap();
    let temp_path = format!("{}/{}", temp_dir, file_id);

    match node.retrieve_file(&file_id, &temp_path).await {
        Ok(_) => {
            // Read the file contents
            match std::fs::read(&temp_path) {
                Ok(contents) => {
                    std::fs::remove_file(&temp_path).ok(); // Clean up
                    HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .body(contents)
                }
                Err(e) => {
                    std::fs::remove_file(&temp_path).ok(); // Clean up
                    HttpResponse::InternalServerError().body(e.to_string())
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn delete_file(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (node_id, file_id) = path.into_inner();
    let mut nodes = data.nodes.lock().unwrap();
    
    let node = match nodes.get_mut(&node_id) {
        Some(n) => n,
        None => return HttpResponse::NotFound().body("Node not found"),
    };

    match node.delete_file(&file_id) {
        Ok(_) => HttpResponse::Ok().body("File deleted successfully"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn test_endpoint() -> HttpResponse {
    HttpResponse::Ok().body("Sunucu çalışıyor!")
}


// Server Configuration
pub async fn run_server() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        nodes: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}