use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::node::StorageNode;
use crate::proof_of_spacetime::periodic_check;


// Structures for token and storage management
#[derive(Clone, Serialize, Deserialize)]
//Bir kişinn stake ettigi token miktarı, storage limiti ve tokenin süresi
pub struct StorageToken {
    pub user_id: String,
    pub amount: u64,          // Amount of tokens staked
    pub storage_limit: u64,   // Storage limit in bytes
    pub expiry: u64,         // Unix timestamp
}

// #[derive(Clone, Serialize, Deserialize)]
// //Mevcut nodeun idsi, mevcut bos alanı, toplam alanı, sağlık durumu ve son kontrol edilme tarihi
// pub struct StorageNode {
//     node_id: String,
//     available_space: u64,
//     total_space: u64,
//     health_status: bool,
//     last_checked: u64,
// }

#[derive(Clone, Serialize, Deserialize)]
//Yüklenen dosyanın idsi, sahibinin idsi, boyutu, yüklendiği nodeun idsi, oluşturulma tarihi ve izinleri
pub struct FileMetadata {
    pub file_id: String,
    pub owner_id: String,
    pub size: u64,
    pub node_id: String,
    pub created_at: u64,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Serialize, Deserialize)]
//Bir kullanıcının dosyaya erişim izni
pub struct Permission {
    pub user_id: String,
    pub access_type: AccessType,
    pub expiry: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
    Delete,
    Admin,
}

pub struct ProgrammableBusinessEngine {
    pub tokens: HashMap<String, StorageToken>,
    nodes: HashMap<String, StorageNode>,
    files: HashMap<String, FileMetadata>,
    // Token rate is the amount of storage bytes per token
    token_rate: u64,  // Storage bytes per token
}

impl ProgrammableBusinessEngine {
    pub fn new(token_rate: u64) -> Self {
        ProgrammableBusinessEngine {
            tokens: HashMap::new(),
            nodes: HashMap::new(),
            files: HashMap::new(),
            token_rate,
        }
    }

    // Token Management
    //Kullanıcının stake ettiği token miktarı ve süresi. Depolama hakkı kazanır.
    pub fn stake_tokens(&mut self, user_id: &str, amount: u64) -> Result<StorageToken, String> {
        let storage_limit = amount * self.token_rate;
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + (30 * 24 * 60 * 60); // 30 days

        let token = StorageToken {
            user_id: user_id.to_string(),
            amount,
            storage_limit,
            expiry,
        };

        self.tokens.insert(user_id.to_string(), token.clone());
        Ok(token)
    }

    // Storage Management
    //Kullanıcının depolama alanı izni kontrol edilir.
    //Kullanıcının depolama hakkıyla, depolanmak istenen dosyanın boyutu karşılaştırılır.
    pub fn check_storage_allowance(&self, user_id: &str, required_space: u64) -> bool {
        if let Some(token) = self.tokens.get(user_id) {
            let current_usage = self.get_user_storage_usage(user_id);
            println!("Token storage limit: {:?}", token.storage_limit);

            token.storage_limit >= current_usage + required_space && token.expiry > SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        } else {
            false
        }
    }

    // Node Management
    //Yeni bir node kaydedilir.
    pub async fn register_node(&mut self, node_id: &str, total_space: u64) -> Result<(), String> {
        println!("Node registered: {:?}", node_id);
        let node = StorageNode::new(
            node_id.to_string(), 
            total_space,
        ).await.map_err(|e| e.to_string())?;
            // last_checked: SystemTime::now()
            //     .duration_since(UNIX_EPOCH)
            //     .unwrap()
            //     .as_secs(),
       
println!("Node available miktarı registered: {:?}", node.available_space);
        self.nodes.insert(node_id.to_string(), node);
        
        Ok(())
    }

    pub async fn get_all_nodes(&self) -> HashMap<String, StorageNode> {
        self.nodes.clone()
    }
    

    //Yüklenen dosyanın boyutuna göre uygun node seçilir.
    // Çıktı olarak node id döner.
    pub fn assign_node(&self, file_size: u64) -> Option<String> {
        println!("File size: {:?}", file_size);
        println!("Nodes: {:?}", self.nodes);
        self.nodes
            .iter()
            .find(|(_, node)| node.available_space >= file_size && node.health_status)
            .map(|(node_id, _)| node_id.clone())
    }

    // Access Control
    //Dosyaya erişim kontrolü yapılır.
    //Dosya izinleri kontrol edilir.
    //Dosya yüklenildiğinde metadata oluşturulur.
    pub fn create_file_entry(&mut self, 
        file_id: &str, 
        owner_id: &str, 
        size: u64, 
        node_id: &str
    ) -> Result<(), String> {
        if !self.check_storage_allowance(owner_id, size) {
            return Err("Insufficient storage allowance".to_string());
        }

        let metadata = FileMetadata {
            file_id: file_id.to_string(),
            owner_id: owner_id.to_string(),
            size,
            node_id: node_id.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            permissions: vec![Permission {
                user_id: owner_id.to_string(),
                access_type: AccessType::Admin,
                expiry: None,
            }],
        };

        self.files.insert(file_id.to_string(), metadata);
        Ok(())
    }

    //Dosyaya erişim kontrolü yapılır.
    //Bir kullanıcının belli br dosyaya erişim izni kontrol edilir.
    pub fn check_access(&self, user_id: &str, file_id: &str, access_type: AccessType) -> bool {
        if let Some(file) = self.files.get(file_id) {
            file.permissions.iter().any(|perm| {
                perm.user_id == user_id 
                && matches!(perm.access_type, AccessType::Admin) 
                && perm.expiry.map_or(true, |exp| exp > SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs())
            })
        } else {
            false
        }
    }

    // Helper functions
    //Kullanıcının depolama alanı kullanımı hesaplanır. Yüklediği tüm dosyaların boyutu toplanır.
    fn get_user_storage_usage(&self, user_id: &str) -> u64 {
        self.files
            .values()
            .filter(|file| file.owner_id == user_id)
            .map(|file| file.size)
            .sum()
    }

    // // Smart Contract Integration
    // //Kullanıcının stake ettiği token miktarı kontrol edilir akıllı kontrat üzerinden.
    // pub fn verify_smart_contract_stake(&self, contract_address: &str, user_id: &str) -> Result<u64, String> {
     
    //     Ok(100) // Mock token amount
    // }

    // Node Health Monitoring
    pub fn update_node_health(&mut self, node_id: &str, is_healthy: bool) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.health_status = is_healthy;
            node.last_checked = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }
    pub fn get_node_mut (& mut self ,node_id: &str) -> Option<&mut StorageNode> {
        self.nodes.get_mut(node_id)
    }

}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_staking() {
        let mut pbe = ProgrammableBusinessEngine::new(1_000_000); // 1MB per token
        let result = pbe.stake_tokens("user1", 100);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.storage_limit, 100_000_000); // 100MB
    }

    #[test]
    fn test_node_assignment() {
        let mut pbe = ProgrammableBusinessEngine::new(1_000_000);
        futures::executor::block_on(pbe.register_node("node1", 1_000_000_000)).unwrap(); // 1GB
        let assigned_node = pbe.assign_node(100_000_000); // 100MB
        assert!(assigned_node.is_some());
        assert_eq!(assigned_node.unwrap(), "node1");
    }
}
