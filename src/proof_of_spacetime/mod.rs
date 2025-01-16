use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, Duration};
use std::fs::File;
use std::io::{Read, Seek, Write};
use tokio::time::{sleep, Duration as TokioDuration};

const CHALLENGE_TIMEOUT: Duration = Duration::new(20 * 60, 0);  // 20 minutes

// Generates a random word.
fn generate_random_challenge() -> String {
    let mut range = thread_rng();
    
    // Generates a random word of 10 characters
    // b is used to convert byte to char
    let challenge_word: String = (0..10)
        .map(|_| range.gen_range(b'a'..=b'z') as char)
        .collect();
    challenge_word
}

// Reads a random part of the file.
fn get_random_file_part(file_path: &str, byte_count: usize) -> Result<Vec<u8>, std::io::Error> {

    let mut file = File::open(file_path)?;
    // Generates a random number.
    let mut range = thread_rng();
    let file_size = std::fs::metadata(file_path)?.len() as usize;
    
    // Starts from a random position in the file.
    let start_byte = range.gen_range(0..file_size - byte_count);
    let mut buffer = vec![0; byte_count];
    file.seek(std::io::SeekFrom::Start(start_byte as u64))?;
    file.read_exact(&mut buffer)?;

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
fn respond_to_challenge(file_path: &str) -> Result<Vec<u8>, String> {
    let challenge_word = generate_random_challenge();
    println!("Challenge Word: {}", challenge_word);
    
    // Gets a random part of the file.
    let file_part = match get_random_file_part(file_path, 100) {
        Ok(part) => part,
        Err(_) => return Err("Failed to read data from file!".to_string()),
    };

    // Generates the hash
    let response_hash = generate_hash(&file_part, &challenge_word);
    println!("Generated Hash: {:?}", response_hash);
    
    // This hash will be sent back to the validator
    Ok(response_hash)
}

// Challenge process and time check
fn proof_of_spacetime(file_path: &str) {
    let start_time = SystemTime::now();

    // Get response to the challenge
    match respond_to_challenge(file_path) {
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
#[tokio::main]
pub async fn periodic_check(file_path: &str) {
    loop {
        proof_of_spacetime(file_path);
        sleep(TokioDuration::from_secs(30)).await;  // 30 seconds wait
    }
}

// fn main() {
//     // Provide the file path and start periodic challenge validation
//     let file_path = "path/to/storage/encrypted_file.dat";  // File path
//     periodic_check(file_path);  // Start periodic check
// }
