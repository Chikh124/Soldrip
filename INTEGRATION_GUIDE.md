# 🔌 Інструкція з інтеграції з soldrip.io API

Цей документ допоможе вашому другу інтегрувати автоматизацію з backend soldrip.io.

## 📡 Рекомендована структура API

### 1. POST `/api/wallet/connect`

**Призначення:** Підключення нового гаманця до системи

**Request:**
```json
{
  "wallet_address": "7XaL9...",
  "signature": "base58_signature",  // Опціонально - для верифікації власності
  "timestamp": "2024-02-11T12:00:00Z"
}
```

**Response (Success):**
```json
{
  "success": true,
  "wallet_address": "7XaL9...",
  "status": "connected",
  "balance": 0.0,
  "drip_rate": 0.15,  // SOL на місяць
  "next_claim_at": "2024-02-12T12:00:00Z"
}
```

**Response (Error):**
```json
{
  "success": false,
  "error": "Wallet already connected"
}
```

---

### 2. POST `/api/wallet/claim`

**Призначення:** Claim накопичених SOL

**Request:**
```json
{
  "wallet_address": "7XaL9...",
  "signature": "base58_signature"  // Опціонально
}
```

**Response (Success):**
```json
{
  "success": true,
  "amount": 0.005,  // Claimed amount
  "balance": 0.145,  // Залишок
  "transaction_id": "5xJ2...",  // Solana TX hash
  "next_claim_at": "2024-02-12T12:00:00Z"
}
```

**Response (Error):**
```json
{
  "success": false,
  "error": "Nothing to claim yet",
  "next_claim_at": "2024-02-12T12:00:00Z"
}
```

---

### 3. GET `/api/wallet/balance/:address`

**Призначення:** Перевірка поточного балансу

**Response:**
```json
{
  "wallet_address": "7XaL9...",
  "balance": 0.145,
  "total_claimed": 12.5,
  "connected_at": "2024-01-01T00:00:00Z",
  "last_claim_at": "2024-02-11T12:00:00Z",
  "next_claim_at": "2024-02-12T12:00:00Z"
}
```

---

### 4. GET `/api/wallet/stats`

**Призначення:** Загальна статистика (опціонально)

**Response:**
```json
{
  "total_wallets": 1000,
  "total_distributed": 15000.5,
  "active_drips": 850,
  "daily_distribution": 5.0
}
```

---

## 🔐 Опціональна верифікація власності

Якщо потрібно підтверджувати, що користувач володіє гаманцем:

### Процес верифікації:

1. **Client генерує підпис:**
   ```rust
   let message = format!("SOLdrip connect: {}", timestamp);
   let signature = keypair.sign_message(message.as_bytes());
   ```

2. **Server верифікує підпис:**
   ```typescript
   import { PublicKey } from '@solana/web3.js';
   import nacl from 'tweetnacl';

   function verifySignature(
     walletAddress: string,
     message: string,
     signature: string
   ): boolean {
     const publicKey = new PublicKey(walletAddress);
     const messageBytes = Buffer.from(message);
     const signatureBytes = bs58.decode(signature);

     return nacl.sign.detached.verify(
       messageBytes,
       signatureBytes,
       publicKey.toBytes()
     );
   }
   ```

---

## 🚦 Rate Limiting

Рекомендації для запобігання зловживанню:

```javascript
// Express.js приклад
const rateLimit = require('express-rate-limit');

const connectLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 хвилин
  max: 100, // максимум 100 підключень
  message: 'Too many connection attempts'
});

app.post('/api/wallet/connect', connectLimiter, handleConnect);
```

---

## 💾 Database Schema

Рекомендована структура БД:

```sql
CREATE TABLE wallets (
    id SERIAL PRIMARY KEY,
    address VARCHAR(44) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    balance DECIMAL(18, 9) DEFAULT 0,
    total_claimed DECIMAL(18, 9) DEFAULT 0,
    connected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_claim_at TIMESTAMP,
    next_claim_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_wallet_address ON wallets(address);
CREATE INDEX idx_wallet_status ON wallets(status);
CREATE INDEX idx_next_claim ON wallets(next_claim_at);

-- Для відстеження транзакцій
CREATE TABLE claims (
    id SERIAL PRIMARY KEY,
    wallet_address VARCHAR(44) NOT NULL,
    amount DECIMAL(18, 9) NOT NULL,
    transaction_id VARCHAR(88),
    claimed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_address) REFERENCES wallets(address)
);
```

---

## 🔄 Автоматичний Drip Механізм

### Варіант 1: Cron Job (рекомендовано)

```javascript
// cron-drip.js
const cron = require('node-cron');

// Виконується кожну хвилину
cron.schedule('* * * * *', async () => {
  const wallets = await getActiveWallets();
  const dripAmount = 0.15 / (30 * 24 * 60); // 0.15 SOL/місяць → за хвилину

  for (const wallet of wallets) {
    await incrementBalance(wallet.address, dripAmount);
  }
});
```

### Варіант 2: При кожному запиті (менш точно)

```javascript
function calculateBalance(wallet) {
  const minutesSinceConnection =
    (Date.now() - wallet.connected_at) / 1000 / 60;

  const dripPerMinute = 0.15 / (30 * 24 * 60);
  const earned = minutesSinceConnection * dripPerMinute;

  return wallet.initial_balance + earned - wallet.total_claimed;
}
```

---

## 🧪 Тестування API

### Використайте curl для тестів:

```bash
# Connect wallet
curl -X POST https://soldrip.io/api/wallet/connect \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "7XaL9..."}'

# Check balance
curl https://soldrip.io/api/wallet/balance/7XaL9...

# Claim
curl -X POST https://soldrip.io/api/wallet/claim \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "7XaL9..."}'
```

---

## 🔧 Інтеграція з Rust кодом

Після того, як API готове, оновіть [`src/soldrip.rs`](src/soldrip.rs):

```rust
// Замініть URL на актуальний
const SOLDRIP_URL: &str = "https://api.soldrip.io";  // або ваш домен

// У функції connect_via_api - оновіть endpoint
.post(format!("{}/api/wallet/connect", SOLDRIP_URL))

// У функції claim - оновіть endpoint
.post(format!("{}/api/wallet/claim", SOLDRIP_URL))
```

---

## 📞 Підтримка

Після імплементації API, надайте:
- ✅ Base URL API
- ✅ Endpoint structure
- ✅ Auth requirements (якщо є)
- ✅ Rate limits
- ✅ Error codes

---

**Happy coding! 🚀**
