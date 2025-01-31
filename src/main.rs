mod storage_;
//mod pbe_;
use bytes::{Bytes, Buf};
use node::StorageNode;
use storage_::File;
mod node;
mod file_system;
mod encryption;
use encryption::{encrypt_file_chunked, decrypt_file_chunked,encrypt_data_chunked,decrypt_data_chunked};
mod key_management;
use key_management::{generate_key_iv, load_and_decrypt_key, save_encrypted_key_to_store};
use crate::storage_::Storage;


use std::error::Error;


//node denemee
use std::time::{SystemTime, UNIX_EPOCH};
use tokio; // Tokio async runtime
use anyhow::Result;
use actix_rt::System;
use std::time::{Duration, Instant};

use crate::file_system::{file_operations, FileSystem};

#[actix_rt::main]
async fn main() -> Result<()> {
    // Example: Create a new StorageNode with an ID and total space
    let node_id = String::from("node_2");
    let total_space = 500 * 1024 * 1024; // Example space of 500MB in bytes

    let mut storage_node = StorageNode::new(node_id, total_space).await?;

    println!("Storage node created with ID: {}", storage_node.node_id);
    println!("Total space: {} bytes", storage_node.total_space);
    println!("Available space: {} bytes", storage_node.available_space);

    // Initialize the storage directory and files
    //storage_node.initialize_storage_file().await?;

    // Simulate storing a file
    let filev_id = "video_1";
    let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4";
    let data = tokio::fs::read(file_path).await?; // Read the file data asynchronously
    match storage_node.store_file(filev_id, &file_path).await {
        Ok(_) => println!("File '{}' stored successfully.", filev_id),
        Err(e) => eprintln!("Error storing file: {}", e),
    }
    let file_id="jpg_1";
    let file_path = "C:/Users/melisates/Documents/WhatsApp Image 2024-12-01 at 14.40.49_48a551a2.jpg";
    let data = tokio::fs::read(file_path).await?; // Read the file data asynchronously

    match storage_node.store_file(file_id, &file_path).await {
        Ok(_) => println!("File '{}' stored successfully.", file_id),
        Err(e) => eprintln!("Error storing file: {}", e),
    }

    // Check available space after storing the file
    println!("Available space after storing file: {} bytes", storage_node.available_space);

    // Retrieve the stored file
    //retrieve ne demek? getirme
    let output_path = "C:/Users/melisates/Downloads";
    match storage_node.retrieve_file(file_id, output_path).await {
        Ok(_) => println!("File '{}' retrieved successfully.", filev_id),
        Err(e) => eprintln!("Error retrieving file: {}", e),
    }


    match storage_node.retrieve_file(filev_id, output_path).await {
        Ok(_) => println!("File '{}' retrieved successfully.", filev_id),
        Err(e) => eprintln!("Error retrieving file: {}", e),
    }


    // Perform a health check
    match storage_node.update_health_status().await {
        Ok(_) => println!("Storage node health status: {}", storage_node.health_status),
        Err(e) => eprintln!("Error updating health status: {}", e),
    }

    //Free up space by deleting a file
    // let file_to_delete = "jpg_1";
    // match storage_node.delete_file(file_to_delete) {
    //     Ok(_) => println!("Space freed after deleting file '{}'.", file_to_delete),
    //     Err(e) => eprintln!("Error deleting file: {}", e),
    // }

    // let file_to_delete = "video_1";
    // match storage_node.delete_file(file_to_delete) {
    //     Ok(_) => println!("Space freed after deleting file '{}'.", file_to_delete),
    //     Err(e) => eprintln!("Error deleting file: {}", e),
    // }



    // Check available space after deletion
    println!("Available space after deletion: {} bytes", storage_node.available_space);

    Ok(())
}


































//KEY MANAGEMENT
// fn main() -> Result<(), Box<dyn Error>> {

//     // Set your environment variable "MASTER_KEY" with a 32-byte key before running
//     // If not set, the program will panic due to missing MASTER_KEY
//     // Example: export MASTER_KEY="your_master_key_32bytes_string"

//     // Generate new key and IV
//     let key_data = generate_key_iv();
//     println!("KeyData: {:?}", key_data);


//     // Save encrypted key data to the key store
//     let file_id = "file_id";
//     if let Err(e) = save_encrypted_key_to_store(&key_data, file_id) {
//         eprintln!("Error saving key: {}", e);
//         return Err(Box::new(e));
//     }

