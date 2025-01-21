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
}
