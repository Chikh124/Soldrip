# 🔧 Для розробника soldrip.io - додавання whitelist

Привіт! Це гайд для тебе (розробник soldrip.io), як додати можливість автоматизації без капчі.

---

## 🎯 Проблема

Зараз для claim потрібна капча, що ускладнює автоматизацію. Є кілька рішень:

---

## ✅ Рішення 1: API Key для автоматизації (РЕКОМЕНДУЮ)

Створи окремий endpoint для автоматизації з API key.

### Backend код (Node.js/Express):

```javascript
// middleware/auth.js
const AUTOMATION_API_KEY = process.env.AUTOMATION_API_KEY || "ваш_секретний_ключ";

function requireApiKey(req, res, next) {
  const apiKey = req.headers['x-api-key'];

  if (!apiKey || apiKey !== AUTOMATION_API_KEY) {
    return res.status(401).json({ error: 'Invalid API key' });
  }

  next();
}

module.exports = { requireApiKey };
```

```javascript
// routes/automation.js
const express = require('express');
const router = express.Router();
const { requireApiKey } = require('../middleware/auth');
const rateLimit = require('express-rate-limit');

// Rate limiting для automation endpoint
const automationLimiter = rateLimit({
  windowMs: 1 * 60 * 1000, // 1 хвилина
  max: 100, // максимум 100 claim на хвилину
  message: 'Too many requests from this API key'
});

// Automation endpoint БЕЗ капчі
router.post('/automation/claim',
  requireApiKey,
  automationLimiter,
  async (req, res) => {
    try {
      const { walletAddress, signature, message, timestamp, nonce, fingerprint } = req.body;

      // Перевірка підпису (так само як в основному endpoint)
      const isValid = await verifySignature(walletAddress, signature, message);
      if (!isValid) {
        return res.status(400).json({ error: 'Invalid signature' });
      }

      // Process claim БЕЗ капчі
      const result = await processClaim(walletAddress);

      return res.json({
        success: true,
        amount: result.amount,
        txId: result.txId
      });

    } catch (error) {
      console.error('Automation claim error:', error);
      return res.status(500).json({ error: error.message });
    }
});

module.exports = router;
```

```javascript
// app.js
const automationRoutes = require('./routes/automation');

// ... інші routes

// Додай automation routes
app.use('/api', automationRoutes);

// Основний claim endpoint (з капчею) залишається без змін
app.post('/api/claim', async (req, res) => {
  // Перевірка капчі для звичайних користувачів
  await verifyCaptcha(req.body.captchaToken);
  // ... решта логіки
});
```

### .env
```bash
AUTOMATION_API_KEY=your_secret_key_here_change_this_12345
```

**Переваги:**
- ✅ Не ламає існуючий функціонал
- ✅ Контроль через API key
- ✅ Rate limiting по ключу
- ✅ Легко відключити якщо потрібно

---

## ✅ Рішення 2: Whitelist адрес

Додай whitelist для конкретних адрес.

```javascript
// config/whitelist.js
const WHITELISTED_ADDRESSES = [
  // Адреси для автоматизації
  '7XaL9ZkvPrVu...',
  'ABe2suEH8paL...',
  // ... додай всі адреси
];

function isWhitelisted(address) {
  return WHITELISTED_ADDRESSES.includes(address);
}

module.exports = { isWhitelisted };
```

```javascript
// routes/claim.js
const { isWhitelisted } = require('../config/whitelist');

app.post('/api/claim', async (req, res) => {
  const { walletAddress, captchaToken } = req.body;

  // Skip captcha для whitelist
  if (!isWhitelisted(walletAddress)) {
    // Перевірка капчі тільки для не-whitelisted
    const captchaValid = await verifyCaptcha(captchaToken);
    if (!captchaValid) {
      return res.status(400).json({ error: 'Invalid captcha' });
    }
  } else {
    console.log(`Skipping captcha for whitelisted address: ${walletAddress}`);
  }

  // Process claim
  const result = await processClaim(walletAddress);
  return res.json(result);
});
```

---

## ✅ Рішення 3: Умовна капча (найгнучкіше)

Капча тільки для Level 1 користувачів або нових адрес.

```javascript
app.post('/api/claim', async (req, res) => {
  const { walletAddress, captchaToken } = req.body;

  const wallet = await getWalletInfo(walletAddress);

  // Капча потрібна тільки для:
  const needsCaptcha =
    wallet.level === 1 ||                    // Level 1 users
    wallet.totalClaimed < 10 ||              // Перші 10 claim
    wallet.createdAt > Date.now() - 86400000; // Нові (< 24 год)

  if (needsCaptcha) {
    const captchaValid = await verifyCaptcha(captchaToken);
    if (!captchaValid) {
      return res.status(400).json({ error: 'Invalid captcha' });
    }
  }

  const result = await processClaim(walletAddress);
  return res.json(result);
});
```

---

## 📊 Порівняння рішень:

| Рішення | Складність | Безпека | Гнучкість | Рекомендація |
|---------|-----------|---------|-----------|--------------|
| API Key | ⭐⭐ Середня | ⭐⭐⭐ Висока | ⭐⭐⭐ Висока | ✅ Найкраще |
| Whitelist | ⭐ Легко | ⭐⭐ Середня | ⭐ Низька | ✅ Швидко |
| Умовна капча | ⭐⭐⭐ Складно | ⭐⭐⭐ Висока | ⭐⭐⭐⭐ Дуже висока | ✅ Для продакшну |

---

## 🚀 Що треба зробити:

### Мінімальний варіант (5 хвилин):

1. Додай whitelist адрес у код
2. Перевіряй whitelist перед капчею
3. Deploy
4. Готово! ✅

### Рекомендований варіант (15 хвилин):

1. Створи automation endpoint з API key
2. Додай rate limiting
3. Згенеруй API key і дай другу
4. Deploy
5. Profit! 💰

---

## 🔐 API Key генерація:

```bash
# Згенеруй випадковий API key:
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"

# Або:
openssl rand -hex 32
```

Приклад output:
```
f4a8b2c9d7e6f1a3b5c8d9e2f7a1b4c6d8e3f9a2b5c7d1e4f8a3b6c9d2e5f7a1
```

---

## 📝 Після імплементації:

Надай другу:
1. **API Key** (якщо обрав рішення 1)
2. **Endpoint URL** (наприклад: `POST /api/automation/claim`)
3. **Rate limits** (скільки requests/min дозволено)

---

## 💡 Додаткові рекомендації:

### Логування:
```javascript
// Логуй automation claims окремо
console.log(`[AUTOMATION] Claim from ${walletAddress}: ${amount} SOL`);
```

### Статистика:
```javascript
// Додай окрему статистику для automation
const automationStats = {
  totalClaims: 0,
  totalAmount: 0,
  lastClaimAt: null
};
```

### Monitoring:
```javascript
// Алерт якщо занадто багато automation claims
if (automationClaimsPerMinute > 200) {
  sendAlert('High automation activity detected');
}
```

---

## ❓ Питання?

Якщо щось незрозуміло або потрібна допомога з імплементацією - пиши!

**Код простий і займе 15-20 хвилин максимум! 🚀**
