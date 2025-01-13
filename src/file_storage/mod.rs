use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use ethers::providers::{Provider, Http};
use libp2p::{swarm, PeerId};
use sha2::{Sha256, Digest};
use tokio::runtime::Runtime;
use std::sync::Arc;
use encryption::{encrypt_file_path, decrypt_file_path};
mod network;
use network::{send_file, receive_file};

use crate::encryption;


// Ethereum provider (örneğin, Infura, Alchemy vs.)
const RPC_URL: &str = "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID";

// Dosya şifreleme işlemi ve blockchain'e yükleme
pub async fn upload_file_to_blockchain(file_path: &str, encrypted_file_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    // Dosya şifrele
    encrypt_file_path(file_path, encrypted_file_path, key, iv)?;
    println!("Dosya şifrelendi ve yüklendi: {}", encrypted_file_path);

    // Dosya hash'ini hesapla
    match calculate_file_hash(file_path) {
        Ok(file_hash) => {
            println!("Dosya hash'ı: {}", file_hash);

            // Dosya boyutunu al
            let file_size = std::fs::metadata(file_path)?.len();

            // Uygun node'u bulmak için kontrat fonksiyonunu çağır.
            // Bu fonksiyon, dosya boyutuna göre en uygun node'u seçer.
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                match find_suitable_node(&file_size).await {
                    Ok(node_id) => {
                        println!("Dosya yüklemesi şu node'a yapılacak: {}", node_id);
                        // Dosyayı seçilen node'a yükleme işlemini başlat
                        match upload_file_to_node(node_id.parse().unwrap(), encrypted_file_path, &mut swarm).await {
                            Ok(_) => println!("Dosya başarıyla node'a yüklendi."),
                            Err(e) => eprintln!("Dosya yükleme hatası: {:?}", e),
                        }
                    },
                    Err(e) => eprintln!("Node bulma hatası: {:?}", e),
                }
            });

            // BSC'ye dosya sahipliğini kaydetme
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                match store_file_ownership_in_contract(&file_hash).await {
                    Ok(_) => println!("Dosya sahipliği başarıyla BSC'ye kaydedildi."),
                    Err(e) => eprintln!("Dosya sahipliği kaydetme hatası: {:?}", e),
                }
            });
        },
        Err(e) => eprintln!("Hash hesaplama hatası: {:?}", e),
    }

    Ok(())
}

// Dosya hash'ini hesaplayan fonksiyon
fn calculate_file_hash(file_path: &str) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let result = hasher.finalize();
    let hash_string = result.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok(hash_string)
}

// Akıllı kontratta dosya sahipliğini saklayan fonksiyon
async fn store_file_ownership_in_contract(file_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let wallet: LocalWallet = "YOUR_PRIVATE_KEY".parse()?;
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let contract_address = "0xYourContractAddress"; // Kontrat adresinizi buraya ekleyin
    let abi = include_bytes!("your_contract_abi.json");
    let contract = Contract::new(
        contract_address.parse()?,
        abi,
        client.clone(),
    );

    let tx = contract
        .method::<_, H256>("recordFileOwnership", (file_hash.to_string(),))?
        .send()
        .await?;

    println!("Transaction hash: {:?}", tx);
    Ok(())
}

// Node seçimi için kontrat fonksiyonu
async fn find_suitable_node(file_size: &u64) -> Result<String, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let wallet: LocalWallet = "YOUR_PRIVATE_KEY".parse()?;
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let contract_address = "0xYourContractAddress"; // Akıllı kontrat adresiniz
    let abi = include_bytes!("your_contract_abi.json");
    let contract = Contract::new(
        contract_address.parse()?,
        abi,
        client.clone(),
    );

    let node_id: String = contract
        .method::<_, String>("findSuitableNode", (file_size.clone(),))
        .unwrap()
        .call()
        .await?;

    Ok(node_id)
}

// Dosya gönderme işlemi
async fn upload_file_to_node(node_id: PeerId, file_path: &str, swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn std::error::Error>> {
    // Dosyayı node'a gönder
    send_file(node_id, file_path, swarm).await?;
    Ok(())
}

// Dosya alma işlemi
 async fn download_file_from_node(node_id: PeerId, file_path: &str, swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn std::error::Error>> {
    // Node'dan dosya al ve kaydet
    receive_file(file_path, swarm).await?;

    // Kullanıcıya dosyanın indirildiğini bildirebiliriz
    println!("Dosya indirildi ve kaydedildi: {}", file_path);
    Ok(())
}
