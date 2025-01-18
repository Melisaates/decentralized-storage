use aes::{Aes128, Aes256};
use block_modes::{Cbc, BlockMode};
use block_modes::block_padding::Pkcs7;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use hex::{encode};

use crate::key_management::{derive_key, encrypt_key_data, generate_key_iv, load_and_decrypt_key, save_encrypted_key};

const CHUNK_SIZE: usize = 10 * 1024 * 1024; // 5 MB
const HMAC_LENGTH: usize = 32;  // HMAC length (in bytes)

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

pub fn encrypt_file_path(file_path: &str, output_path: &str, password: &str) -> std::io::Result<()> {
    let key_data = generate_key_iv();
    let encryption_key = derive_key(password); // Kullanıcı şifresi ile anahtar türet
    let encrypted_key = encrypt_key_data(&key_data, &encryption_key);
    
    // Merkezi servise anahtarı saklama (bu, örneğin dosya veya veritabanı olabilir)
    save_encrypted_key("encrypted_key_file", &encrypted_key)?;

    println!("Key data: {:?}", key_data);
    println!("keyiv lennnnnn: {:?}", key_data.iv.len());
    // Dosyayı şifrele
    // AES-256 için anahtar 32 byte ve IV 16 byte
    let cipher = Aes128Cbc::new_from_slices(&key_data.key, &key_data.iv).unwrap();  // Bu doğru şekilde çalışmalıdır
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let encrypted_data = cipher.encrypt_vec(&data);

    // HMAC hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(&key_data.key).expect("HMAC can take key of any size");
    hmac.update(&encrypted_data);
    let hmac_result = hmac.finalize().into_bytes();

    // Şifreli veriyi ve HMAC'ı dosyaya yazma
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    output_file.write_all(&hmac_result)?;

    Ok(())
}




//withchunk
// HMAC hesaplama ve doğrulama işlemi
pub fn encrypt_file_path_with_chunk(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<()> {
    // Dosyayı parçalara böl

    println!("key: {:?}", key);
    println!("iv: {:?}", iv);
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



// Decrypt the file with key from the key manager
pub fn decrypt_file_path(file_path: &str, output_path: &str, password: &str) -> std::io::Result<()> {
    // Load and decrypt the key using the password
    let key_data = load_and_decrypt_key(file_path, password)?;

    let mut file = File::open(file_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    // HMAC check (we assume the HMAC is stored at the end of the file)
    if encrypted_data.len() < 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data too short to contain HMAC"));
    }

    let hmac_offset = encrypted_data.len() - 32;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    // Compute HMAC
    let mut hmac = Hmac::<Sha256>::new_from_slice(&key_data.key).expect("HMAC can take key of any size");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    // HMAC verification
    if hmac_received != hmac_calculated.as_slice() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    // Decrypt the file data
    let cipher = Aes128Cbc::new_from_slices(&key_data.key, &key_data.iv).expect("Error creating cipher");
    let decrypted_data = cipher.decrypt_vec(encrypted_data).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
    })?;

    // Write the decrypted data to the output file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;

    Ok(())
}



// decrypt file path fonksiyonu oluşturuldu   
// Chunk size ve HMAC length sabitleri tanımlandı
pub fn decrypt_file_path_with_chunk(file_path: &str, output_path: &str, key: &[u8; 16], iv: &[u8; 16]) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut output_file = File::create(output_path)?;
    


    let file_metadata = file.metadata()?;
    let file_size = file_metadata.len() as usize;

    if file_size < HMAC_LENGTH {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Data too short to contain HMAC"));
    }

    file.seek(SeekFrom::End(-(HMAC_LENGTH as i64)))?;
    let mut hmac_received = vec![0u8; HMAC_LENGTH];
    file.read_exact(&mut hmac_received)?;

    file.seek(SeekFrom::Start(0))?;

    let mut hmac_calculator = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_read = 0;

    while total_read + CHUNK_SIZE < file_size - HMAC_LENGTH {
        let read_bytes = file.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }
        // Log the encrypted data chunk before decryption
        println!("Encrypted chunk: {:?}", &buffer[..read_bytes]);
    
        // HMAC update
        hmac_calculator.update(&buffer[..read_bytes]);
    
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(key, iv).expect("Error creating cipher");
        let decrypted_chunk = cipher.decrypt_vec(&buffer[..read_bytes]).map_err(|e| {
            eprintln!("Decryption error: {:?}", e);
            io::Error::new(io::ErrorKind::InvalidData, "Decryption failed")
        })?;
    
        // Log the decrypted chunk
        println!("Decrypted chunk: {:?}", decrypted_chunk);
        output_file.write_all(&decrypted_chunk)?;
    
        total_read += read_bytes;
    }
    
    // HMAC doğrulaması
    let hmac_calculated = hmac_calculator.finalize().into_bytes();
    println!("Calculated HMAC: {:?}", hmac_calculated);
