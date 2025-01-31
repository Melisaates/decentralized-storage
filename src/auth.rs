use std::collections::HashMap;
use sha2::{Sha256, Digest}; // Şifreleme için.
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

/// Kullanıcı verilerini temsil eden yapı.
pub struct User {
    pub user_id: String,
    pub password_hash: String,
    pub token: Option<String>, // Giriş yapılmışsa bir token saklar.
}

/// Kullanıcı yönetimi ve doğrulama sistemi.
pub struct AuthSystem {
    users: HashMap<String, User>, // Kullanıcı adı ve kullanıcı eşleştirmesi.
}

impl AuthSystem {
    /// Yeni bir `AuthSystem` oluşturur.
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Yeni kullanıcı kaydeder.
    /// 
    /// # Arguments
    /// - `user_id`: Kullanıcı adı.
    /// - `password`: Kullanıcı şifresi.
    /// 
    /// # Returns
    /// - İşlem başarılıysa `true`, değilse `false`.
    pub fn register_user(&mut self, user_id: &str, password: &str) -> bool {
        if self.users.contains_key(user_id) {
            return false; // Kullanıcı adı zaten mevcut.
        }

        let password_hash = Self::hash_password(password);
        self.users.insert(
            user_id.to_string(),
            User {
                user_id: user_id.to_string(),
                password_hash,
                token: None,
            },
        );
        true
    }

    /// Kullanıcı giriş yapar ve bir oturum token'ı döner.
    /// 
    /// # Arguments
    /// - `user_id`: Kullanıcı adı.
    /// - `password`: Kullanıcı şifresi.
    /// 
    /// # Returns
    /// - Token (String) veya giriş başarısızsa `None`.
    pub fn login(&mut self, user_id: &str, password: &str) -> Option<String> {
        if let Some(user) = self.users.get_mut(user_id) {
            if user.password_hash == Self::hash_password(password) {
                let token = Self::generate_token();
                user.token = Some(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// Kullanıcıyı oturumdan çıkarır.
    /// 
    /// # Arguments
    /// - `user_id`: Kullanıcı adı.
    pub fn logout(&mut self, user_id: &str) -> bool {
        if let Some(user) = self.users.get_mut(user_id) {
            user.token = None;
            return true;
        }
        false
    }

    /// Token doğrulama işlemi.
    /// 
    /// # Arguments
    /// - `user_id`: Kullanıcı adı.
    /// - `token`: Doğrulanacak token.
    /// 
    /// # Returns
    /// - Token geçerliyse `true`, değilse `false`.
    pub fn validate_token(&self, user_id: &str, token: &str) -> bool {
        if let Some(user) = self.users.get(user_id) {
            return user.token.as_deref() == Some(token);
        }
        false
    }

    /// Şifre hash'ini oluşturur.
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Rastgele bir oturum token'ı oluşturur.
    fn generate_token() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_register_and_login() {
//         let mut auth_system = AuthSystem::new();

//         assert!(auth_system.register_user("user1", "password1"));
//         assert!(!auth_system.register_user("user1", "password2")); // Kullanıcı adı zaten var.

//         let token = auth_system.login("user1", "password1");
//         assert!(token.is_some());

//         let invalid_token = auth_system.login("user1", "wrong_password");
//         assert!(invalid_token.is_none());

//         let valid_token = token.unwrap();
//         assert!(auth_system.validate_token("user1", &valid_token));

//         auth_system.logout("user1");
//         assert!(!auth_system.validate_token("user1", &valid_token)); // Token artık geçerli değil.
//     }
// }
