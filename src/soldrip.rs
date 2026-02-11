use anyhow::{anyhow, Result};
use log::{info, error};
use sha2::{Sha256, Digest};

use crate::wallet::Wallet;

const SOLDRIP_URL: &str = "https://soldrip.io";
const API_ENDPOINT: &str = "https://soldrip.io/api/auth/connect";

/// Генерує fingerprint для сесії
fn generate_fingerprint(wallet_address: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wallet_address.as_bytes());
    hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
    hex::encode(hasher.finalize())
}

/// Створює повідомлення для підпису (формат soldrip.io)
fn create_message(wallet_address: &str, timestamp: i64, action: &str) -> String {
    format!(
        "SOLdrip\n\nAction: {}\nWallet: {}\nTimestamp: {}\nNonce: 0",
        action, wallet_address, timestamp
    )
}

/// Підписує повідомлення приватним ключем
fn sign_message(private_key: &str, message: &str) -> Result<String> {
    use crate::wallet::restore_keypair;
    use ed25519_dalek::Signer;

    let signing_key = restore_keypair(private_key)?;
    let signature = signing_key.sign(message.as_bytes());

    Ok(bs58::encode(signature.to_bytes()).into_string())
}

/// Підключає гаманець до SOLdrip через API
pub async fn connect_wallet(wallet: &Wallet) -> Result<()> {
    info!("🔗 Підключаємо гаманець: {}", wallet.address);

    let client = reqwest::Client::new();

    // Генеруємо timestamp (в мілісекундах)
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Створюємо повідомлення для підпису
    let message = create_message(&wallet.address, timestamp, "connect");

    // Підписуємо повідомлення
    let signature = sign_message(&wallet.private_key, &message)?;

    // Генеруємо fingerprint
    let fingerprint = generate_fingerprint(&wallet.address);

    info!("  📝 Message: {}", message.replace('\n', "\\n"));
    info!("  ✍️  Signature: {}...", &signature[..20]);

    // Відправляємо запит
    let response = client
        .post(API_ENDPOINT)
        .header("accept", "*/*")
        .header("content-type", "application/json")
        .header("origin", "https://soldrip.io")
        .header("referer", "https://soldrip.io/")
        .header("x-fingerprint", fingerprint)
        .json(&serde_json::json!({
            "walletAddress": wallet.address,
            "referralCode": null,
            "signature": signature,
            "message": message,
            "timestamp": timestamp
        }))
        .send()
        .await?;

    let status = response.status();

    if status.is_success() {
        let response_text = response.text().await?;
        info!("✅ Гаманець успішно підключено!");
        info!("  Response: {}", response_text);
        Ok(())
    } else {
        let error_text = response.text().await?;
        error!("❌ Помилка підключення (HTTP {}): {}", status, error_text);
        Err(anyhow!("Failed to connect wallet: {} - {}", status, error_text))
    }
}

/// Генерує nonce (хеш для claim)
fn generate_nonce() -> String {
    use rand::Rng;
    let mut hasher = Sha256::new();
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    hasher.update(&random_bytes);
    hex::encode(hasher.finalize())
}

/// Створює повідомлення для claim з nonce
fn create_claim_message(wallet_address: &str, timestamp: i64, nonce: &str) -> String {
    format!(
        "SOLdrip\n\nAction: claim\nWallet: {}\nTimestamp: {}\nNonce: {}",
        wallet_address, timestamp, nonce
    )
}

/// Виконує claim для гаманця (БЕЗ капчі - для тестування)
pub async fn claim(wallet: &Wallet) -> Result<f64> {
    info!("💰 Виконуємо claim для: {}", wallet.address);
    info!("⚠️  УВАГА: Для claim потрібна капча! Використайте claim_with_captcha()");

    Err(anyhow!("Claim requires CAPTCHA token. Use claim_with_captcha() instead"))
}

/// Виконує claim з капчею
pub async fn claim_with_captcha(wallet: &Wallet, captcha_token: &str) -> Result<f64> {
    info!("💰 Виконуємо claim для: {}", wallet.address);

    let client = reqwest::Client::new();

    // Генеруємо timestamp
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Генеруємо nonce
    let nonce = generate_nonce();

    // Створюємо повідомлення для claim з nonce
    let message = create_claim_message(&wallet.address, timestamp, &nonce);

    // Підписуємо повідомлення
    let signature = sign_message(&wallet.private_key, &message)?;

    // Генеруємо fingerprint (або використовуємо збережений)
    let fingerprint = generate_fingerprint(&wallet.address);

    info!("  📝 Nonce: {}", &nonce[..20]);
    info!("  ✍️  Signature: {}...", &signature[..20]);

    let response = client
        .post("https://soldrip.io/api/claim")
        .header("accept", "*/*")
        .header("content-type", "application/json")
        .header("origin", "https://soldrip.io")
        .header("referer", "https://soldrip.io/")
        .json(&serde_json::json!({
            "walletAddress": wallet.address,
            "signature": signature,
            "message": message,
            "timestamp": timestamp,
            "nonce": nonce,
            "fingerprint": fingerprint,
            "captchaToken": captcha_token
        }))
        .send()
        .await?;

    let status = response.status();

    if status.is_success() {
        let data: serde_json::Value = response.json().await?;

        // Отримуємо amount з відповіді
        let amount = data["amount"]
            .as_f64()
            .unwrap_or(0.0);

        info!("✅ Успішно claimed {} SOL", amount);
        Ok(amount)
    } else {
        let error_text = response.text().await?;
        error!("❌ Помилка claim (HTTP {}): {}", status, error_text);
        Err(anyhow!("Failed to claim: {} - {}", status, error_text))
    }
}

/// Отримує поточний баланс гаманця на SOLdrip
pub async fn get_balance(wallet: &Wallet) -> Result<f64> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/balance/{}", SOLDRIP_URL, wallet.address))
        .send()
        .await?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;
        let balance = data["balance"]
            .as_f64()
            .ok_or_else(|| anyhow!("Invalid balance format"))?;

        Ok(balance)
    } else {
        Err(anyhow!("Failed to get balance"))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sign_message() {
        use crate::wallet::generate_wallet;

        let wallet = generate_wallet().unwrap();
        let signature = sign_message(&wallet.private_key, "test").unwrap();
        assert!(!signature.is_empty());
    }
}