//     // Load and decrypt key data
//     match load_and_decrypt_key(file_id) {
//         Ok(decrypted_key_data) => {
//             println!("Decrypted key for file '{}': {:?}", file_id, decrypted_key_data.key);
//         }
//         Err(e) => {
//             eprintln!("Error loading and decrypting key: {}", e);
//         }
//     }


//     Ok(())
// }


































































// mod p2p;
// mod file_system;
// //use encryption::{encrypt_file_chunked, decrypt_file_chunked,encrypt_data_chunked,decrypt_data_chunked};
// use ethers::core::k256::elliptic_curve::rand_core::le;
// use key_management::{generate_key_iv, load_and_decrypt_key, save_encrypted_key_to_store};
// use libp2p::core::network;
// use p2p::{find_available_node, Network};
// mod node;

// use pkcs7::encrypted_data_content;
// use serde::Deserialize;
// use storage::{can_store_file, store_chunk_on_node, store_file};
// use uuid::Uuid; // To connect to the P2P network
// //mod blockchain;
// // use blockchain::{BscClient}; 
// // Communication with Binance Smart Chain
// //mod encryption;
// mod key_management;
// mod storage;
// use std::fs::File;
// use std::io::Read;
// use std::net::SocketAddr;
// use std::path::Path;
// use std::process;
// use std::sync::Arc;
// use tokio::task;
// use tokio::runtime::Runtime;
// use std::time::Duration;
// mod proof_of_spacetime;
// use proof_of_spacetime::periodic_check;
// use tokio::time::{sleep};
// use tokio::sync::Mutex;
// use tokio::net::TcpListener;
// use reqwest;
// use node::Node;
// use actix_web::{web, App, HttpServer, Responder};


// //mod storage_api_p2p;
// //use storage_api_p2p::{wait_for_peers, StorageAPI};

// use futures::future::ok;

// use anyhow::Result;

// /// Helper function to read a file and return it as a byte array
// fn read_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
//     let mut file = File::open(file_path)?;
//     let mut buffer = Vec::new();
//     file.read_to_end(&mut buffer)?;
//     Ok(buffer)
// }

// use std::error::Error;

// use tokio;
// use std::env;
// use std::fs;
// use std::io;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {


//     let config = DiskConfig {
//         base_path: "./storage/node".to_string(),
//         max_capacity: 5 * 1024 * 1024 * 1024, // 5GB
//         reserved_space: 500,    // 500MB
//     };

//     let disk_manager: Arc<DiskManager> = Arc::new(DiskManager::new(config).await?);
    
//     HttpServer::new(move || {
//         App::new()
//             .app_data(web::Data::new(disk_manager.clone()))
//             .route("/upload", web::post().to(upload_file))
//             .route("/download/{file_id}", web::get().to(download_file))
//             .route("/space", web::get().to(check_space))
//             .route("/health", web::get().to(health_check))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }





//en sonki updateden t. önceki hali
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
 

//     // Komut satırı argümanlarını al
//     let args: Vec<String> = env::args().collect();
//     if args.len() != 2 {
//         eprintln!("Usage: {} <address>", args[0]);
//         std::process::exit(1);
//     }

//     let server_addr: std::net::SocketAddr = args[1].parse().expect("Invalid address");


//     // Define initial peers for the P2P network
//     let initial_peers = vec![
//         "127.0.0.1:8081".parse::<SocketAddr>()?,
//         "127.0.0.1:8082".parse::<SocketAddr>()?,
//         "127.0.0.1:8083".parse::<SocketAddr>()?,
//     ];

    
// // #[derive(Debug, Deserialize)]
// // struct NodeCapacity {
// //     total_storage: u64,   // Örnek alanlar
// //     used_storage: u64,    // Örnek alanlar
// //     available_storage: u64,
// //     max_bandwidth: u64,
// //     current_bandwidth: u64,
// // }

// //     async fn get_node_capacity(node_address: &str) -> Result<NodeCapacity, Box<dyn std::error::Error>> {
// //         let url = format!("http://{}/capacity", node_address);
// //         let response = reqwest::get(&url).await?;
// //         let capacity: NodeCapacity = response.json().await?;
// //         Ok(capacity)
// //     }


