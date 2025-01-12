use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

//jwt ile token oluşturma ve doğrulama işlemleri



#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Kullanıcı bilgisi
    pub exp: usize,  // Expiration (geçerlilik süresi)
}

pub struct TokenManager {
    secret: String, // JWT için kullanılan gizli anahtar
}

impl TokenManager {
    pub fn new(secret: String) -> Self {
        TokenManager { secret }
    }

    // Token oluştur
    pub fn create_token(&self, user_id: &str) -> String {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; //1 saat geçerli olacak

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
        };

        let header = Header::default();
        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .unwrap()
    }

    // Token doğrula
    pub fn validate_token(&self, token: &str) -> bool {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let validation = Validation::default();
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}














/*Kullanıcı giriş yaptığında bir token oluşturulur
Bu token kullanıcıya verilir
Kullanıcı her istek yaptığında bu token'ı gönderir
Sunucu token'ı doğrulayarak kullanıcının kimliğini teyit eder

Her token 1 saat geçerli kalacak şekilde ayarlanmış ve güvenli bir şekilde şifrelenmiş durumda. */