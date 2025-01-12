use std::fs;
use std::path::Path;

pub struct Node {
    pub id: String,
    pub storage_path: String,
    pub total_space: u64,   // Toplam kapasite (MB)
    pub used_space: u64,    // Kullanılmış kapasite (MB)
}

impl Node {
    /// Yeni bir node oluştur
    pub fn new(id: &str, storage_path: &str, total_space: u64) -> Self {
        fs::create_dir_all(storage_path).expect("Depolama dizini oluşturulamadı");
        Self {
            id: id.to_string(),
            storage_path: storage_path.to_string(),
            total_space,
            used_space: 0,
        }
    }

    /// Node'un kalan boş alanını döndür
    pub fn available_space(&self) -> u64 {
        self.total_space - self.used_space
    }

    /// Dosya boyutuna göre yeterli alan olup olmadığını kontrol et
    pub fn has_space(&self, file_size: u64) -> bool {
        self.available_space() >= file_size
    }

    /// Depolama dizinindeki toplam kullanılan alanı hesapla
    pub fn calculate_used_space(&mut self) {
        let dir = Path::new(&self.storage_path);
        let mut size = 0;
        if dir.exists() {
            for entry in fs::read_dir(dir).expect("Depolama dizini okunamadı") {
                let entry = entry.expect("Dizin girişine erişilemedi");
                let metadata = entry.metadata().expect("Metadata alınamadı");
                if metadata.is_file() {
                    size += metadata.len();
                }
            }
        }
        self.used_space = size / (1024 * 1024); // MB cinsine çevir
    }
}