//     // Initialize the StorageAPI
//     println!("Initializing StorageAPI...");
//     //let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "storage/".to_string());
//     let storage_path = "storage/"; // Storage path
//     if !Path::new(&storage_path).exists() {
//         return Err("Storage path does not exist.".into());
//     }
//     let storage_api = StorageAPI::new(&storage_path, server_addr, initial_peers).await?;
//     println!("{:?}",storage_api.list_nodes().await?);
//     println!("Waiting for peers to connect...");
//     wait_for_peers(&storage_api, 20).await?;

//     // println!("*****************81Node capacity: {:?}*****************", get_node_capacity("127.0.0.1:8081").await?);
//     // println!("*****************82Node capacity: {:?}*****************", get_node_capacity("127.0.0.1:8082").await?);
//     // println!("*****************83Node capacity: {:?}*****************", get_node_capacity("127.0.0.1:8083").await?);


//     // List available nodes
//     let nodes = storage_api.list_nodes().await?;
//     println!("Connected to {} peers", nodes.len());
    
//     if nodes.is_empty() {
//         println!("Warning: No peers available, some operations may fail");
//     }

//     // Example: Upload a file
//     let file_path = "C:/Users/melisates/Downloads/1. Algorithms and Computation.mp4";
//     let owner = "user123";
//     let encryption_password = "your-secure-password";

//     println!("Uploading file...");
//     let file_id = match storage_api.upload_file(file_path, owner, encryption_password).await {
//         Ok(file_id) =>  println!("File uploaded successfully. File ID: {}", file_id),
//         Err(e) => {
//             eprintln!("Failed to upload file: {:?} at {:?}", e, file_path);
//             // Print available nodes for debugging
//             println!("\nAvailable nodes:");
//             for node in storage_api.list_nodes().await? {
//                 println!("Node ID: {}, Address: {}, Available Space: {}", 
//                     node.id, node.address, node.available_space);
//             }
//         }
//     };



//     // List available nodes
//     println!("\nListing available nodes:");
//     match storage_api.list_nodes().await {
//         Ok(nodes) => {
//             for node in nodes {
//                 println!("Node ID: {}, Address: {}, Available Space: {} bytes", 
//                     node.id, node.address, node.available_space);
//             }
//         }
//         Err(e) => eprintln!("Failed to list nodes: {:?}", e),
//     }

//     let mut id= String::from("");
//     // List stored files
//     println!("\nListing stored files:");
//     match storage_api.list_files().await {
//         Ok(files) => {
//             for file in files {
//                 println!("File ID: {}, Name: {}, Owner: {}, Size: {} bytes",
//                     file.file_id, file.file_name, file.owner, file.file_size);
//                     if file.file_name == "1. Algorithms and Computation.mp4" {
//                         id = file.file_id;
//                     }
//             }
//         }
//         Err(e) => eprintln!("Failed to list files: {:?}", e),
//     }

//     // Download a file
//     let download_path = "C:/Users/melisates/Documents/downloaded_file.mp4";
//     let password = "your-secure-password";


//     println!("file id: {}", id);
//     println!("\nDownloading file...");
//     match storage_api.download_file_for_reading(&id, download_path, password).await {
//         Ok(_) => println!("File downloaded successfully to: {}", download_path),
//         Err(e) => eprintln!("Failed to download file: {:?}", e),
//     }

//     // storage_api.delete_file(&id).await?;
//     // println!("File deleted successfully.");

//     // List stored files
//     println!("\nListing stored files:");
//     match storage_api.list_files().await {
//         Ok(files) => {
//             for file in files {
//                 println!("File ID: {}, Name: {}, Owner: {}, Size: {} bytes",
//                     file.file_id, file.file_name, file.owner, file.file_size);
//             }
//         }
//         Err(e) => eprintln!("Failed to list files: {:?}", e),
//     }


    
//     // // Download a file
//     // let destination_path = "C:/Users/melisates/Documents/downloaded_file.mp4";
//     // storage_api.download_file_for_reading(&id, destination_path, encryption_password);



//     // Keep the main thread running
//     println!("\nServer running. Press Ctrl+C to exit.");
//     tokio::signal::ctrl_c().await?;
//     println!("Shutting down...");

//     Ok(())
// }
















////*********************key_management deneme******************************************************************* */
// fn main() -> io::Result<()> {
//     // Example usage:
//     let password = "pass2o3rrd";
//     let file_id = "file1.6";

