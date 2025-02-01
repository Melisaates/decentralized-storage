use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::fs::{metadata, read_dir, File};
use std::io::{Read, Seek};
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, Duration as TokioDuration};

use crate::node::StorageNode;
use crate::storage_::Storage;

const CHALLENGE_TIMEOUT: Duration = Duration::new(30, 0); // 30 seconds timeout

// Generates a random word.
fn generate_random_challenge() -> String {
    let mut range = thread_rng();
    (0..10).map(|_| range.gen_range(b'a'..=b'z') as char).collect()
}

// Reads a random part of the file.
fn get_random_file_part(file_path: &str, byte_count: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let file_size = metadata(file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?.len() as usize;
    let byte_count = std::cmp::min(byte_count, file_size);
    let mut range = thread_rng();
    let start_byte = range.gen_range(0..file_size - byte_count);
    let mut buffer = vec![0; byte_count];
    file.seek(std::io::SeekFrom::Start(start_byte as u64)).map_err(|e| format!("Failed to seek in file: {}", e))?;
    file.read_exact(&mut buffer).map_err(|e| format!("Failed to read data from file: {}", e))?;
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
    let paths = read_dir(storage_path).map_err(|e| format!("Failed to read storage directory: {}", e))?;
    let mut files: Vec<String> = paths.filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().unwrap())).collect();
    if files.is_empty() {
        return Err("No files found in the storage directory.".to_string());
    }
    let mut range = thread_rng();
    let random_file = files.choose(&mut range).ok_or("No valid files available.".to_string())?;
    let file_part = get_random_file_part(&format!("{}/{}", storage_path, random_file), 100)?;
    let challenge_word = generate_random_challenge();
    let response_hash = generate_hash(&file_part, &challenge_word);
    Ok(response_hash)
}

// Checks proof-of-spacetime for a given node
fn proof_of_spacetime(node: &StorageNode) {
    let start_time = SystemTime::now();
    match respond_to_challenge(&node.storage_path) {
        Ok(response_hash) => {
            let elapsed = SystemTime::now().duration_since(start_time).unwrap();
            if elapsed <= CHALLENGE_TIMEOUT {
                println!("Node {} passed challenge! Hash: {:?}", node.node_id, response_hash);
            } else {
                println!("Node {} failed challenge: Timeout.", node.node_id);
            }
        }
        Err(err) => {
            println!("Node {} error while processing challenge: {}", node.node_id, err);
        }
    }
}

// Periodically checks proof-of-spacetime for all nodes
pub async fn periodic_check(nodes: Vec<StorageNode>) {
    loop {
        for node in &nodes {
            proof_of_spacetime(node);
        }
        sleep(TokioDuration::from_secs(30)).await; // Wait 30 seconds before next check
    }
}
