use libp2p::{
    mdns::{Mdns, MdnsConfig},
    request_response::{RequestResponse, RequestResponseCodec, RequestResponseConfig},
    PeerId, Swarm, NetworkBehaviour,
};
use async_trait::async_trait;
use std::error::Error;
use std::io::{self, Read, Write};
use futures::prelude::*;
use tokio; // Asenkron çalışma için tokio kullanacağız
use std::fs::File;
use async_trait::async_trait;

// Dosya değişim protokolü ve kodlaması
#[derive(Debug, Clone)]
pub struct FileExchangeProtocol;

#[derive(Clone)]
pub struct FileExchangeCodec;

#[async_trait::async_trait]
impl RequestResponseCodec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = Vec<u8>; // Dosya verisi
    type Response = Vec<u8>; // Gelen dosya verisi veya cevabı

    fn read_request(&mut self, _: &FileExchangeProtocol, io: &mut dyn Read) -> io::Result<Self::Request> {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn read_response(&mut self, _: &FileExchangeProtocol, io: &mut dyn Read) -> io::Result<Self::Response> {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn write_request(&mut self, _: &FileExchangeProtocol, io: &mut dyn Write, req: Self::Request) -> io::Result<()> {
        io.write_all(&req)?;
        Ok(())
    }

    fn write_response(&mut self, _: &FileExchangeProtocol, io: &mut dyn Write, res: Self::Response) -> io::Result<()> {
        io.write_all(&res)?;
        Ok(())
    }
}

// NetworkBehaviour içinde mDNS ve dosya değişim protokolü
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: Mdns,
    file_exchange: RequestResponse<FileExchangeCodec>,
}

impl Behaviour {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mdns = Mdns::new(MdnsConfig::default())?;
        let file_exchange = RequestResponse::new(FileExchangeCodec, vec![FileExchangeProtocol]);
        Ok(Behaviour { mdns, file_exchange })
    }
}

pub async fn send_file(peer_id: PeerId, file_path: &str, swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn std::error::Error>> {
    // Dosyayı oku
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // Dosyayı hedef peer'e gönder
    let response = swarm.behaviour_mut().file_exchange.send_request(&peer_id, buf);
    
    println!("Yanıt: {}", response);
    Ok(())
}

pub async fn receive_file(file_path: &str, swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn std::error::Error>> {
    // Dosya alındığında, alıcı tarafından kaydedilecek
    use futures::stream::StreamExt;
    let file_data = swarm.next().await.unwrap();
    let mut file = File::create(file_path)?;
    file.write_all(&file_data)?;

    println!("Dosya alındı ve kaydedildi!");
    Ok(())
}



// // Ana program
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Yerel peer için anahtar çiftini oluştur
//     let local_key = libp2p::core::identity::Keypair::generate_ed25519();
//     let local_peer_id = PeerId::from(local_key.public());

//     println!("Local peer id: {:?}", local_peer_id);

//     // Davranış nesnesi oluştur
//     let behaviour = Behaviour::new()?;

//     // Swarm yapılandırması
//     let mut swarm = Swarm::new(behaviour, local_key.clone(), local_peer_id);

//     // Bağlantı adresini dinlemeye başla
//     let listening_addr = "/ip4/0.0.0.0/tcp/0".parse()?;
//     swarm.listen_on(listening_addr)?;

//     // Ana döngüde gelen olayları işle
//     loop {
//         match swarm.next().await {
//             Some(libp2p::SwarmEvent::Behaviour(libp2p::request_response::RequestResponseEvent::Received {
//                 peer_id,
//                 message,
//                 ..
//             })) => {
//                 println!("Received file from {:?}: {:?}", peer_id, message);
//                 // Dosyayı aldıktan sonra işlemler yapılabilir
//             },
//             Some(libp2p::SwarmEvent::Behaviour(libp2p::request_response::RequestResponseEvent::Sent {
//                 peer_id,
//                 message,
//                 ..
//             })) => {
//                 println!("Sent file to {:?}: {:?}", peer_id, message);
//                 // Dosya gönderildikten sonra yapılacak işlemler
//             },
//             _ => {}
//         }
//     }
// }
