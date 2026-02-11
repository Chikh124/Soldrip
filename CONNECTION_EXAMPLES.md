# 🔌 Приклади підключення - різні сценарії

Після відповіді розробника, використайте один з цих варіантів.

---

## 📌 Варіант 1: Простий API (БЕЗ верифікації)

**Найпростіший варіант** - просто надсилаємо адресу гаманця

### Оновіть `src/soldrip.rs`:

```rust
pub async fn connect_wallet(wallet: &Wallet) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://soldrip.io/api/wallet/connect")  // ← Змініть URL
        .json(&serde_json::json!({
            "wallet_address": wallet.address,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Гаманець {} підключено", wallet.address);
        Ok(())
    } else {
        let error = response.text().await?;
        Err(anyhow!("Помилка підключення: {}", error))
    }
}

pub async fn claim(wallet: &Wallet) -> Result<f64> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://soldrip.io/api/wallet/claim")  // ← Змініть URL
        .json(&serde_json::json!({
            "wallet_address": wallet.address,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;
        let amount = data["amount"].as_f64().unwrap_or(0.0);
        Ok(amount)
    } else {
        Err(anyhow!("Помилка claim"))
    }
}
```

---

## 📌 Варіант 2: API з підписом (З верифікацією)

**З proof of ownership** - підписуємо повідомлення приватним ключем

### Оновіть `src/soldrip.rs`:

```rust
use solana_sdk::signature::Signer;

pub async fn connect_wallet(wallet: &Wallet) -> Result<()> {
    let client = reqwest::Client::new();

    // Створюємо повідомлення для підпису
    let timestamp = chrono::Utc::now().timestamp();
    let message = format!("SOLdrip connect: {}", timestamp);

    // Підписуємо повідомлення
    let keypair = crate::wallet::restore_keypair(&wallet.private_key)?;
    let signature = keypair.sign_message(message.as_bytes());
    let signature_base58 = bs58::encode(signature.as_ref()).into_string();

    let response = client
        .post("https://soldrip.io/api/wallet/connect")  // ← Змініть URL
        .json(&serde_json::json!({
            "wallet_address": wallet.address,
            "message": message,
            "signature": signature_base58,
            "timestamp": timestamp,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Гаманець {} підключено (з підписом)", wallet.address);
        Ok(())
    } else {
        let error = response.text().await?;
        Err(anyhow!("Помилка підключення: {}", error))
    }
}

pub async fn claim(wallet: &Wallet) -> Result<f64> {
    let client = reqwest::Client::new();

    // Підписуємо claim запит
    let timestamp = chrono::Utc::now().timestamp();
    let message = format!("SOLdrip claim: {}", timestamp);

    let keypair = crate::wallet::restore_keypair(&wallet.private_key)?;
    let signature = keypair.sign_message(message.as_bytes());
    let signature_base58 = bs58::encode(signature.as_ref()).into_string();

    let response = client
        .post("https://soldrip.io/api/wallet/claim")  // ← Змініть URL
        .json(&serde_json::json!({
            "wallet_address": wallet.address,
            "message": message,
            "signature": signature_base58,
            "timestamp": timestamp,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;
        let amount = data["amount"].as_f64().unwrap_or(0.0);
        Ok(amount)
    } else {
        Err(anyhow!("Помилка claim"))
    }
}
```

---

## 📌 Варіант 3: Через Phantom Wallet (браузерна автоматизація)

**Якщо немає API** - емулюємо поведінку користувача в браузері

### Встановіть Playwright замість headless_chrome:

Оновіть `Cargo.toml`:
```toml
# Замість headless_chrome
playwright = "0.0.20"
```

### Оновіть `src/soldrip.rs`:

```rust
use playwright::api::{BrowserType, Playwright};

pub async fn connect_wallet_browser(wallet: &Wallet) -> Result<()> {
    // Запускаємо браузер
    let playwright = Playwright::initialize().await?;
    let chromium = playwright.chromium();

    let browser = chromium
        .launcher()
        .headless(true)
        .launch()
        .await?;

    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;

    // Переходимо на сайт
    page.goto("https://soldrip.io").await?;

    // Чекаємо кнопку Connect Wallet
    let connect_btn = page
        .wait_for_selector("button:has-text('Connect')", None)
        .await?;

    connect_btn.click(None).await?;

    // TODO: Тут потрібна автоматизація Phantom popup
    // Це складно, тому краще використовувати API варіанти вище

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    browser.close().await?;
    Ok(())
}
```

**⚠️ УВАГА:** Браузерна автоматизація з Phantom дуже складна і ненадійна. Краще використовувати API!

---

## 📌 Варіант 4: З API Key/Token

Якщо потрібна авторизація:

```rust
pub async fn connect_wallet(wallet: &Wallet, api_key: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://soldrip.io/api/wallet/connect")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-API-Key", api_key)  // або інший header
        .json(&serde_json::json!({
            "wallet_address": wallet.address,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Unauthorized"))
    }
}
```

---

## 🧪 Тестування

Після вибору варіанта, протестуйте з 1 гаманцем:

```bash
# 1. Згенеруйте тестовий гаманець
cargo run --release
# Оберіть "1", введіть "1"

# 2. Спробуйте підключити
# Оберіть "2"

# 3. Перевірте логи
RUST_LOG=debug cargo run --release
```

---

## 📋 Checklist після отримання API інфо:

- [ ] Дізнатися точний endpoint URL
- [ ] Перевірити формат request/response
- [ ] Визначити чи потрібен підпис
- [ ] Обрати відповідний варіант (1, 2, 3 або 4)
- [ ] Оновити `src/soldrip.rs`
- [ ] Протестувати на 1 гаманці
- [ ] Запустити масово

---

## 💡 Рекомендація

**Варіант 1 або 2** - найкращі для автоматизації!
- ✅ Швидко
- ✅ Надійно
- ✅ Легко масштабувати

**Варіант 3** (браузер) - використовуйте тільки якщо немає API.
