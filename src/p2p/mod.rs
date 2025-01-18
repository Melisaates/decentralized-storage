use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub storage_path: String, // Storage path
    pub available_space: u64,
}

// Network structure representing the P2P network
// I used HashMap to store the nodes in the network
// Used Mutex to prevent simultaneous access by multiple threads
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

    // Function to start the P2P network
    // Reads data from the socket, converts it to a Node structure, and adds it to the HashMap
    pub async fn start_server(&self, addr: SocketAddr) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server started on {:?}", addr);

        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let nodes = self.nodes.clone();
                    // Asynchronously read data from the socket
                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];
                        match socket.read(&mut buffer).await {
                            Ok(bytes_read) if bytes_read > 0 => {
                                // Convert the received data to a Node structure
                                if let Ok(message) =
                                    serde_json::from_slice::<Node>(&buffer[..bytes_read])
                                {
                                    nodes
                                        .lock()
                                        .await
                                        .insert(message.id.clone(), message.clone());
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

// Function to find a node with enough available space for a file
pub fn find_available_node(file_size: u64, nodes: &[Node]) -> Option<Node> {
    nodes
        .iter()
        .find(|&node| node.available_space >= file_size)
        .cloned()
}
