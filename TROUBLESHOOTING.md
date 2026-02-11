# 🔧 Troubleshooting Guide

## Поширені проблеми та рішення

### 1. Rust не встановлений

**Проблема:**
```
cargo: command not found
```

**Рішення:**
```bash
# Windows
# Завантажте rustup-init.exe з https://rustup.rs/

# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

### 2. Помилки компіляції

**Проблема:**
```
error: failed to compile soldrip_automation
```

**Рішення:**
```bash
# Оновіть Rust до останньої версії
rustup update

# Очистіть кеш та перебудуйте
cargo clean
cargo build --release
```

---

### 3. Помилка "failed to connect"

**Можливі причини:**
- ❌ API endpoint не існує
- ❌ Неправильний формат запиту
- ❌ CORS блокування
- ❌ Rate limiting

**Рішення:**

1. **Перевірте endpoint:**
   ```bash
   curl https://soldrip.io/api/wallet/connect
   ```

2. **Увімкніть debug логи:**
   ```bash
   RUST_LOG=debug cargo run
   ```

3. **Перевірте код в `src/soldrip.rs`:**
   - Правильний URL?
   - Правильний формат JSON?
   - Чи потрібні headers?

---

### 4. CSV файл не створюється

**Проблема:**
```
Permission denied: wallets.csv
```

**Рішення:**
```bash
# Перевірте права доступу
ls -la wallets.csv

# Видаліть старий файл якщо потрібно
rm wallets.csv

# Запустіть з правами адміністратора (Windows)
# Клік правою кнопкою → Run as Administrator
```

---

### 5. Повільна генерація гаманців

**Проблема:** Генерація 1000 гаманців займає багато часу

**Оптимізація:**

Відредагуйте `src/main.rs`, додайте паралелізацію:

```rust
use rayon::prelude::*;

async fn generate_wallets() -> Result<()> {
    // ... existing code ...

    let wallets: Vec<_> = (0..count)
        .into_par_iter()  // Parallel iterator
        .map(|_| wallet::generate_wallet())
        .collect::<Result<Vec<_>>>()?;

    // ... rest of code ...
}
```

Додайте в `Cargo.toml`:
```toml
rayon = "1.8"
```

---

### 6. "Too many requests" error

**Проблема:** Rate limiting на API

**Рішення:**

Збільште затримку між запитами в `src/main.rs`:

```rust
// Замість 1000ms
tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

// Використайте 2000-3000ms
tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
```

---

### 7. Браузерна автоматизація не працює

**Проблема:**
```
Could not launch Chrome
```

**Рішення:**

1. **Встановіть Chrome/Chromium:**
   ```bash
   # Ubuntu/Debian
   sudo apt install chromium-browser

   # macOS
   brew install chromium

   # Windows - завантажте Chrome
   ```

2. **Використайте API замість браузера:**
   - Закоментуйте код з `headless_chrome`
   - Використовуйте тільки `reqwest` HTTP запити

---

### 8. Неправильний баланс

**Проблема:** Баланс не оновлюється після claim

**Рішення:**

Перевірте функцію `claim` в `src/soldrip.rs`:

```rust
// Переконайтесь що парсить правильне поле
let amount = data["amount"]
    .as_f64()
    .ok_or_else(|| anyhow!("Invalid response format"))?;

// Додайте логування для дебагу
println!("Response: {:?}", data);
```

---

### 9. Втрата приватних ключів

**КРИТИЧНО!** Якщо ви втратили `wallets.csv`:

- ❌ Приватні ключі **неможливо** відновити
- ✅ Створіть резервні копії:
  ```bash
  # Копія з timestamp
  cp wallets.csv "wallets_backup_$(date +%Y%m%d_%H%M%S).csv"

  # Або використайте git
  git init
  git add wallets.csv
  git commit -m "Backup wallets"
  ```

---

### 10. Перевірка з'єднання з API

**Тестовий скрипт:**

```bash
# test_api.sh
#!/bin/bash

echo "Testing SOLdrip API..."

# Test connect
echo "1. Testing /api/wallet/connect"
curl -X POST https://soldrip.io/api/wallet/connect \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"TEST123"}' \
  -v

echo ""
echo "2. Testing /api/wallet/balance"
curl https://soldrip.io/api/wallet/balance/TEST123 -v

echo ""
echo "3. Testing /api/wallet/claim"
curl -X POST https://soldrip.io/api/wallet/claim \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"TEST123"}' \
  -v
```

---

## 📞 Додаткова допомога

Якщо проблема не вирішена:

1. **Увімкніть повне логування:**
   ```bash
   RUST_LOG=trace cargo run 2>&1 | tee debug.log
   ```

2. **Перевірте версії:**
   ```bash
   rustc --version
   cargo --version
   ```

3. **Створіть issue** з:
   - Повним текстом помилки
   - Файлом `debug.log`
   - Версією Rust
   - Операційною системою

---

## 🐛 Debug режими

### Minimal test:
```bash
# Згенеруйте тільки 1 гаманець для тесту
cargo run --release
# Оберіть "1", введіть "1"
```

### API test without browser:
У `src/soldrip.rs` закоментуйте всі виклики `headless_chrome`

### Dry run mode:
Додайте прапорець в код для тестування без реальних запитів

---

**Потрібна допомога?** Напишіть розробнику soldrip.io!