//     // Generate key data
//     let key_data = generate_key_iv();
//     println!("KeyData: {:?}", key_data);

//     // Save the encrypted key to the key store (JSON file)
//     let s= save_encrypted_key_to_store(&key_data, password, file_id)?;
//     println!("Encrypted KeyData saved to the key store.");
//     println!("{:?}", s);

//     // Load and decrypt the key
//     let decrypted_key_data = load_and_decrypt_key(password, file_id)?;
//     println!("Decrypted KeyData: {:?}", decrypted_key_data);

//     Ok(())
// }
//****************************************************************************************************** */
































//*********************p2p deneme******************************************************************* */
// #[tokio::main]
// async fn main() {
//     // Komut satırı argümanlarını al
//     let args: Vec<String> = env::args().collect();
//     if args.len() != 2 {
//         eprintln!("Usage: {} <address>", args[0]);
//         std::process::exit(1);
//     }

//     let addr: std::net::SocketAddr = args[1].parse().expect("Invalid address");

//     let network: Arc<Network> = Arc::new(Network::new());

//     // Statik olarak tanımlanmış başlangıç düğümleri
//     let static_peers = vec![
//         "127.0.0.1:8081".parse().unwrap(),
//         "127.0.0.1:8082".parse().unwrap(),
//     ];

//     let network_clone = Arc::clone(&network);

//     // Mevcut düğümleri kontrol et
//     let x = network_clone.get_nodes().await;
//     println!("Current nodes: {:?}", x);

//     // // Peer keşfini başlat
//     // tokio::spawn(async move {
//     //     network.discover_peers(static_peers).await;
//     // });

//     // // Sunucuyu başlat
//     // tokio::spawn(async move {
//     //     if let Err(e) = network_clone.start_server(addr).await {
//     //         eprintln!("Server error: {:?}", e);
//     //     }
//     // });

//     let runtime = Runtime::new().unwrap();
    
//     // Start the server and periodic peer update concurrently
//     {
//         let network = Arc::clone(&network);
//         let addr = addr.clone();
//         runtime.spawn(async move {
//             network.start_server(addr).await.unwrap();
//         });
//     }

//     {
//         let network = Arc::clone(&network);
//         runtime.spawn(async move {
//             network.periodic_peer_update(static_peers).await;
//         });
//     }




//     // Programın açık kalması için beklet
//     tokio::signal::ctrl_c().await.unwrap();
//     println!("Shutting down...");
    
// }
//********************************************************************************************************************** */


    // // Get the storage path from the environment or define it
    // let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "/default/storage/path".to_string());

    // // Initialize the StorageAPI with network and file index
    // let storage_api = storage_api::StorageAPI::new(&storage_path, "127.0.0.1:8000")
    //     .await
    //     .expect("Failed to initialize StorageAPI");

    // // Initialize the Network and add nodes
    // let network = p2p::Network::new();
    // let initial_peers = vec!["127.0.0.1:8001".parse().unwrap(), "127.0.0.1:8002".parse().unwrap()];
    // network.discover_peers(initial_peers).await;

    // // Start the server to accept peer connections
    // let server_addr = "127.0.0.1:8000".parse().unwrap();
    // tokio::spawn(async move {
    //     network.start_server(server_addr).await.unwrap();
    // });

    // // Test uploading a file
    // let file_path = "  "; 
    // let owner = "owner_id";
    // let encryption_password = "encryption_password";

    // match storage_api.upload_file(file_path, owner, encryption_password).await {
    //     Ok(file_id) => println!("File uploaded successfully with file ID: {}", file_id),
    //     Err(err) => eprintln!("Error uploading file: {}", err),
    // }

    // Ok(())



// // Assuming your code is already imported here

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Initialize StorageAPI
//     let storage_path = "storage/"; // Storage path
//     let server_addr = "127.0.0.1:8080"; // You can replace this with the actual server address
//     let storage_api = StorageAPI::new(storage_path, server_addr).await?;

//     let listNodes =storage_api.list_nodes().await?;
//     println!("Available nodes: {:?}", listNodes);



//     // Define the file path and owner information
//     let file_path = "C:/Users/melisates/Documents/WhatsApp Image 2024-12-01 at 14.40.49_48a551a2.jpg"; 
//     let owner = "user_id_123";
//     let encryption_password = "password123";

