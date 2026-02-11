use anyhow::{anyhow, Result};
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

const CAPTCHA_API_URL: &str = "https://2captcha.com/in.php";
const CAPTCHA_RESULT_URL: &str = "https://2captcha.com/res.php";

/// Конфігурація для 2Captcha
pub struct CaptchaConfig {
    pub api_key: String,
    pub site_key: String, // reCAPTCHA site key від soldrip.io
}

impl CaptchaConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("CAPTCHA_API_KEY")
            .map_err(|_| anyhow!("CAPTCHA_API_KEY не знайдено. Створіть файл .env"))?;

        // Site key для soldrip.io (знайдено з Network запитів)
        let site_key = std::env::var("CAPTCHA_SITE_KEY")
            .unwrap_or_else(|_| "6LeYdFcsAAAAACXSB7EiVlXm6Wp2F1bkESWKnhkg".to_string());

        Ok(Self { api_key, site_key })
    }
}

/// Розв'язує reCAPTCHA v3 через 2Captcha
pub async fn solve_captcha(config: &CaptchaConfig) -> Result<String> {
    info!("🤖 Надсилаємо капчу на розв'язання...");
    info!("  🔑 Site Key: {}", config.site_key);

    let client = reqwest::Client::new();

    // Крок 1: Надсилаємо капчу (reCAPTCHA v2, БЕЗ version/action/min_score!)
    let submit_url = format!(
        "{}?key={}&method=userrecaptcha&googlekey={}&pageurl={}&json=1",
        CAPTCHA_API_URL,
        config.api_key,
        config.site_key,
        "https://soldrip.io"
    );

    let submit_response = client
        .get(&submit_url)
        .send()
        .await?;

    let submit_data: serde_json::Value = submit_response.json().await?;

    if submit_data["status"].as_i64() != Some(1) {
        let error = submit_data["request"].as_str().unwrap_or("Unknown error");
        return Err(anyhow!("2Captcha submit failed: {}", error));
    }

    let captcha_id = submit_data["request"]
        .as_str()
        .ok_or_else(|| anyhow!("No captcha ID received"))?;

    info!("  📝 Captcha ID: {}", captcha_id);
    info!("  ⏳ Чекаємо розв'язання (зазвичай 15-30 секунд)...");

    // Крок 2: Чекаємо розв'язання
    for attempt in 1..=30 {
        sleep(Duration::from_secs(5)).await;

        let result_url = format!(
            "{}?key={}&action=get&id={}&json=1",
            CAPTCHA_RESULT_URL,
            config.api_key,
            captcha_id
        );

        let result_response = client
            .get(&result_url)
            .send()
            .await?;

        let result_data: serde_json::Value = result_response.json().await?;

        match result_data["status"].as_i64() {
            Some(1) => {
                // Успішно розв'язано!
                let token = result_data["request"]
                    .as_str()
                    .ok_or_else(|| anyhow!("No token in response"))?;

                info!("✅ Капча розв'язана за {} секунд!", attempt * 5);
                info!("  🎫 Token (перші 50 символів): {}...", &token[..token.len().min(50)]);
                return Ok(token.to_string());
            }
            Some(0) => {
                let request = result_data["request"].as_str().unwrap_or("");
                if request == "CAPCHA_NOT_READY" {
                    if attempt % 3 == 0 {
                        info!("  ⏳ Ще чекаємо... ({}/30 спроб)", attempt);
                    }
                    continue;
                } else {
                    return Err(anyhow!("2Captcha error: {}", request));
                }
            }
            _ => {
                return Err(anyhow!("Unexpected response from 2Captcha"));
            }
        }
    }

    Err(anyhow!("Timeout: капча не розв'язана за 150 секунд"))
}

/// Отримує баланс 2Captcha акаунту
pub async fn get_balance(api_key: &str) -> Result<f64> {
    let client = reqwest::Client::new();

    let url = format!(
        "{}?key={}&action=getbalance&json=1",
        CAPTCHA_RESULT_URL,
        api_key
    );

    let response = client.get(&url).send().await?;
    let data: serde_json::Value = response.json().await?;

    if data["status"].as_i64() == Some(1) {
        let balance = data["request"]
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
    #[ignore] // Тільки для мануального тестування
    async fn test_solve_captcha() {
        let config = CaptchaConfig::from_env().unwrap();
        let token = solve_captcha(&config).await.unwrap();
        assert!(!token.is_empty());
    }
}
