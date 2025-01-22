use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

// JWT için kullanılan Claim yapısı
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Kullanıcı bilgisi
    pub exp: u64,    // Expiration (geçerlilik süresi)
}

// TokenManager yapısı, JWT işlemleri için kullanılan servis
pub struct TokenManager {
    secret: String, // JWT için kullanılan gizli anahtar
}

impl TokenManager {
    // Yeni TokenManager oluştur
    pub fn new(secret: String) -> Self {
        TokenManager { secret }
    }

    // Token oluştur
    pub fn create_token(&self, user_id: &str) -> Result<String, String> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "SystemTime error".to_string())?
            .as_secs() + 3600; // 1 saat geçerli olacak

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
        };

        let header = Header::default();
        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        ).map_err(|_| "Token encoding error".to_string()) // Hata yönetimi
    }

    // Token doğrula
    pub fn validate_token(&self, token: &str) -> Result<bool, String> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let validation = Validation::default();
        
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
