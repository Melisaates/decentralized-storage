use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::http::header;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, RwLock};
use crate::encryption::{encrypt_file_path, decrypt_file_path, generate_key_iv};
use crate::proof_of_spacetime::proof_of_spacetime;
use crate::token::check_access_token; 

// jwt ile token oluşturma ve doğrulama işlemleri
//  Dosya yükleme, indirme, doğrulama ve silme işlemleri için API'ler


#[derive(Deserialize)]
pub struct FileRequest {
    pub file_path: String,
}

#[derive(Serialize)]
pub struct FileResponse {
    pub message: String,
}

// Dosya yükleme işlemi
async fn upload_file(file_req: web::Json<FileRequest>, token: Option<String>) -> impl Responder {
    // Token doğrulaması başlık üzerinden yapılacak
    if let Some(token) = token {
        if !check_access_token(&token) {
            return HttpResponse::Unauthorized().json(FileResponse {
                message: "Geçersiz token.".to_string(),
            });
        }
    }

    let file_path = &file_req.file_path;

    // Dosyayı şifrele ve depola
    match encrypt_file(file_path, "my_path", &generate_key_iv().0, &generate_key_iv().1) {
        Ok(_) => HttpResponse::Ok().json(FileResponse {
            message: "Dosya başarıyla yüklendi ve şifrelendi.".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(FileResponse {
            message: format!("Dosya yüklenirken hata oluştu: {}", e),
        }),
    }
}   

// Dosya indirme işlemi
async fn download_file(file_req: web::Json<FileRequest>, token: Option<String>) -> impl Responder {
    // Token doğrulaması başlık üzerinden yapılacak
    if let Some(token) = token {
        if !check_access_token(&token) {
            return HttpResponse::Unauthorized().json(FileResponse {
                message: "Geçersiz token.".to_string(),
            });
        }
    }

    let file_path = &file_req.file_path;

    // Dosyayı çöz ve geri gönder
    match decrypt_file_path(file_path, "my_path", &generate_key_iv().0, &generate_key_iv().1) {
        Ok(_) => HttpResponse::Ok().json(FileResponse {
            message: "Dosya başarıyla indirildi ve şifresi çözüldü.".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(FileResponse {
            message: format!("Dosya indirilirken hata oluştu: {}", e),
        }),
    }
}

// Proof of SpaceTime doğrulaması
async fn validate_file(file_req: web::Json<FileRequest>) -> impl Responder {
    let file_path = &file_req.file_path;

    // Proof of SpaceTime doğrulaması
    match proof_of_spacetime(file_path) {
        Ok(_) => HttpResponse::Ok().json(FileResponse {
            message: "Dosya doğrulandı.".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(FileResponse {
            message: format!("Dosya doğrulama işlemi başarısız: {}", e),
        }),
    }
}

// Dosya silme işlemi
async fn delete_file(file_req: web::Json<FileRequest>, token: Option<String>) -> impl Responder {
    // Token doğrulaması başlık üzerinden yapılacak
    if let Some(token) = token {
        if !check_access_token(&token) {
            return HttpResponse::Unauthorized().json(FileResponse {
                message: "Geçersiz token.".to_string(),
            });
        }
    }

    let file_path = &file_req.file_path;

    // Dosyanın var olup olmadığını kontrol et
    if !fs::metadata(file_path).is_ok() {
        return HttpResponse::NotFound().json(FileResponse {
            message: "Dosya bulunamadı.".to_string(),
        });
    }

    // Dosyayı sil
    match fs::remove_file(file_path) {
        Ok(_) => HttpResponse::Ok().json(FileResponse {
            message: "Dosya başarıyla silindi.".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(FileResponse {
            message: format!("Dosya silinirken hata oluştu: {}", e),
        }),
    }
}

// API'yi başlatan ana fonksiyon
pub async fn start_api() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(Arc::new(RwLock::new(()))))  
            .wrap(Logger::default())  
            .route("/upload", web::post().to(upload_file))
            .route("/download", web::post().to(download_file))
            .route("/validate", web::post().to(validate_file))
            .route("/delete", web::post().to(delete_file)) 
    })
    .bind("127.0.0.1:8080")? // API'nin dinleyeceği port
    .run()
    .await
}
