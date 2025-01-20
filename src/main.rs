mod p2p;
use encryption::{encrypt_file_chunked, decrypt_file_chunked,encrypt_data_chunked,decrypt_data_chunked};
use key_management::{generate_key_iv, load_and_decrypt_key, save_key_locally};
use p2p::{find_available_node, Network, Node};
use storage::{can_store_file, store_file}; // To connect to the P2P network
//mod blockchain;
// use blockchain::{BscClient}; 
// Communication with Binance Smart Chain
mod encryption;
mod key_management;
mod storage;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use tokio::task;
mod proof_of_spacetime;
use proof_of_spacetime::periodic_check;
mod network_behaviour;
use network_behaviour::{store_chunk_on_node};


mod storage_api;
use storage_api::StorageAPI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    // API'yi başlat
    let storage_api = StorageAPI::new(
        "storage/", 
        "127.0.0.1:8080"
    ).await?;
let node1: Node = Node {
    id: "node1".to_string(),
    storage_path: "/data/node1".to_string(),
    available_space: 1000000,

    
};

    // Node'ları ekle
    storage_api.add_node(node1.clone()).await?;

    let node2: Node = Node {
        id: "node2".to_string(),
        storage_path: "/data/node2".to_string(),
        available_space: 2000000,
    };
    
        // Node'ları ekle
        storage_api.add_node(node2.clone()).await?;



        let node3: Node = Node {
            id: "node3".to_string(),
            storage_path: "/data/node3".to_string(),
            available_space: 3000000,
        };
        
            // Node'ları ekle
            storage_api.add_node(node3.clone()).await?;



    // Dosya yükle
    let file_id = storage_api.upload_file(
        "test.txt",
        "password123",
        "file_id"  // Third argument
    ).await?;
    println!("File uploaded with ID: {}", file_id);

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
    

    Ok(())
}


































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
//  let (key, iv) = generate_key_iv();
