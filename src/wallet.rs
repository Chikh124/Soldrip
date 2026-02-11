use anyhow::Result;
use serde::{Deserialize, Serialize};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

/// Структура для зберігання інформації про гаманець
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Публічна адреса гаманця
    pub address: String,

    /// Приватний ключ (base58)
    pub private_key: String,

    /// Статус (pending, connected, failed)
    pub status: String,

    /// Поточний баланс
    pub balance: f64,

    /// Дата створення
    pub created_at: String,

    /// Дата останнього claim
    pub last_claim: Option<String>,
}

/// Генерує новий Solana гаманець
pub fn generate_wallet() -> Result<Wallet> {
    use rand::RngCore;

    // Генеруємо випадкові байти для приватного ключа
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);

    // Створюємо signing key
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Отримуємо публічну адресу (base58 від публічного ключа)
    let address = bs58::encode(verifying_key.as_bytes()).into_string();

    // Конвертуємо приватний ключ в base58 (Solana формат: 64 байти = 32 приватний + 32 публічний)
    let mut full_key = [0u8; 64];
    full_key[..32].copy_from_slice(&signing_key.to_bytes());
    full_key[32..].copy_from_slice(verifying_key.as_bytes());
    let private_key = bs58::encode(full_key).into_string();

    // Поточний час
    let created_at = chrono::Utc::now().to_rfc3339();

    Ok(Wallet {
        address,
        private_key,
        status: "pending".to_string(),
        balance: 0.0,
        created_at,
        last_claim: None,
    })
}

/// Відновлює keypair з приватного ключа
pub fn restore_keypair(private_key: &str) -> Result<SigningKey> {
    let bytes = bs58::decode(private_key).into_vec()?;

    // Solana формат: 64 байти (перші 32 - приватний ключ)
    if bytes.len() == 64 {
        let key_bytes: [u8; 32] = bytes[..32].try_into()
            .map_err(|_| anyhow::anyhow!("Invalid private key"))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }
    // Якщо 32 байти - старий формат
    else if bytes.len() == 32 {
        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid private key"))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }
    else {
        Err(anyhow::anyhow!("Invalid private key length: expected 32 or 64 bytes, got {}", bytes.len()))
    }
}

/// Отримує публічну адресу з приватного ключа
pub fn get_address(private_key: &str) -> Result<String> {
    let signing_key = restore_keypair(private_key)?;
    let verifying_key = signing_key.verifying_key();
    Ok(bs58::encode(verifying_key.as_bytes()).into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generation() {
        let wallet = generate_wallet().unwrap();
        assert!(!wallet.address.is_empty());
        assert!(!wallet.private_key.is_empty());
        assert_eq!(wallet.status, "pending");
        assert_eq!(wallet.balance, 0.0);
    }

    #[test]
    fn test_keypair_restore() {
        let wallet = generate_wallet().unwrap();
        let signing_key = restore_keypair(&wallet.private_key).unwrap();
        let address = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        assert_eq!(address, wallet.address);
    }
}
