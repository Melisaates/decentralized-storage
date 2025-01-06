use libp2p::{
    mdns::{Mdns, MdnsConfig},
    swarm::{Swarm, SwarmBuilder},
    PeerId, NetworkBehaviour, SwarmBuilder,
};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::error::Error;

// P2P Ağ Davranışı
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: Mdns,
}

impl Behaviour {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mdns = Mdns::new(MdnsConfig::default())?;
        Ok(Behaviour { mdns })
    }
}

// Depolama alanı kontrolü
fn check_storage_space(required_size: u64) -> bool {
    let available_space = get_available_space();
    available_space >= required_size
}

// Yerel sistemdeki boş depolama alanını kontrol et
fn get_available_space() -> u64 {
    let path = "/"; // Sistem kök dizini veya başka bir hedef dizin
    if let Ok(metadata) = fs::metadata(path) {
        return metadata.len();
    }
    0 // Hata durumunda 0 döndür
}

// Şifreli dosyayı belirli bir depolama dizinine kaydet
fn store_encrypted_file(file_path: &str, encrypted_file: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let storage_path = Path::new(file_path);
    let mut file = File::create(storage_path)?;
    file.write_all(&encrypted_file)?;
    Ok(())
}

// Dosya şifreleme (örnek basit şifreleme)
fn encrypt_file(file_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    // Şifreleme algoritması burada uygulanabilir
    Ok(content) // Şifrelenmiş içerik döndürülür
}

// // Ana işlev
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Node oluşturma
//     let node_key = libp2p::core::identity::Keypair::generate_ed25519();
//     let local_peer_id = PeerId::from(node_key.public());
//     println!("Local peer id: {:?}", local_peer_id);

//     // Ağda node'ları keşfetmek için mDNS kullan
//     let behaviour = Behaviour::new()?;
//     let mut swarm = SwarmBuilder::new(behaviour, node_key, local_peer_id.clone())
//         .executor(Box::new(|fut| {
//             tokio::spawn(fut);
//         }))
//         .build();

//     // Ağdaki node'ları keşfet
//     swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

//     // Depolama alanı kontrolü
//     let required_size: u64 = 1000000; // Örnek dosya boyutu
//     let file_path = "path/to/file.txt";

//     // Depolama alanı yeterli mi?
//     if check_storage_space(required_size) {
//         // Dosyayı şifrele
//         match encrypt_file(file_path) {
//             Ok(encrypted_file) => {
//                 // Şifreli dosyayı yükle
//                 match store_encrypted_file("path/to/storage/encrypted_file", encrypted_file) {
//                     Ok(_) => println!("File uploaded successfully!"),
//                     Err(err) => eprintln!("Error storing file: {}", err),
//                 }
//             }
//             Err(err) => eprintln!("Error encrypting file: {}", err),
//         }
//     } else {
//         println!("Not enough storage space available!");
//     }

//     Ok(())
// }





// Açıklamalar:

//     P2P Ağ Keşfi (libp2p):
//         libp2p kütüphanesi kullanılarak node'lar mDNS (Multicast DNS) protokolü ile birbirini keşfeder. Bu, yerel ağdaki node'lar arasında iletişim kurmayı sağlar.

//     Depolama Alanı Kontrolü (check_storage_space):
//         check_storage_space fonksiyonu, her node'un kendi depolama alanını kontrol eder. Bu işlem yerel disk alanını fs::metadata ile kontrol ederek yapılır.

//     Dosya Şifreleme ve Yükleme:
//         encrypt_file fonksiyonu, belirtilen dosyayı okur ve basit bir şifreleme işlemi (şu an sadece okuma ve döndürme) gerçekleştirir. Bu şifreleme kısmı daha ileri seviye algoritmalarla geliştirilebilir.
//         Şifreli dosya, store_encrypted_file fonksiyonu ile belirtilen depolama alanına kaydedilir.

//     P2P Ağda Bağlantı Kurma:
//         SwarmBuilder ile her node'a bir kimlik (PeerId) atanır ve mDNS kullanarak ağdaki diğer node'larla keşif yapılır.