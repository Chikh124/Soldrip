mod wallet;
mod storage;
mod soldrip;
mod captcha;

use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // Завантажуємо .env файл
    dotenv::dotenv().ok();

    env_logger::init();

    println!("{}", "🚀 SOLdrip Automation Tool".bright_green().bold());
    println!("{}", "=" .repeat(50));
    println!();

    loop {
        println!("{}", "\nОберіть дію:".bright_cyan());
        println!("1. Генерувати нові гаманці");
        println!("2. Підключити гаманці до SOLdrip");
        println!("3. Зробити claim для всіх гаманців");
        println!("4. Показати статистику");
        println!("5. 🔄 Автоматичний режим (Auto-Claim Loop)");
        println!("6. Вийти");
        print!("\nВаш вибір: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => generate_wallets().await?,
            "2" => connect_wallets().await?,
            "3" => claim_all().await?,
            "4" => show_stats().await?,
            "5" => auto_claim_loop().await?,
            "6" => {
                println!("{}", "👋 До побачення!".bright_yellow());
                break;
            }
            _ => println!("{}", "❌ Невірний вибір!".red()),
        }
    }

    Ok(())
}

async fn generate_wallets() -> Result<()> {
    print!("Скільки гаманців згенерувати? ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let count: usize = input.trim().parse()?;

    println!("\n{}", format!("🔐 Генеруємо {} гаманців...", count).bright_blue());

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut wallets = Vec::new();
    for _ in 0..count {
        let wallet = wallet::generate_wallet()?;
        wallets.push(wallet);
        pb.inc(1);
    }
    pb.finish_with_message("✅ Готово!");

    storage::save_wallets(&wallets)?;

    println!("\n{}", format!("✨ Успішно згеновано {} гаманців!", count).bright_green());
    println!("📄 Збережено у файл: {}", "wallets.xlsx".bright_yellow());

    Ok(())
}

async fn connect_wallets() -> Result<()> {
    println!("\n{}", "🔗 Підключаємо гаманці до SOLdrip...".bright_blue());

    let wallets = storage::load_wallets()?;
    let total = wallets.len();

    if total == 0 {
        println!("{}", "⚠️  Немає гаманців для підключення. Спочатку згенеруйте їх!".yellow());
        return Ok(());
    }

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut connected = 0;
    let mut failed = 0;

    for wallet in wallets {
        match soldrip::connect_wallet(&wallet).await {
            Ok(_) => {
                connected += 1;
                // Оновлюємо статус, але не паднаємо якщо не вдалося
                if let Err(e) = storage::update_wallet_status(&wallet.address, "connected") {
                    log::error!("Failed to update status for {}: {}", wallet.address, e);
                    eprintln!("⚠️  Не вдалося оновити статус для {}", wallet.address);
                }
            }
            Err(e) => {
                failed += 1;
                log::error!("Failed to connect {}: {}", wallet.address, e);
                eprintln!("❌ Помилка підключення {}: {}", &wallet.address[..8], e);
            }
        }
        pb.inc(1);

        // Затримка між запитами, щоб не перевантажити сервер
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    pb.finish_with_message("✅ Готово!");

    println!("\n{}", "📊 Результати:".bright_cyan());
    println!("   ✅ Підключено: {}", connected.to_string().bright_green());
    println!("   ❌ Помилок: {}", failed.to_string().bright_red());

    Ok(())
}

async fn claim_all() -> Result<()> {
    println!("\n{}", "💰 Виконуємо claim для всіх гаманців...".bright_blue());

    // Завантажуємо конфігурацію 2Captcha
    let captcha_config = match captcha::CaptchaConfig::from_env() {
        Ok(config) => {
            // Перевіряємо баланс
            match captcha::get_balance(&config.api_key).await {
                Ok(balance) => {
                    println!("💳 Баланс 2Captcha: ${:.2}", balance);
                    if balance < 0.5 {
                        println!("{}", "⚠️  УВАГА: Низький баланс! Поповніть на https://2captcha.com".yellow());
                    }
                }
                Err(e) => {
                    println!("{}", format!("⚠️  Не вдалось перевірити баланс: {}", e).yellow());
                }
            }
            config
        }
        Err(e) => {
            println!("{}", format!("❌ Помилка конфігурації 2Captcha: {}", e).red());
            println!("{}", "💡 Створіть файл .env з CAPTCHA_API_KEY".yellow());
            return Ok(());
        }
    };

    let wallets = storage::load_wallets()?;
    let connected: Vec<_> = wallets
        .into_iter()
        .filter(|w| w.status == "connected")
        .collect();

    if connected.is_empty() {
        println!("{}", "⚠️  Немає підключених гаманців!".yellow());
        return Ok(());
    }

    println!("\n🤖 Використовую 2Captcha для автоматичного розв'язання капчі");
    println!("⏱️  Кожен claim займе ~20-30 секунд (час розв'язання капчі)");
    println!();

    let pb = ProgressBar::new(connected.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut total_claimed = 0.0;
    let mut successful = 0;
    let mut captcha_errors = 0;

    for wallet in connected {
        // Розв'язуємо капчу
        let captcha_token = match captcha::solve_captcha(&captcha_config).await {
            Ok(token) => token,
            Err(e) => {
                log::error!("Captcha failed for {}: {}", wallet.address, e);
                captcha_errors += 1;
                pb.inc(1);
                continue;
            }
        };

        // Виконуємо claim з капчею
        match soldrip::claim_with_captcha(&wallet, &captcha_token).await {
            Ok(amount) => {
                total_claimed += amount;
                successful += 1;
                storage::update_wallet_balance(&wallet.address, amount)?;
            }
            Err(e) => {
                log::error!("Claim failed for {}: {}", wallet.address, e);
            }
        }
        pb.inc(1);

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    pb.finish_with_message("✅ Готово!");

    println!("\n{}", "💎 Результати:".bright_cyan());
    println!("   ✅ Успішно: {}", successful.to_string().bright_green());
    println!("   💰 Всього зібрано: {} SOL", format!("{:.4}", total_claimed).bright_yellow());

    if captcha_errors > 0 {
        println!("   ⚠️  Помилок капчі: {}", captcha_errors.to_string().yellow());
    }

    Ok(())
}

async fn show_stats() -> Result<()> {
    let wallets = storage::load_wallets()?;

    let total = wallets.len();
    let connected = wallets.iter().filter(|w| w.status == "connected").count();
    let total_balance: f64 = wallets.iter().map(|w| w.balance).sum();

    println!("\n{}", "📊 Статистика:".bright_cyan().bold());
    println!("{}", "=" .repeat(50));
    println!("   📝 Всього гаманців: {}", total.to_string().bright_white());
    println!("   🔗 Підключено: {}", connected.to_string().bright_green());
    println!("   💰 Загальний баланс: {} SOL", format!("{:.4}", total_balance).bright_yellow());

    if connected > 0 {
        let monthly_estimate = connected as f64 * 0.15;
        println!("   📈 Очікуваний дохід/місяць: {} SOL", format!("{:.2}", monthly_estimate).bright_cyan());
    }

    println!("{}", "=" .repeat(50));

    Ok(())
}

async fn auto_claim_loop() -> Result<()> {
    println!("\n{}", "🔄 Автоматичний режим Auto-Claim".bright_blue().bold());
    println!("{}", "=" .repeat(50));
    println!();
    println!("Цей режим буде автоматично:");
    println!("  • Перевіряти статус накопичення кожні 5 хвилин");
    println!("  • Автоматично клеймити, коли досягне 100%");
    println!("  • Працювати безперервно, поки не зупините (Ctrl+C)");
    println!();

    // Завантажуємо конфігурацію 2Captcha
    let captcha_config = match captcha::CaptchaConfig::from_env() {
        Ok(config) => {
            match captcha::get_balance(&config.api_key).await {
                Ok(balance) => {
                    println!("💳 Баланс 2Captcha: ${:.2}", balance);
                    if balance < 0.5 {
                        println!("{}", "⚠️  УВАГА: Низький баланс! Поповніть на https://2captcha.com".yellow());
                    }
                }
                Err(e) => {
                    println!("{}", format!("⚠️  Не вдалось перевірити баланс: {}", e).yellow());
                }
            }
            config
        }
        Err(e) => {
            println!("{}", format!("❌ Помилка конфігурації 2Captcha: {}", e).red());
            println!("{}", "💡 Створіть файл .env з CAPTCHA_API_KEY".yellow());
            return Ok(());
        }
    };

    println!();
    print!("{}", "Натисніть Enter для запуску або Ctrl+C для відміни...".bright_yellow());
    io::stdout().flush()?;
    let mut _confirm = String::new();
    io::stdin().read_line(&mut _confirm)?;

    println!("\n{}", "✅ Автоматичний режим запущено!".bright_green().bold());
    println!("{}", "⏱️  Інтервал перевірки: 5 хвилин".bright_white());
    println!("{}", "🛑 Для зупинки натисніть Ctrl+C".bright_white());
    println!("{}", "=" .repeat(50));

    let check_interval = tokio::time::Duration::from_secs(5 * 60); // 5 хвилин
    let mut iteration = 0;

    loop {
        iteration += 1;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        println!("\n{}", format!("🔄 Ітерація #{} - {}", iteration, now).bright_cyan().bold());
        println!("{}", "-".repeat(50));

        // Завантажуємо підключені гаманці
        let wallets = storage::load_wallets()?;
        let connected: Vec<_> = wallets
            .into_iter()
            .filter(|w| w.status == "connected")
            .collect();

        if connected.is_empty() {
            println!("{}", "⚠️  Немає підключених гаманців!".yellow());
            println!("💡 Спочатку підключіть гаманці через опцію 2");
            break;
        }

        println!("📊 Перевіряємо {} гаманців...", connected.len());

        let mut ready_to_claim = Vec::new();

        // Перевіряємо accumulation для кожного гаманця
        for (idx, wallet) in connected.iter().enumerate() {
            match soldrip::check_accumulation(wallet).await {
                Ok(status) => {
                    let addr_short = &wallet.address[..8];
                    if status.is_full {
                        println!("  ✅ {} - {:.1}% - ГОТОВО ДО CLAIM!", addr_short, status.percentage);
                        ready_to_claim.push(wallet.clone());
                    } else {
                        println!("  ⏳ {} - {:.1}%", addr_short, status.percentage);
                    }
                }
                Err(e) => {
                    log::error!("Failed to check accumulation for {}: {}", wallet.address, e);
                    println!("  ❌ {} - помилка перевірки", &wallet.address[..8]);
                }
            }

            // Невелика затримка між перевірками
            if idx < connected.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        // Якщо є готові до claim - виконуємо
        if !ready_to_claim.is_empty() {
            println!("\n{}", format!("💰 Знайдено {} гаманців готових до claim!", ready_to_claim.len()).bright_green().bold());

            let mut successful = 0;
            let mut failed = 0;
            let mut total_claimed = 0.0;

            for wallet in ready_to_claim {
                println!("\n🤖 Обробляю {}...", &wallet.address[..12]);

                // Розв'язуємо капчу
                let captcha_token = match captcha::solve_captcha(&captcha_config).await {
                    Ok(token) => token,
                    Err(e) => {
                        log::error!("Captcha failed for {}: {}", wallet.address, e);
                        println!("  ❌ Не вдалося розв'язати капчу: {}", e);
                        failed += 1;
                        continue;
                    }
                };

                // Виконуємо claim
                match soldrip::claim_with_captcha(&wallet, &captcha_token).await {
                    Ok(amount) => {
                        total_claimed += amount;
                        successful += 1;
                        storage::update_wallet_balance(&wallet.address, amount)?;
                        println!("  ✅ Успішно claimed {} SOL", amount);
                    }
                    Err(e) => {
                        failed += 1;
                        log::error!("Claim failed for {}: {}", wallet.address, e);
                        println!("  ❌ Помилка: {}", e);
                    }
                }

                // Затримка між claims
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }

            println!("\n{}", "📊 Підсумок claim:".bright_cyan().bold());
            println!("   ✅ Успішно: {}", successful);
            println!("   ❌ Помилок: {}", failed);
            println!("   💰 Зібрано: {:.4} SOL", total_claimed);
        } else {
            println!("\n{}", "⏳ Жоден гаманець не готовий до claim".yellow());
        }

        // Чекаємо до наступної перевірки
        println!("\n{}", format!("😴 Чекаю {} хвилин до наступної перевірки...", check_interval.as_secs() / 60).bright_white());
        println!("{}", format!("   Наступна перевірка: {}",
            (chrono::Local::now() + chrono::Duration::seconds(check_interval.as_secs() as i64))
                .format("%H:%M:%S")).bright_white());

        tokio::time::sleep(check_interval).await;
    }

    Ok(())
}