//     // Upload the file using the API
//     match storage_api.upload_file(file_path, owner, encryption_password).await {
//         Ok(file_id) => {
//             println!("File uploaded successfully with ID: {}", file_id);
//         }
//         Err(e) => {
//             eprintln!("Error uploading file: {:?}", e);
//         }
//     }

//     storage_api.list_files().await?;
//     println!("Stored files: {:?}", storage_api.list_files().await?);

//     Ok(())
// }
























// #[tokio::main]
// async fn main() -> Result<()> {
//     // Örnek dosya verisi
//     let file_path = "C:/Users/melisates/Downloads/1. Algorithms and Computation.mp4";
//     let encrypted_data_content= read_file(file_path).expect("Failed to read the file!");
//     let node_storage_path = "./storage"; // Depolama alanı yolu
//     let file_name = "test_file.mp4"; // Dosya adı
//     let node_id = "node_1"; // Node ID
//     let password = "password123"; // Şifre
//     let output_path = "C:/Users/melisates/Documents/encrypted_file.mp4"; // Şifrelenmiş dosya çıktı yolu

//     encrypt_file_chunked(file_path, output_path, password);

//     // Dosya depolama işlemi
//     match store_file(&encrypted_data_content, node_storage_path, file_name, node_id) {
//         Ok(_) => println!("File stored successfully."),
//         Err(e) => eprintln!("Error storing file: {}", e),
//     }

//     // Node'ları oluştur (Bu kısmı, her node için gerekli bilgileri sağlamak amacıyla özelleştirebilirsiniz)
//     let nodes = vec![
//         Node {
//             id: "node_1".to_string(),
//             storage_path: "./storage".to_string(),
//             available_space: 100_000, // 100 KB örnek
//             address: "127.0.0.1:8080".to_string(),
//         },
//         Node {
//             id: "node_2".to_string(),
//             storage_path: "./storage".to_string(),
//             available_space: 200_000, // 200 KB örnek
//             address: "127.0.0.1:8081".to_string(),
//         },
//     ];

//     // Dosya boyutunu al (örnek: 1 KB)
//     let file_size = encrypted_data_content.len() as u64;

//     // Hangi node'un dosyayı depolayabileceğini kontrol et
//     match can_store_file(&nodes, file_size).await {
//         Some(node_id) => println!("File can be stored on node: {}", node_id),
//         None => println!("No suitable node found to store the file."),
//     }

//     // Chunk'ı bir node'a depolama işlemi (örnek chunk)
//     let chunk_data = b"Some chunk data";
//     //50 saniye içinde 3 tekrar deneme
//     match store_chunk_on_node(chunk_data, &nodes[0], 3,50).await {
//         Ok(_) => println!("Chunk successfully stored on node."),
//         Err(e) => eprintln!("Error storing chunk: {}", e),
//     }

//     Ok(())
// }



//     // API'yi başlat
//     let storage_api = StorageAPI::new(
//         "storage/", 
//         "127.0.0.1:8080"
//     ).await?;
// let node1: Node = Node {
//     id: "node1".to_string(),
//     storage_path: "/data/node1".to_string(),
//     available_space: 1000000,


// };

//     // Node'ları ekle
//     storage_api.add_node(node1.clone()).await?;

//     let node2: Node = Node {
//         id: "node2".to_string(),
//         storage_path: "/data/node2".to_string(),
//         available_space: 2000000,
//     };
    
//         // Node'ları ekle
//         storage_api.add_node(node2.clone()).await?;



//         let node3: Node = Node {
//             id: "node3".to_string(),
//             storage_path: "/data/node3".to_string(),
//             available_space: 3000000,
//         };
        
//             // Node'ları ekle
//             storage_api.add_node(node3.clone()).await?;



//     // Dosya yükle
//     let file_id = storage_api.upload_file(
//         "test.txt",
//         "password123",
//         "file_id"  // Third argument
//     ).await?;
//     println!("File uploaded with ID: {}", file_id);

    // // Node'ları listele
    // let nodes = storage_api.list_nodes().await?;
    // println!("Available nodes: {:?}", nodes);

    // // Dosyaları listele
    // let files = storage_api.list_files().await?;
    // println!("Stored files: {:?}", files);

    // // Dosya indir
    // storage_api.download_file(
    //     &file_id,
    //     "downloaded_test.txt",
    //     Some("password123")
    // ).await?;
    

