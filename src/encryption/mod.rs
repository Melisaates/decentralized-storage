use aes::{Aes128, Aes256};
use block_modes::{Cbc, BlockMode};
use block_modes::block_padding::Pkcs7;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use hex::{encode};

const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5 MB

type Aes128Cbc = Cbc<Aes128, Pkcs7>;
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

// HMAC hesaplama ve doğrulama işlemi
pub fn encrypt_file_path(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    // Dosyayı parçalara böl
    let chunks = split_file(file_path, CHUNK_SIZE);
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let mut output_file = File::create(output_path)?;

    // HMAC oluştur
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");

    // Her parçayı şifrele ve yaz
    for chunk in chunks {
        let encrypted_data = cipher.clone().encrypt_vec(&chunk);
        hmac.update(&encrypted_data);
        output_file.write_all(&encrypted_data)?;
    }

    // Son olarak HMAC'i dosyaya yaz
    let hmac_result = hmac.finalize().into_bytes();
    output_file.write_all(&hmac_result)?;

    Ok(())
}

// Şifreli dosyayı deşifre etme ve HMAC doğrulama
pub fn decrypt_file_path(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut output_file = File::create(output_path)?;

    // HMAC uzunluğunu belirleyin (32 byte)
    const HMAC_LENGTH: usize = 32;

    // Şifreli dosyanın toplam boyutunu alın
    let file_metadata = file.metadata()?;
    let file_size = file_metadata.len() as usize;

    if file_size < HMAC_LENGTH {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Data too short to contain HMAC",
        ));
    }

    // HMAC'i şifreli dosyanın sonundan okuyun
    file.seek(std::io::SeekFrom::End(-(HMAC_LENGTH as i64)))?;
    let mut hmac_received = vec![0u8; HMAC_LENGTH];
    file.read_exact(&mut hmac_received)?;

    // Şifreli veriyi başa dönerek okumaya başlayın
    file.seek(std::io::SeekFrom::Start(0))?;

    // Değişkenler
    let mut hmac_calculator = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let mut buffer = vec![0u8; CHUNK_SIZE];

    let mut total_read = 0;

    while total_read + CHUNK_SIZE < file_size - HMAC_LENGTH {
        let read_bytes = file.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }

        // Şifreli parçayı HMAC'e ekle
        hmac_calculator.update(&buffer[..read_bytes]);

        // Şifreyi çöz
        let decrypted_chunk = cipher.clone().decrypt_vec(&buffer[..read_bytes]).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
        })?;

        // Çözülmüş parçayı yaz
        output_file.write_all(&decrypted_chunk)?;

        total_read += read_bytes;
    }

    // HMAC doğrulaması
    let hmac_calculated = hmac_calculator.finalize().into_bytes();
    if hmac_received != hmac_calculated.as_slice() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "HMAC mismatch: Data is corrupted",
        ));
    }

    Ok(())
}


pub fn encrypt_file(file_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<Vec<u8>> {
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    
    // Veriyi parçalar halinde işleyin
    let mut encrypted_result = Vec::new();
    for chunk in file_data.chunks(CHUNK_SIZE) {
        let encrypted_chunk = cipher.clone().encrypt_vec(chunk);
        encrypted_result.extend(encrypted_chunk);
    }

    // HMAC hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(&encrypted_result);
    let hmac_result = hmac.finalize().into_bytes();

    // Şifrelenmiş veri ile HMAC birleştirme
    encrypted_result.extend_from_slice(&hmac_result);

    Ok(encrypted_result)
}

pub fn decrypt_file(encrypted_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<Vec<u8>> {
    // HMAC'ı kontrol etme
    if encrypted_data.len() < 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data too short to contain HMAC"));
    }

    // Şifreli veriyi ve HMAC'i ayırma
    let hmac_offset = encrypted_data.len() - 32;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    // HMAC hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    // HMAC doğrulama
    if hmac_received != hmac_calculated.as_slice() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    // AES CBC ile şifre çözme
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");

    let mut decrypted_result = Vec::new();
    for chunk in encrypted_data.chunks(CHUNK_SIZE) {
        let decrypted_chunk = cipher.clone().decrypt_vec(chunk).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
        })?;
        decrypted_result.extend(decrypted_chunk);
    }

    Ok(decrypted_result)
}



pub fn split_file(file_path: &str, chunk_size: usize) -> Vec<Vec<u8>> {
    let mut file = File::open(file_path).expect("Dosya açılamadı");
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).expect("Dosya okunamadı");

    let mut chunks = Vec::new();
    for chunk in file_contents.chunks(chunk_size) {
        chunks.push(chunk.to_vec());
    }
    chunks
}