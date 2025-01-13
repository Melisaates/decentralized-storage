use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use ethers::types::Address;

#[derive(Clone, Serialize, Deserialize,Debug)]
pub struct Node {
    pub id: String,
    pub storage_path: String,// Depolama yolu
    pub available_space: u64,
}

// P2P ağını temsil eden Network yapısı
// Ağdaki node'ları depolamak için HashMap kullandım
// Mutex ile aynı anda birden fazla iş parçacığının erişimini engelledim
pub struct Network {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.lock().await;
        nodes.values().cloned().collect()
    }

    pub async fn add_node(&self, node: Node) {
        let mut nodes = self.nodes.lock().await;
        nodes.insert(node.id.clone(), node);
    }

    pub async fn start_server(&self, addr: SocketAddr) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server started on {:?}", addr);
    
        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let nodes = self.nodes.clone();
                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];
                        match socket.read(&mut buffer).await {
                            Ok(bytes_read) if bytes_read > 0 => {
                                if let Ok(message) = serde_json::from_slice::<Node>(&buffer[..bytes_read]) {
                                    nodes.lock().await.insert(message.id.clone(), message.clone());
                                    println!("Node added: {:?}", message);
                                } else {
                                    eprintln!("Failed to deserialize Node from received data");
                                }
                            }
                            Ok(_) => {
                                println!("No data received");
                            }
                            Err(e) => {
                                eprintln!("Failed to read from socket: {:?}", e);
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {:?}", e);
                }
            }
        }
    }
    
}

pub fn find_available_node(file_size: u64, nodes: &[Node]) -> Option<Node> {
    nodes.iter().find(|&node| node.available_space >= file_size).cloned()
}
