mod encryption;
mod storage;
mod blockchain;
mod models;

use crate::models::FileMetadata;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4"; // Yüklenecek dosya
    let node_storage_path = "./node_storage"; // Node belleği için klasör

    // Dosyayı şifrele ve yerel node'a kaydet
    let encrypted_file_path = storage::store_file(file_path, node_storage_path)?;

    // Dosya hash'ini hesapla
    let file_hash = storage::calculate_file_hash(&encrypted_file_path)?;

    // Dosya Metadata'sını Oluştur
    let metadata = FileMetadata {
        file_name: file_path.to_string(),
        file_hash: file_hash.clone(),
        node_address: "http://127.0.0.1:8000/download".to_string(),
    };

    // Metadata'yı Blockchain'e Gönder
    blockchain::send_metadata_to_blockchain(&metadata)?;

    // Dosyayı İndir ve Şifresini Çöz
    let output_path = "downloaded_example_file.txt";
    storage::download_file(&encrypted_file_path, output_path)?;

    // Dosyayı Sil
    storage::delete_file(&encrypted_file_path)?;

    println!("Dosya yükleme, indirme ve silme işlemleri tamamlandı.");
    Ok(())
}
