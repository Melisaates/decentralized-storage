#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    pub file_name: String,
    pub file_hash: String,
    pub node_address: String,
}
