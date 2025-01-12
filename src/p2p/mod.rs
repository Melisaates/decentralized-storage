use std::collections::HashMap;

use crate::node::Node;


pub struct Network {
    pub nodes: HashMap<String, Node>, // Node ID -> Node
}

impl Network {
    /// Yeni bir P2P ağı oluştur
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Ağa bir node ekle
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Yeterli alana sahip bir node bul
    pub fn find_suitable_node(&self, file_size: u64) -> Option<&Node> {
        self.nodes.values().find(|node| node.has_space(file_size))
    }
}
