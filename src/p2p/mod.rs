use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::task;
use tokio::time::{sleep, Duration};



#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub storage_path: String, // Storage path
    pub available_space: u64,
    pub address: String, // Node'un ağ adresi
}

// Ağ yapısı (Network) P2P ağı temsil eder
pub struct Network {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Ağdaki mevcut düğümleri döndürür
    pub async fn get_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.lock().await;
        nodes.values().cloned().collect()
    }

    // Yeni bir düğüm ekler
    pub async fn add_node(&self, node: Node) {
        let mut nodes = self.nodes.lock().await;
        if !nodes.contains_key(&node.id) {
            println!("Node added: {:?}", node.clone());
            nodes.insert(node.id.clone(), node);
        } else {
            println!("Node already exists: {:?}", node.id);
        }
    }

    pub async fn discover_peers(&self, initial_peers: Vec<SocketAddr>) {
        for peer in initial_peers {
            // Skip self connection check
            if peer.ip() == std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST) && peer.port() == self.local_port() {
                println!("Skipping self connection to peer: {:?}", peer);
                continue;
            }
    
            let mut retry_attempts = 5;
            let retry_interval = Duration::from_secs(2); // Retry every 2 seconds
    
            while retry_attempts > 0 {
                // Apply timeout to each connection attempt to avoid hanging indefinitely
                let connect_result = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(peer)).await;
    
                match connect_result {
                    Ok(Ok(mut stream)) => {
                        println!("Connected to peer: {:?}", peer);
    
                        // Request the peer list
                        let request = b"GET_NODES";
                        if let Err(e) = stream.write_all(request).await {
                            eprintln!("Failed to send request to peer: {:?}, error: {:?}", peer, e);
                            break;
                        }
    
                        let mut buffer = vec![0; 1024];
                        if let Ok(size) = stream.read(&mut buffer).await {
                            if let Ok(peer_nodes) = serde_json::from_slice::<Vec<Node>>(&buffer[..size]) {
                                for node in peer_nodes {
                                    self.add_node(node).await;
                                }
                                println!("Discovered and added new peers.");
                            } else {
                                eprintln!("Failed to deserialize peer list from peer: {:?}", peer);
                            }
                        }
                        break; // Exit if connection and processing is successful
                    },
                    Ok(Err(e)) => {
                        eprintln!("Failed to connect to peer: {:?}, error: {:?}", peer, e);
                    },
                    Err(_) => {
                        eprintln!("Connection to peer: {:?} timed out", peer);
                    }
                }
    
                retry_attempts -= 1;
                if retry_attempts > 0 {
                    println!("Retrying connection to peer: {:?} in {} seconds...", peer, retry_interval.as_secs());
                    sleep(retry_interval).await; // Wait before retrying
                } else {
                    println!("Failed to connect to peer: {:?} after multiple attempts", peer);
                }
            }
        }
    }

    pub async fn start_server(&self, addr: SocketAddr) -> io::Result<()> {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Server started on {:?}", addr);
                loop {
                    match listener.accept().await {
                        Ok((mut socket, _)) => {
                            let nodes = self.nodes.clone();
                            tokio::spawn(async move {
                                let mut buffer = [0; 1024];
                                match socket.read(&mut buffer).await {
                                    Ok(bytes_read) if bytes_read > 0 => {
                                        let received_data = &buffer[..bytes_read];
                                        if let Ok(message) = serde_json::from_slice::<Node>(received_data) {
                                            nodes
                                                .lock()
                                                .await
                                                .insert(message.id.clone(), message.clone());
                                            println!("Node added: {:?}", message);
                                        } else if let Ok(command) = String::from_utf8(received_data.to_vec()) {
                                            if command == "GET_NODES" {
                                                let node_list = nodes.lock().await.values().cloned().collect::<Vec<_>>();
                                                let response = serde_json::to_vec(&node_list).unwrap();
                                                socket.write_all(&response).await.unwrap();
                                            }
                                        }
                                    },
                                    Ok(_) => {
                                        println!("Connection closed");
                                    },
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
            Err(e) => {
                eprintln!("Failed to bind to port {:?}: {:?}", addr, e);
                Err(e)
            }
        }
    }
    

    // Yerel portu döndürür (yardımcı işlev)
    fn local_port(&self) -> u16 {
        8080
    }


   // Periyodik olarak peer listesini günceller
pub async fn periodic_peer_update(&self, initial_peers: Vec<SocketAddr>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30)); // 30 saniyede bir
    loop {
        interval.tick().await;
        println!("Running periodic peer update...");
        self.discover_peers(initial_peers.clone()).await;  // Yalnızca peer keşfini çağırıyoruz
    }
}

}

// Belirtilen boyutta dosya için uygun bir düğüm bulur
pub fn find_available_node(file_size: u64, nodes: &[Node]) -> Option<Node> {
    nodes
        .iter()
        .find(|&node| node.available_space >= file_size)
        .cloned()
}

/*mod p2p;

use p2p::{Network, Node};
use std::net::SocketAddr;
use tokio;

#[tokio::main]
async fn main() {
    let network = Network::new();
    let addr = "127.0.0.1:8080".parse().unwrap();

    // Statik olarak tanımlanmış başlangıç düğümleri
    let static_peers = vec![
        "127.0.0.1:8081".parse().unwrap(),
        "127.0.0.1:8082".parse().unwrap(),
    ];

    // Peer keşfini başlat
    tokio::spawn(async move {
        network.discover_peers(static_peers).await;
    });

    // Sunucuyu başlat
    if let Err(e) = network.start_server(addr).await {
        eprintln!("Failed to start server: {:?}", e);
    }
}
 */