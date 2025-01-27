use crate::p2p::{Network};
use crate::node::Node;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::fs::{metadata, File};
use std::io::{Read, Seek};
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, Duration as TokioDuration};

const CHALLENGE_TIMEOUT: Duration = Duration::new(30, 0);

// Random bir challenge kelimesi oluşturuluyor
fn generate_random_challenge() -> String {
    let mut range = thread_rng();
    (0..10)
        .map(|_| range.gen_range(b'a'..=b'z') as char)
        .collect()
}

fn get_random_file_part(node: &Node, byte_count: usize) -> Result<Vec<u8>, String> {
    // Okunabilir dosyaları almak
    let files: Vec<String> = std::fs::read_dir(&node.storage_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                // Dosyanın okunabilir olup olmadığı kontrol ediliyor
                if let Ok(metadata) = entry.metadata() {
                    if !metadata.is_dir() && metadata.permissions().readonly() == false {
                        return entry.file_name().into_string().ok();
                    }
                }
            }
            None
        })
        .collect();

    if files.is_empty() {
        return Err("No readable files in storage".to_string());
    }

    let mut range = thread_rng();
    let random_file = files.choose(&mut range)
        .ok_or("Failed to select file")?;

    let file_path = format!("{}/{}", node.storage_path, random_file);

    let mut file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
    
    let file_size = metadata(&file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len() as usize;

    let byte_count = std::cmp::min(byte_count, file_size);
    let start_byte = thread_rng().gen_range(0..file_size - byte_count);

    let mut buffer = vec![0; byte_count];
    file.seek(std::io::SeekFrom::Start(start_byte as u64))
        .map_err(|e| format!("Failed to seek in file: {}", e))?;

    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read data from file: {}", e))?;

    Ok(buffer)
}

// Hash üretme fonksiyonu (değiştirilmedi)
fn generate_hash(file_part: &[u8], challenge_word: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(file_part);
    hasher.update(challenge_word.as_bytes());
    hasher.finalize().to_vec()
}

async fn respond_to_challenge(network: &Network) -> Result<Vec<u8>, String> {
    // Ağdaki düğümleri alın
    let nodes = network.get_nodes().await;

    if nodes.is_empty() {
        return Err("Ağda kullanılabilir düğüm yok".to_string());
    }

    let mut rng = thread_rng();
    let random_node = nodes.choose(&mut rng)
        .ok_or("Rasgele bir düğüm seçilemedi")?;

    
    // Dosyayı almak için dosya yolunu kontrol edin
    let file_part = match get_random_file_part(random_node, 100) {
        Ok(part) => part,
        Err(_) => return Err(format!("Dosya alınırken hata oluştu. Node: {:?}", random_node)),
    };

    // Challenge kelimesini oluşturun
    let challenge_word = generate_random_challenge();

    // Hash'i oluşturun
    let response_hash = generate_hash(&file_part, &challenge_word);

    Ok(response_hash)
}


// Proof of spacetime için yeni işlev
async fn proof_of_spacetime(network: &Network) {
    let start_time = SystemTime::now();

    match respond_to_challenge(network).await {
        Ok(response_hash) => {
            let elapsed = SystemTime::now().duration_since(start_time).unwrap();
            if elapsed <= CHALLENGE_TIMEOUT {
                println!("Challenge passed! Hash: {:?}", response_hash);
            } else {
                println!("Challenge failed: Timeout.");
            }
        }
        Err(err) => {
            println!("Error while processing challenge: {}", err);
        }
    }
}

// Periyodik kontrol fonksiyonu
pub async fn periodic_check(network: &Network) {
    loop {
        proof_of_spacetime(network).await;
        sleep(TokioDuration::from_secs(30)).await;
    }
}



/*You said:
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::fs::{metadata, read_dir, File};
use std::io::{Read, Seek};
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, Duration as TokioDuration};

const CHALLENGE_TIMEOUT: Duration = Duration::new(30, 0); // 20 minutes=20*60

// Generates a random word.
fn generate_random_challenge() -> String {
    let mut range = thread_rng();

    // Generates a random word of 10 characters
    let challenge_word: String = (0..10)
        .map(|_| range.gen_range(b'a'..=b'z') as char)
        .collect();
    challenge_word
}

// Reads a random part of the file.
fn get_random_file_part(file_path: &str, byte_count: usize) -> Result<Vec<u8>, String> {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };

    let file_size = match metadata(file_path) {
        Ok(meta) => meta.len() as usize,
        Err(e) => return Err(format!("Failed to get file metadata: {}", e)),
    };

    // Handle small files by adjusting byte count dynamically
    let byte_count = std::cmp::min(byte_count, file_size);

    let mut range = thread_rng();
    let start_byte = range.gen_range(0..file_size - byte_count);

    let mut buffer = vec![0; byte_count];

    // Seek to the random starting byte and read
    if let Err(e) = file.seek(std::io::SeekFrom::Start(start_byte as u64)) {
        return Err(format!("Failed to seek in file: {}", e));
    }

    if let Err(e) = file.read_exact(&mut buffer) {
        return Err(format!("Failed to read data from file: {}", e));
    }
    
    println!("buffer: {:?}", buffer);

    Ok(buffer)
}

// Performs the hash operation.
fn generate_hash(file_part: &[u8], challenge_word: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(file_part);
    hasher.update(challenge_word.as_bytes());

    hasher.finalize().to_vec()
}

// Function that responds to the challenge.
fn respond_to_challenge(storage_path: &str) -> Result<Vec<u8>, String> {
    // List all files in the storage path and choose one randomly
    let paths = match read_dir(storage_path) {
        Ok(paths) => paths,
        Err(e) => return Err(format!("Failed to read storage directory: {}", e)),
    };

    let mut files: Vec<String> = Vec::new();

    for path in paths {
        match path {
            Ok(entry) => {
                let file_name = entry.file_name().into_string().unwrap();
                files.push(file_name);
            }
            Err(_) => continue,
        }
    }

    if files.is_empty() {
        return Err("No files found in the storage directory.".to_string());
    }

    // Choose a random file
    let mut range = thread_rng();
    let random_file = files.choose(&mut range).unwrap();

    println!("Selected File: {}", random_file);

    // Get a random part of the selected file
    let file_part = match get_random_file_part(&format!("{}/{}", storage_path, random_file), 100) {
        Ok(part) => part,
        Err(err) => return Err(err),
    };

    // Generate the hash
    let challenge_word = generate_random_challenge();
    println!("Challenge Word: {}", challenge_word);

    let response_hash = generate_hash(&file_part, &challenge_word);
    println!("Generated Hash: {:?}", response_hash);

    // Return the hash
    Ok(response_hash)
}

// Challenge process and time check
fn proof_of_spacetime(storage_path: &str) {
    let start_time = SystemTime::now();

    // Get response to the challenge
    match respond_to_challenge(storage_path) {
        Ok(response_hash) => {
            let elapsed = SystemTime::now().duration_since(start_time).unwrap();
            if elapsed <= CHALLENGE_TIMEOUT {
                // If responded in time, validation is successful
                println!("Challenge passed! Hash: {:?}", response_hash);
            } else {
                println!("Challenge failed: Timeout.");
            }
        }
        Err(err) => {
            println!("Error while processing challenge: {}", err);
        }
    }
}

// Function that periodically calls proof_of_spacetime
pub async fn periodic_check(storage_path: &str) {
    loop {
        proof_of_spacetime(storage_path);
        sleep(TokioDuration::from_secs(30)).await; // 1200 seconds wait
    }
} */