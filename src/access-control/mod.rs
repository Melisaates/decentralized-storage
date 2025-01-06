use std::fs::File;
use std::io::{Read, Write};
use std::error::Error;
use crate::encryption::{decrypt_file};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData};
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Claims {
    sub: String,  // Kullanıcı ID'si
    exp: usize,   // Token geçerlilik süresi
}

#[derive(Deserialize)]
struct User {
    id: String,
    token: String,
}

fn check_access_token(token: &str) -> Result<Claims, Box<dyn Error>> {
    // Secret key - Gerçek projede secret key bir çevre değişkeni veya güvenli bir yerde saklanmalıdır.
    let secret_key = "your_secret_key"; 
    
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    // Token'ı decode ederek geçerliliğini kontrol et
    let token_data: TokenData<Claims> = decode::<Claims>(&token, &decoding_key, &validation)?;

    Ok(token_data.claims)
}

fn get_file_path(user_id: &str) -> String {
    // Kullanıcıya ait dosya yolunu belirleriz.
    format!("storage/{}.enc", user_id)
}

pub fn request_file_access(user: User) -> Result<Vec<u8>, Box<dyn Error>> {
    // Token doğrulaması
    let claims = match check_access_token(&user.token) {
        Ok(claims) => claims,
        Err(_) => return Err("Geçersiz token!".into()),
    };

    // Kullanıcının token'ındaki id ile kullanıcı id'sinin eşleşip eşleşmediğini kontrol et
    if claims.sub != user.id {
        return Err("Token ile dosya erişimi uyumsuz!".into());
    }

    // Dosya yolu belirleme
    let file_path = get_file_path(&user.id);

    // Şifreli dosyayı çözme
    let key = [0u8; 16]; // Anahtar, burada örnek olarak sıfırlanmış bir anahtar kullanıyoruz.
    let iv = [0u8; 16];  // IV de sıfırlanmış olarak varsayalım.
    
    let output_path = "output/decrypted_file";
    decrypt_file(&file_path, output_path, &key, &iv)?;

    // Şifresi çözülen dosyayı okuma
    let mut decrypted_file = File::open(output_path)?;
    let mut decrypted_data = Vec::new();
    decrypted_file.read_to_end(&mut decrypted_data)?;

    // Çözülmüş dosyayı kullanıcıya döndür
    Ok(decrypted_data)
}
