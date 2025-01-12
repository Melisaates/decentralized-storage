use crate::models::FileMetadata;
use reqwest::Client;

pub fn send_metadata_to_blockchain(metadata: &FileMetadata) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let blockchain_api_url = "https://bsc-smart-contract-endpoint.com/metadata";

    let response = client.post(blockchain_api_url)
        .json(metadata)
        .send()?;

    if response.status().is_success() {
        println!("Metadata successfully sent to blockchain: {:?}", metadata);
    } else {
        eprintln!("Failed to send metadata to blockchain: {:?}", response.text()?);
    }

    Ok(())
}
