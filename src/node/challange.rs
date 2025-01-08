use sha2::{Sha256, Digest};
use chrono::{Utc, Duration};
use tokio::time::{sleep, Duration as TokioDuration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Her bir node'un durumu (challenge cevabı ve zamanlayıcı)
struct NodeState {
    last_challenge_time: chrono::DateTime<Utc>,
    last_response: Option<String>,
    is_valid: bool,
}

pub struct ValidatorNetwork {
    nodes: Arc<Mutex<HashMap<String, NodeState>>>,
}

impl ValidatorNetwork {
    pub fn new() -> Self {
        ValidatorNetwork {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn challenge_node(&self, node_id: &str) {
        let challenge = self.generate_challenge();
        println!("Challenge gönderildi: {}", challenge);

        // Zamanlayıcı: 20 dakika içinde cevap bekle
        let timeout = TokioDuration::from_secs(20 * 60);
        sleep(timeout).await;

        // Node cevabını kontrol et
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(node_state) = nodes.get_mut(node_id) {
            if let Some(response) = &node_state.last_response {
                let hashed_response = self.hash_response(response);
                println!("Node cevabı: {}", hashed_response);

                // Yanıt doğrulaması
                if hashed_response == self.hash_response(&challenge) {
                    node_state.is_valid = true;
                    println!("Node geçerli.");
                } else {
                    node_state.is_valid = false;
                    println!("Node hatalı: Yanıt eşleşmedi.");
                }
            } else {
                node_state.is_valid = false;
                println!("Node cevabı alınamadı, node hatalı.");
            }
        }
    }

    fn generate_challenge(&self) -> String {
        // Örnek challenge: Zaman damgası ile birleşmiş rastgele bir değer
        let random_value = Utc::now().timestamp_millis();
        format!("Challenge_{}", random_value)
    }

    fn hash_response(&self, response: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(response);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn add_node(&self, node_id: String) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node_id.clone(), NodeState {
            last_challenge_time: Utc::now(),
            last_response: None,
            is_valid: true,
        });
    }

    pub fn set_node_response(&self, node_id: &str, response: String) {
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(node_state) = nodes.get_mut(node_id) {
            node_state.last_response = Some(response);
        }
    }
}

#[tokio::main]
async fn main() {
    let validator_network = ValidatorNetwork::new();
    let node_id = "node1".to_string();

    // Yeni node ekle
    validator_network.add_node(node_id.clone());

    // Challenge gönder
    validator_network.challenge_node(&node_id).await;

    // Node'a cevap set et (örnek)
    validator_network.set_node_response(&node_id, "Challenge_1654078600000".to_string());

    // Challenge tekrar gönder (doğrulama)
    validator_network.challenge_node(&node_id).await;
}
