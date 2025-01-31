use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

/// Ağ düğümleri arasında iletişim kurmayı sağlayan yapı.
pub struct Network {
    pub port: u16, // Ağ portu
}

impl Network {
    /// Yeni bir ağ sistemi oluşturur.
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Ağ düğümünü başlatır ve bağlantıları dinler.
    pub fn start_listener(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.port))?;
        println!("Ağ dinleyicisi {} portunda başlatıldı.", self.port);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Yeni bağlantı alındı: {:?}", stream.peer_addr());
                    thread::spawn(|| {
                        if let Err(e) = Self::handle_client(stream) {
                            eprintln!("Bağlantı sırasında hata oluştu: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Bağlantı reddedildi: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Veriyi belirli bir düğüme gönderir.
    ///
    /// # Arguments
    /// - `address`: Hedef düğüm adresi (IP:Port formatında).
    /// - `data`: Gönderilecek veri.
    pub fn send_data(address: &str, data: &[u8]) -> std::io::Result<()> {
        let mut stream = TcpStream::connect(address)?;
        stream.write_all(data)?;
        println!("Veri gönderildi: {:?}", data);
        Ok(())
    }

    /// Gelen bağlantıyı işler.
    ///
    /// # Arguments
    /// - `stream`: Gelen bağlantı akışı.
    fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read > 0 {
            println!(
                "Veri alındı: {}",
                String::from_utf8_lossy(&buffer[..bytes_read])
            );
            stream.write_all(b"Data received successfully!")?;
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::thread;

//     #[test]
//     fn test_network_communication() {
//         let port = 7878;

//         let network = Network::new(port);
//         thread::spawn(move || {
//             network.start_listener().unwrap();
//         });

//         // Bağlantının başlaması için kısa bir gecikme.
//         std::thread::sleep(std::time::Duration::from_secs(1));

//         // Test: Veri gönderimi.
//         let address = format!("127.0.0.1:{}", port);
//         let data = b"Test verisi";
//         assert!(Network::send_data(&address, data).is_ok());
//     }
// }
