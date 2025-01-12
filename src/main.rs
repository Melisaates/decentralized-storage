use ethers::types::spoof::storage;
use hex::encode;
use node::Node;
use std::collections::HashMap;
mod encryption;
use encryption::{decrypt_file, encrypt_file};
use rand::Rng;
mod key_management;
mod proof_of_spacetime;
use proof_of_spacetime::{periodic_check};
mod p2p;
use p2p::{Network};
mod node;
mod storage;
mod blockchain;
use blockchain::{BscClient};

use ethers::types::Address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Binance Smart Chain istemcisi başlat
    let client = BscClient::new().await?;

    // Kullanıcıdan gelen dosyayı şifrele ve depola
    let file_data = b"Merhaba Binance Smart Chain!";
    let file_name = "example.txt";

    // Node'un diskinin yolu
    let storage_path = "./node_storage";
    storage::store_file(file_data, storage_path, file_name)?;

    println!("Dosya başarıyla node üzerine yüklendi: {}", file_name);

    // Metadata bilgisi oluştur
    let file_id = "unique_file_hash";
    let node_id = "node1";

    let contract_address: Address = "0xYourSmartContractAddress".parse()?;

    // Metadata bilgisini Smart Contract'a gönder
    let tx_hash = client
        .send_metadata(contract_address, file_id, node_id)
        .await?;

    println!("Metadata blockchain'e yazıldı. İşlem Hash'i: {:?}", tx_hash);

    Ok(())

}















//try to create a blockchain and network
    // // Blockchain başlat
    // let mut blockchain = Blockchain::new();

    // // P2P ağını başlat
    // let mut network = Network::new();

    // // Node'ları oluştur ve ağa ekle
    // let mut node1 = Node::new("node1", "./node1_storage", 100); // 100 MB
    // let mut node2 = Node::new("node2", "./node2_storage", 200); // 200 MB
    // node1.calculate_used_space();
    // node2.calculate_used_space();
    // network.add_node(node1);
    // network.add_node(node2);

    // // Dosya yükleme işlemi
    // let file_data = b"Hello, world!";
    // let file_size = (file_data.len() / (1024 * 1024)) as u64;

    // if let Some(node) = network.find_suitable_node(file_size) {
    //     println!("Dosya {} noduna yükleniyor...", node.id);
    //     storage::store_file(file_data, &node.storage_path, "example.txt");

    //     // Blockchain'e işlem ekle
    //     let transaction = Transaction {
    //         file_id: "example.txt".to_string(),
    //         node_id: node.id.clone(),
    //         timestamp: blockchain::current_timestamp(),
    //     };

    //     blockchain.add_block(vec![transaction]);
    //     println!("Blockchain güncellendi: {:?}", blockchain.chain);
    // } else {
    //     println!("Yeterli alana sahip bir node bulunamadı.");
    // }







//try to proof of spacetime
    // let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4";
    // periodic_check(file_path);  // Periyodik kontrol başlatılır





//try to encrypt and decrypt a file
    // let file_path = "C:/Users/melisates/Documents/WhatsApp Video 2024-11-03 at 18.47.50_f9c56fbd.mp4";
    // let encrypted_file_path = "C:/Users/melisates/Documents/encrypted_file.mp4";
    // let decrypted_file_path = "C:/Users/melisates/Documents/decrypted_file.mp4";
    // let key_file_path = "C:/Users/melisates/Documents/key_data.json";

    // // 1. Anahtar ve IV oluştur
    // let key_data = generate_key_iv();

    // // 2. Şifreleme anahtarı ve IV'yi kullanıcıya kaydedilecek şekilde JSON olarak kaydet
    // save_key_locally(key_file_path, &key_data)?;
    // println!("Anahtar ve IV kullanıcıya kaydedildi: {}", key_file_path);

    // // 3. Dosyayı şifrele
    // encrypt_file(file_path, encrypted_file_path, &key_data.key, &key_data.iv)?;
    // println!("Dosya başarıyla şifrelendi: {}", encrypted_file_path);

    // // 4. Dosyanın şifresini çöz
    // decrypt_file(
    //     encrypted_file_path,
    //     decrypted_file_path,
    //     &key_data.key,
    //     &key_data.iv,
    // )?;
    // println!("Dosya başarıyla çözüldü: {}", decrypted_file_path);

   










 // Anahtar ve IV üret
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