println!("Received HMAC: {:?}", hmac_received);
    if hmac_received != hmac_calculated.as_slice() {
        eprintln!("HMAC mismatch: expected {:?}, got {:?}", hmac_received, hmac_calculated);
        return Err(io::Error::new(io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    Ok(())
}


pub fn encrypt_file(file_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<Vec<u8>> {
  

    // AES CBC ile şifreleme
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let encrypted_data = cipher.encrypt_vec(&file_data);
     // HMAC hesaplama
     let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
     hmac.update(&encrypted_data);
     let hmac_result = hmac.finalize().into_bytes();

     Ok(encrypted_data)
}




pub fn encrypt_file_with_chunk(file_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<Vec<u8>> {
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

    let hmac_offset = encrypted_data.len() - 32;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    println!("Received HMAC: {:?}", hmac_received);

    // HMAC'ı hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    println!("Calculated HMAC: {:?}", hmac_calculated);

    // HMAC doğrulama
    if hmac_received != hmac_calculated.as_slice() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    // Şifreyi çözme
    // AES CBC ile şifre çözme
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let decrypted_data = cipher.decrypt_vec(encrypted_data).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
    })?;

    Ok(decrypted_data)
}





pub fn decrypt_file_with_chunk(encrypted_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> std::io::Result<Vec<u8>> {
    //let mut file = File::open(file_path)?;
  //  let mut encrypted_data = Vec::new();
  //  file.read_to_end(&mut encrypted_data)?;

    // HMAC'ı kontrol etme
    if encrypted_data.len() < 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Data too short to contain HMAC"));
    }

    // HMAC'ı ve şifreli veriyi ayırma
    // HMAC, verinin son 32 byte'ıdır
    // Veri, HMAC'den önceki kısımdır
    let hmac_offset = encrypted_data.len() - 32;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    println!("Received HMAC: {:?}", hmac_received);

    // HMAC'ı hesaplama
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    println!("Calculated HMAC: {:?}", hmac_calculated);

    // HMAC doğrulama
    if hmac_received != hmac_calculated.as_slice() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HMAC mismatch: Data is corrupted"));
    }

    // Şifreyi çözme
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Error creating cipher");
    let decrypted_data = cipher.decrypt_vec(encrypted_data).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Decryption failed")
    })?;

    Ok(decrypted_data)
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










//**************************************************************** */
// Define the AES-128 CBC type with PKCS7 padding

// Struct to hold encryption key and initialization vector (IV)
pub struct KeyData {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}


// Encrypt a single chunk of data
fn encrypt_chunk(chunk: &[u8], cipher: &Aes128Cbc) -> Vec<u8> {
    cipher.clone().encrypt_vec(chunk)
}

// Function to encrypt a file in chunks
pub fn encrypt_file_chunked(
    file_path: &str,
    output_path: &str,
    key: &[u8; 16],
    iv: &[u8; 16],
) -> io::Result<()> {
    let mut input_file = File::open(file_path)?;
    let mut output_file = File::create(output_path)?;
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Cipher creation failed");
    
    // Create a temporary buffer to store all encrypted data before writing
    let mut encrypted_buffer = Vec::new();
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let chunk = &buffer[..bytes_read];
        let encrypted_chunk = encrypt_chunk(chunk, &cipher);
        
        // Write chunk length
        let chunk_len = (encrypted_chunk.len() as u32).to_le_bytes();
        encrypted_buffer.extend_from_slice(&chunk_len);
        
        // Write encrypted chunk
        encrypted_buffer.extend_from_slice(&encrypted_chunk);
    }

    // Calculate HMAC for all encrypted data
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC creation failed");
    hmac.update(&encrypted_buffer);
    let hmac_result = hmac.finalize().into_bytes();

    // Write everything to file
    output_file.write_all(&encrypted_buffer)?;
    output_file.write_all(&hmac_result)?;
    
    Ok(())
}

