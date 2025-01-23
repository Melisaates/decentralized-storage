use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::storage;

#[derive(Clone, Serialize, Deserialize, Debug)]
// address : IP address of the node
//storage_path : Path to the storage directory
pub struct Node {
    pub id: String, //unique identifier
    pub storage_path: String,
    pub address: String,
    pub total_space: u64,
    pub available_space: u64,

}
impl Node {
    pub fn new(id: String, storage_path: String, address: String, total_space: u64) -> Self {
        Node {
            id,
            storage_path,
            address,
            total_space,
            available_space: total_space,
        }
    }
    pub fn update_available_space(&mut self, file_size: u64) {
        // Dosya boyutu kadar kullanılabilir alanı günceller
        if self.available_space >= file_size {
            self.available_space -= file_size;
        }
    }

    
}

// Network struct that holds the nodes
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
        if !nodes.contains_key(&node.id) {
            println!("Node added: {:?}", node.clone());
            nodes.insert(node.id.clone(), node);
            println!("Current network size: {}", nodes.len());
        } else {
            println!("Node already exists: {:?}", node.id);
        }
    }

    // This function is used to handle incoming messages from peers
    async fn handle_peer_message(socket: &mut TcpStream, nodes: Arc<Mutex<HashMap<String, Node>>>) -> io::Result<()> {
        let mut buffer = [0; 1024];
        match socket.read(&mut buffer).await {
            Ok(bytes_read) if bytes_read > 0 => {
                let received_data = &buffer[..bytes_read];
                if let Ok(message) = serde_json::from_slice::<Node>(received_data) {
                    println!("Received node data: {:?}", message);
                    nodes.lock().await.insert(message.id.clone(), message);
                    println!("Successfully added node to network");
                } else if let Ok(command) = String::from_utf8(received_data.to_vec()) {
                    if command == "GET_NODES" {
                        let node_list: Vec<Node> = nodes.lock().await.values().cloned().collect();
                        println!("Sending node list of size: {}", node_list.len());
                        let response = serde_json::to_vec(&node_list)?;
                        socket.write_all(&response).await?;
                    }
                }
            }
            Ok(_) => println!("Connection closed"),
            Err(e) => eprintln!("Failed to read from socket: {:?}", e),
        }
        Ok(())
    }

  

    // This function is used to discover new peers by connecting to the initial peers
    pub async fn discover_peers(&self, initial_peers: Vec<SocketAddr>) {
        for peer in initial_peers {
            // Purpose is to avoid connecting to self
            if peer.ip() == std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                && peer.port() == self.local_port()
            {
                println!("Skipping self connection to peer: {:?}", peer);
                continue;
            }

            // Retry connection attempts if failed
            let mut retry_attempts = 5;
            let retry_interval = Duration::from_secs(2); // Retry every 2 seconds

            while retry_attempts > 0 {
                // Apply timeout to each connection attempt to avoid hanging indefinitely
                let connect_result =
                    tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(peer)).await;

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
                            if let Ok(peer_nodes) =
                                serde_json::from_slice::<Vec<Node>>(&buffer[..size])
                            {
                                for node in peer_nodes {
                                    self.add_node(node).await;
                                }
                                println!("Discovered and added new peers.");
                            } else {
                                eprintln!("Failed to deserialize peer list from peer: {:?}", peer);
                            }
                        }
                        break; // Exit if connection and processing is successful
                    }
                    Ok(Err(e)) => {
                        eprintln!("Failed to connect to peer: {:?}, error: {:?}", peer, e);
                    }
                    Err(_) => {
                        eprintln!("Connection to peer: {:?} timed out", peer);
                    }
                }

                retry_attempts -= 1;
                if retry_attempts > 0 {
                    println!(
                        "Retrying connection to peer: {:?} in {} seconds...",
                        peer,
                        retry_interval.as_secs()
                    );
                    sleep(retry_interval).await; // Wait before retrying
                } else {
                    println!(
                        "Failed to connect to peer: {:?} after multiple attempts",
                        peer
                    );
                }
            }
        }
    }

    pub async fn start_server(&self, addr: SocketAddr) -> io::Result<()> {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Server started on {:?}", addr);
                loop {
                    // Accept a new connection
                    match listener.accept().await {
                        Ok((mut socket, _)) => {
                            let nodes = self.nodes.clone();
                            // tokio spawn is used to run the async block concurrently
                            tokio::spawn(async move {
                                let mut buffer = [0; 1024];
                                match socket.read(&mut buffer).await {
                                    Ok(bytes_read) if bytes_read > 0 => {
                                        let received_data = &buffer[..bytes_read];
                                        if let Ok(message) =
                                            serde_json::from_slice::<Node>(received_data)
                                        {
                                            nodes
                                                .lock()
                                                .await
                                                .insert(message.id.clone(), message.clone());
                                            println!("Node added: {:?}", message);
                                        // Else if the received data is a command
                                        } else if let Ok(command) =
                                            String::from_utf8(received_data.to_vec())
                                        {
                                            if command == "GET_NODES" {
                                                let node_list = nodes
                                                    .lock()
                                                    .await
                                                    .values()
                                                    .cloned()
                                                    .collect::<Vec<_>>();
                                                let response =
                                                    serde_json::to_vec(&node_list).unwrap();
                                                socket.write_all(&response).await.unwrap();
                                            }
                                        }
                                    }
                                    Ok(_) => {
                                        println!("Connection closed");
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
            Err(e) => {
                eprintln!("Failed to bind to port {:?}: {:?}", addr, e);
                Err(e)
            }
        }
    }

    fn local_port(&self) -> u16 {
        8080
    }

    // This function is used to periodically update the peer list
    pub async fn periodic_peer_update(&self, initial_peers: Vec<SocketAddr>) {

        let initial_peers_clone = initial_peers.clone();
        for peer in initial_peers_clone {
            self.add_node(Node {
                id: peer.to_string(),
                storage_path: std::env::var("STORAGE_PATH").unwrap_or_else(|_| "default/path".to_string()),
                total_space: 1000000000,
                available_space: 1000000000,
                address: peer.to_string(),
            }).await;    
        }

        let mut interval = tokio::time::interval(Duration::from_secs(30)); // 30 saniyede bir
        loop {
            interval.tick().await;
            println!("Running periodic peer update...");
            self.discover_peers(initial_peers.clone()).await;
        }
    }
}
// This function is used to find an available node with enough space for the file
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