//     Ok(())
// }


































//************************************************************************************************************** */
//[tokio::main]
//async fn main() {
//     // 1. Start the P2P network
//     let network = Arc::new(Network::new());
//     let server_address: SocketAddr = "127.0.0.1:8080".parse().unwrap_or_else(|e| {
//         eprintln!("Invalid address: {:?}", e);
//         process::exit(1);
//     });
//     // Add a sample node
//     let node = Node {
//         id: "node_1".to_string(),
//         storage_path: "/data/node_1".to_string(),
//         available_space: 10000,
//     };

//     network.add_node(node).await;

//     let node3 = Node {
//         id: "node_8".to_string(),
//         storage_path: "/data/node_8".to_string(),
//         available_space: 800000000000,
//     };

//     network.add_node(node3).await;

//     println!("Nodes added successfully!");
//     println!("{:?}", network.get_nodes().await);

//     let network_clone = Arc::clone(&network);
//     tokio::spawn(async move {
//         if let Err(e) = network_clone.start_server(server_address).await {
//             eprintln!("Server error: {:?}", e);
//         }
//     });

//     // 2. Start the storage system
//     let storage_path = "storage/";

//     // 3. Generate and save a key for encryption
//     let key_file_path = "keys/key_data.json";
//     let password = "128"; // Set a password for the key
//                           //let key_data = generate_key_iv();
//                           //save_key_locally(key_file_path, &key_data, password).expect("Failed to save the key!");

//     // println!("Key generated and saved successfully!");
//     // println!("Key: {:?}", key_data);
//     // println!("***************");

//     // 5. Automatically calculate the file size and check the storage space
//     let file_path =
//         "C:/Users/melisates//Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4";
//     let file_name = "wp.mp4";
//     let file_data = read_file(file_path).expect("Failed to read the file!");
//     let file_size = file_data.len() as u64; // Calculate the file size in bytes.
//                                             // Get the nodes in the network and find a suitable node
//     let nodes = network.get_nodes().await; // Now accessible
//     if let Some(available_node) = find_available_node(2000, &nodes) {
//         println!("Available Node: {:?}", available_node);
//     } else {
//         println!("No suitable node found");
//     }

//     // Check if there is a node that can store the file
//     if let Some(node_id) = can_store_file(&nodes, file_size).await {
//         // 6. Encrypt the file
//         let encrypted_file_path = "C:/Users/melisates/Documents/encryptedwp_file.mp4";
//         encrypt_file_chunked(file_path, encrypted_file_path, password).expect("Failed to encrypt the file!");
//         println!("File encrypted successfully: {}", encrypted_file_path);

//         // 7. Store the encrypted file
//         let encrypted_file_data = read_file(encrypted_file_path).expect("Failed to read the encrypted file!");
//         // // 4. Load and verify the key
//         // let loaded_key_data =
//         //     load_and_decrypt_key(key_file_path, password).expect("Failed to load the key!");
//         // println!("Key loaded successfully: {:?}", loaded_key_data);

//         store_file(&encrypted_file_data, storage_path, file_name).expect("Failed to save the file!");
//         println!(
//             "Encrypted file saved to storage: {}",
//             file_name
//         );
//     } else {
//         println!("Not enough storage space!");
//     }

//     // 8. Run the Proof-of-Spacetime mechanism
//     let cloned_storage_path = storage_path.to_string();
//     task::spawn(async move {
//         periodic_check(&cloned_storage_path).await;
//     });

//     // Keep the application running
//     loop {
//         tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
//     }
// }

// /// Helper function to read a file and return it as a byte array
// fn read_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
//     let mut file = File::open(file_path)?;
//     let mut buffer = Vec::new();
//     file.read_to_end(&mut buffer)?;
//     Ok(buffer)
// }

//************************************************************************************************************** */








//     // Start Binance Smart Chain client
//     let client = BscClient::new().await?;

//     // Encrypt and store the file from the user
//     let file_data = b"Hello Binance Smart Chain!";
//     let file_name = "example.txt";

//     // Real network nodes
//     let mut network = Network::new();  // Connect to nodes in the P2P network
//     let nodes = network.get_nodes().await;  // Get the nodes in the network

//     // Get the file size
//     let file_size = file_data.len() as u64;

