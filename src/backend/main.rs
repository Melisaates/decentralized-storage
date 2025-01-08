use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::http::header::ContentType;
use std::fs::File;
use std::path::Path;
use std::io::{Write};
use serde::{Serialize, Deserialize};
use crate::encryption::{encrypt_file, generate_key};

mod encryption;

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

// Dosya yükleme ve şifreleme işlemi
async fn upload_file(info: web::Json<FileUpload>) -> impl Responder {
    // Anahtar ve IV üretimi
    let encryption_key = generate_key();
    
    // Base64'ten dosya verisine dönüştürme
    let decoded_data = base64::decode(&info.file_data).unwrap();

    // Dosyayı geçici bir dosyaya kaydet
    let temp_file_path = format!("./uploads/{}", info.file_name);
    let mut temp_file = File::create(&temp_file_path).unwrap();
    temp_file.write_all(&decoded_data).unwrap();

    // Dosyayı şifrele
    match encrypt_file(&temp_file_path, &encryption_key) {
        Ok(encrypted_data) => {
            // Şifrelenmiş veriyi dosyaya kaydet
            let encrypted_file_path = format!("./uploads/encrypted_{}", info.file_name);
            let mut encrypted_file = File::create(encrypted_file_path).unwrap();
            encrypted_file.write_all(encrypted_data.as_bytes()).unwrap();

            // Orijinal dosyayı sil
            std::fs::remove_file(&temp_file_path).unwrap();

            // Kullanıcıya yanıt gönder
            HttpResponse::Ok().json(ResponseData {
                status: "success".to_string(),
                message: "Dosya başarıyla şifrelendi ve yüklendi.".to_string(),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ResponseData {
                status: "error".to_string(),
                message: "Şifreleme işlemi sırasında hata oluştu.".to_string(),
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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
