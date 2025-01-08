use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::http::header::ContentType;
use serde::{Serialize, Deserialize};
use crate::storage::{check_storage_availability, save_file};

mod storage;

#[derive(Deserialize)]
struct FileUpload {
    file_name: String,
    file_data: String, // Base64 encoded file data
}

#[derive(Serialize)]
struct ResponseData {
    status: String,
    message: String,
}

// Dosya yükleme uç noktası
async fn upload_file(info: web::Json<FileUpload>) -> impl Responder {
    let decoded_data = base64::decode(&info.file_data).unwrap();
    
    // Depolama alanı kontrolü
    match check_storage_availability(decoded_data.len() as u64).await {
        Ok(true) => {
            // Dosya depolama işlemi
            match save_file(decoded_data, &info.file_name).await {
                Ok(file_info) => {
                    HttpResponse::Ok().json(ResponseData {
                        status: "success".to_string(),
                        message: format!("Dosya başarıyla yüklendi: ID {}", file_info.id),
                    })
                },
                Err(err) => {
                    HttpResponse::InternalServerError().json(ResponseData {
                        status: "error".to_string(),
                        message: err,
                    })
                }
            }
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(ResponseData {
                status: "error".to_string(),
                message: err,
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Sunucu başlatma
    HttpServer::new(|| {
        App::new()
            .route("/upload", web::post().to(upload_file))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