//     // Find a suitable node for the file
//     if let Some(node) = find_available_node(file_size, &nodes) {
//         // Store the file on the suitable node
//         let node_storage_path = node.storage_path.clone();
//         storage::store_file(file_data, &node_storage_path, file_name)?;
//         println!("File successfully uploaded to the node: {}", file_name);

//         // Create metadata information
//         let file_id = "unique_file_hash";  // Should be replaced with the actual file hash
//         let node_id = &node.id;  // Get the node ID

//         let contract_address: Address = "0xYourSmartContractAddress".parse()?;

//         // Send metadata information to the Smart Contract
//         let tx_hash = client
//             .send_metadata(contract_address, file_id, node_id)
//             .await?;

//         println!("Metadata written to the blockchain. Transaction Hash: {:?}", tx_hash);
//     } else {
//         println!("No node with sufficient storage space found.");
//     }

//     Ok(())
// }

//try to create a blockchain and network
// // Start the blockchain
// let mut blockchain = Blockchain::new();

// // Start the P2P network
// let mut network = Network::new();

// // Create and add nodes to the network
// let mut node1 = Node::new("node1", "./node1_storage", 100); // 100 MB
// let mut node2 = Node::new("node2", "./node2_storage", 200); // 200 MB
// node1.calculate_used_space();
// node2.calculate_used_space();
// network.add_node(node1);
// network.add_node(node2);

// // File upload process
// let file_data = b"Hello, world!";
// let file_size = (file_data.len() / (1024 * 1024)) as u64;

// if let Some(node) = network.find_suitable_node(file_size) {
//     println!("Uploading file to node {}...", node.id);
//     storage::store_file(file_data, &node.storage_path, "example.txt");

//     // Add transaction to the blockchain
//     let transaction = Transaction {
//         file_id: "example.txt".to_string(),
//         node_id: node.id.clone(),
//         timestamp: blockchain::current_timestamp(),
//     };

//     blockchain.add_block(vec![transaction]);
//     println!("Blockchain updated: {:?}", blockchain.chain);
// } else {
//     println!("No node with sufficient space found.");
// }

//try to proof of spacetime
// let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4";
// periodic_check(file_path);  // Start periodic check

//try to encrypt and decrypt a file
//     let file_path = "C:/Users/melisates/Downloads/1. Algorithms and Computation.mp4";
//     //"C:\Users\melisates\Downloads\1. Algorithms and Computation.mp4"
//     //Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4
//     //WhatsApp Image 2024-12-01 at 14.40.49_48a551a2.jpg
//     let encrypted_file_path = "C:/Users/melisates/Documents/encrypted_fileeee.mp4";
//     let decrypted_file_path = "C:/Users/melisates/Documents/decrypted_fileee.mp4";
//     let key_file_path = "C:/Users/melisates/Documents/key_data.mp4";

//     println!("Encrypted file size: {}", std::fs::metadata(encrypted_file_path)?.len());
// println!("File size before decryption: {}", std::fs::metadata(decrypted_file_path)?.len());

//     // 1. Generate key and IV
//     let key_data = generate_key_iv();
//     println!("Key_: {:?}", key_data.key);
// println!("IV_: {:?}", key_data.iv);

//     // 2. Save the encryption key and IV as JSON for the user
//     save_key_locally(key_file_path, &key_data)?;
//     println!("Key and IV saved for the user: {}", key_file_path);

//     // 3. Encrypt the file
//     encrypt_file_path(file_path, encrypted_file_path, &key_data.key, &key_data.iv)?;
//     println!("File encrypted successfully: {}", encrypted_file_path);

//     // 4. Decrypt the file
//     decrypt_file_path(
//         encrypted_file_path,
//         decrypted_file_path,
//         &key_data.key,
//         &key_data.iv,
//     )?;
//     println!("File decrypted successfully: {}", decrypted_file_path);

//     println!("Decrypted file size: {}", std::fs::metadata(decrypted_file_path)?.len());

// Ok(())
// }

// Generate key and IV
// let mut key_manager =KeyManager{
//     keys: HashMap::new(),
// };
// let key_data = KeyManager::generate_key_iv();

//   pub fn generate_key_iv() -> ([u8; 16], [u8; 16]) {
//     let mut key = [0u8; 16];
//     let mut iv = [0u8; 16];
//     let mut rng = rand::thread_rng();

//     rng.fill(&mut key);
//     rng.fill(&mut iv);

//     (key, iv)
// }
