use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, Duration};
use std::fs::File;
use std::io::{Read, Write};

const CHALLENGE_TIMEOUT: Duration = Duration::new(20 * 60, 0);  // 20 dakika

// Rastgele kelime üreten fonksiyon
fn generate_random_challenge() -> String {
    let mut rng = thread_rng();
    let challenge_word: String = (0..10)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect();
    challenge_word
}

// Dosyanın rastgele bir parçasını okuyan fonksiyon (Örnek olarak 100 byte)
fn get_random_file_part(file_path: &str, byte_count: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut rng = thread_rng();
    let file_size = std::fs::metadata(file_path)?.len() as usize;
    
    // Dosyanın rastgele bir yerinden başla
    let start_byte = rng.gen_range(0..file_size - byte_count);
    let mut buffer = vec![0; byte_count];
    file.seek(std::io::SeekFrom::Start(start_byte as u64))?;
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}

// Hash işlemi (dosya parçası ve challenge kelimesi)
fn generate_hash(file_part: &[u8], challenge_word: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(file_part);
    hasher.update(challenge_word.as_bytes());
    hasher.finalize().to_vec()
}

// Challenge işlemine yanıt veren fonksiyon
fn respond_to_challenge(file_path: &str) -> Result<Vec<u8>, String> {
    let challenge_word = generate_random_challenge();
    println!("Challenge Word: {}", challenge_word);
    
    // Dosyanın rastgele bir parçasını al
    let file_part = match get_random_file_part(file_path, 100) {
        Ok(part) => part,
        Err(_) => return Err("Dosyadan veri alınamadı!".to_string()),
    };

    // Hash'i oluştur
    let response_hash = generate_hash(&file_part, &challenge_word);
    println!("Generated Hash: {:?}", response_hash);
    
    // Bu hash validator'a geri gönderilecek
    Ok(response_hash)
}

// Challenge işlemi ve zaman denetimi
fn proof_of_spacetime(file_path: &str) {
    let start_time = SystemTime::now();

    // Challenge'a yanıt al
    match respond_to_challenge(file_path) {
        Ok(response_hash) => {
            let elapsed = SystemTime::now().duration_since(start_time).unwrap();
            if elapsed <= CHALLENGE_TIMEOUT {
                // Zamanında cevap verildiyse, doğrulama başarılı
                println!("Challenge passed! Hash: {:?}", response_hash);
            } else {
                println!("Challenge failed: Zaman aşımı.");
            }
        }
        Err(err) => {
            println!("Error while processing challenge: {}", err);
        }
    }
}

// fn main() {
//     // Dosya yolunu ver ve challenge doğrulamasını başlat
//     let file_path = "path/to/storage/encrypted_file.dat";  // Dosya yolu
//     proof_of_spacetime(file_path);
// }