// Function to decrypt a file in chunks
pub fn decrypt_file_chunked(
    file_path: &str,
    output_path: &str,
    key: &[u8; 16],
    iv: &[u8; 16],
) -> Result<(), Box<dyn Error>> {
    let mut input_file = File::open(file_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    // Verify file length
    if encrypted_data.len() < HMAC_LENGTH {
        return Err("File too short".into());
    }

    // Split HMAC and encrypted data
    let hmac_offset = encrypted_data.len() - HMAC_LENGTH;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    // Verify HMAC
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC creation failed");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    if hmac_received != hmac_calculated.as_slice() {
        println!("Expected HMAC: {:?}", hmac_calculated.as_slice());
        println!("Received HMAC: {:?}", hmac_received);
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "HMAC verification failed")));
    }

    // Prepare for decryption
    let cipher = Aes128Cbc::new_from_slices(key, iv)?;
    let mut writer = BufWriter::new(File::create(output_path)?);
    let mut offset = 0;

    // Process chunks
    while offset + 4 <= encrypted_data.len() {
        // Read chunk length
        let chunk_len = u32::from_le_bytes(
            encrypted_data[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        // Verify chunk boundaries
        if offset + chunk_len > encrypted_data.len() {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Invalid chunk length")));
        }

        // Decrypt chunk
        let encrypted_chunk = &encrypted_data[offset..offset + chunk_len];
        let decrypted_chunk = cipher.clone().decrypt_vec(encrypted_chunk)?;
        writer.write_all(&decrypted_chunk)?;

        offset += chunk_len;
    }

    writer.flush()?;
    Ok(())
}

// Function to encrypt data in chunks
pub fn encrypt_data_chunked(
    file_data: &[u8],
    key: &[u8; 16],
    iv: &[u8; 16],
) -> std::io::Result<Vec<u8>> {
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Cipher creation failed");
    
    let mut encrypted_buffer = Vec::new();
    let mut offset = 0;

    // Process chunks of file data
    while offset < file_data.len() {
        let chunk_end = std::cmp::min(offset + CHUNK_SIZE, file_data.len());
        let chunk = &file_data[offset..chunk_end];
        let encrypted_chunk = encrypt_chunk(chunk, &cipher);

        // Add chunk length
        let chunk_len = (encrypted_chunk.len() as u32).to_le_bytes();
        encrypted_buffer.extend_from_slice(&chunk_len);
        
        // Add encrypted chunk
        encrypted_buffer.extend_from_slice(&encrypted_chunk);

        offset = chunk_end;
    }

    // Calculate HMAC for the encrypted data
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC creation failed");
    hmac.update(&encrypted_buffer);
    let hmac_result = hmac.finalize().into_bytes();

    // Append HMAC to the encrypted buffer
    encrypted_buffer.extend_from_slice(&hmac_result);

    Ok(encrypted_buffer)
}

// Function to decrypt data in chunks
pub fn decrypt_data_chunked(
    encrypted_data: &[u8],
    key: &[u8; 16],
    iv: &[u8; 16],
) -> std::io::Result<Vec<u8>> {
    // Verify the HMAC
    if encrypted_data.len() < HMAC_LENGTH {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Encrypted data too short"));
    }

    let hmac_offset = encrypted_data.len() - HMAC_LENGTH;
    let hmac_received = &encrypted_data[hmac_offset..];
    let encrypted_data = &encrypted_data[..hmac_offset];

    let mut hmac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC creation failed");
    hmac.update(encrypted_data);
    let hmac_calculated = hmac.finalize().into_bytes();

    if hmac_received != hmac_calculated.as_slice() {
        println!("Expected HMAC: {:?}", hmac_calculated.as_slice());
        println!("Received HMAC: {:?}", hmac_received);
        return Err(io::Error::new(io::ErrorKind::InvalidData, "HMAC verification failed"));
    }

    // Decrypt the data
    let cipher = Aes128Cbc::new_from_slices(key, iv).expect("Cipher creation failed");
    let mut decrypted_buffer = Vec::new();
    let mut offset = 0;

    while offset + 4 <= encrypted_data.len() {
        // Read the chunk length
        let chunk_len = u32::from_le_bytes(
            encrypted_data[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        // Validate chunk boundaries
        if offset + chunk_len > encrypted_data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid chunk length"));
        }

        // Decrypt the chunk
        let encrypted_chunk = &encrypted_data[offset..offset + chunk_len];
        let decrypted_chunk = cipher.clone().decrypt_vec(encrypted_chunk).expect("Decryption failed");
        decrypted_buffer.extend_from_slice(&decrypted_chunk);

        offset += chunk_len;
    }

    Ok(decrypted_buffer)
}

//**************************************************************** */
























//use openssl::symm::{Cipher, Crypter, Mode};

// pub fn encrypt_decrypt_test() {
//     let key = b"verysecretkey123";  // 16-byte key for AES-128
//     let iv = b"initialvector123";  // 16-byte IV
//     let data = b"Hello, world!";  // Known data to test

//     // Encrypt the data
//     let cipher = Cipher::aes_128_cbc();
//     let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(iv)).unwrap();
//     let mut encrypted_data = vec![0; data.len() + cipher.block_size()];
//     let count = encrypter.update(data, &mut encrypted_data).unwrap();
//     let rest = encrypter.finalize(&mut encrypted_data[count..]).unwrap();
//     encrypted_data.truncate(count + rest);

//     println!("Encrypted data: {:?}", encrypted_data);

//     // Decrypt the data
//     let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(iv)).unwrap();
//     let mut decrypted_data = vec![0; encrypted_data.len()];
//     let count = decrypter.update(&encrypted_data, &mut decrypted_data).unwrap();
//     let rest = decrypter.finalize(&mut decrypted_data[count..]).unwrap();
//     decrypted_data.truncate(count + rest);

//     println!("Decrypted data: {:?}", String::from_utf8(decrypted_data).unwrap());
// }