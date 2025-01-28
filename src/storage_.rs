use sha2::{Sha256, Digest};
use aes::{Aes256, BlockEncrypt, BlockDecrypt};
use aes::cipher::{NewBlockCipher, BlockCipher};
//use aes::block_cipher::generic_array::GenericArray;
use bytes::{Bytes, Buf};
use std::collections::HashMap;
use std::path::Path;

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Debug, Clone)]
pub struct File {
    pub id: String,
    pub data: Vec<Bytes>, // Parçalara ayrılmış veri
    pub file_type: String, // Dosya türü (örn: png, jpg, mp4)
}

#[derive(Debug)]
pub struct Storage {
    node_id: String,
    files: HashMap<String, File>,
}

impl Storage {
    pub fn new(node_id: &str) -> Self {
        Storage {
            node_id: node_id.to_string(),
            files: HashMap::new(),
        }
    }

    // Dosya yükleme (parçalı)
    pub fn store_file(&mut self, file: File) {
        let hash = self.compute_hash(&file);
        self.files.insert(file.id.clone(), file);
        println!("Stored file with hash: {} on node {}", hash, self.node_id);
    }

    // Dosyayı indir (parçalı)
    pub fn download_file(&self, file_name: &str) -> Option<Vec<Bytes>> {
        if let Some(file) = self.files.get(file_name) {
            Some(file.data.clone()) // Dosya parçalarını döndürür
        } else {
            None
        }
    }

    // Dosya silme (parçalı)
    pub fn delete_file(&mut self, file_name: &str) {
        self.files.remove(file_name);
        println!("File '{}' has been deleted from node {}", file_name, self.node_id);
    }

    // Hash hesaplama (dosyanın tamamı için)
    pub fn compute_hash(&self, file: &File) -> String {
        let mut hasher = Sha256::new();
        for chunk in &file.data {
            hasher.update(chunk);
        }
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    // // Dosya verisini şifreleme
    // pub fn encrypt_data(&self, data: &Bytes, key: &[u8; 32]) -> Bytes {
    //     let cipher = Aes256::new(GenericArray::from_slice(key));
    //     let mut encrypted_data = data.clone();
    //     cipher.encrypt_block(&mut encrypted_data);
    //     encrypted_data
    // }

    // // Şifreli veriyi çözme
    // pub fn decrypt_data(&self, data: &Bytes, key: &[u8; 32]) -> Bytes {
    //     let cipher = Aes256::new(GenericArray::from_slice(key));
    //     let mut decrypted_data = data.clone();
    //     cipher.decrypt_block(&mut decrypted_data);
    //     decrypted_data
    // }

    // Dosya parçalara ayırma
    pub fn split_file_into_chunks(file_data: Bytes) -> Vec<Bytes> {
        let mut chunks = Vec::new();
        let mut offset = 0;
        while offset < file_data.len() {
            let end = std::cmp::min(offset + CHUNK_SIZE, file_data.len());
            chunks.push(file_data.slice(offset..end));
            offset = end;
        }
        chunks
    }

    // Dosya türünü kontrol etme (örneğin, mp4, png, jpg)
    pub fn check_file_type(&self, path: &str) -> String {
        let extension = Path::new(path).extension().unwrap_or_default().to_str().unwrap_or_default();
        match extension {
            "png" | "jpg" | "jpeg" => "image".to_string(),
            "mp4" => "video".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

