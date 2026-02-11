use anyhow::{Context, Result};
use calamine::{Reader, open_workbook, Xlsx, DataType};
use rust_xlsxwriter::*;
use std::path::Path;

use crate::wallet::Wallet;

const WALLETS_FILE: &str = "wallets.xlsx";

/// Зберігає гаманці у XLSX файл
pub fn save_wallets(wallets: &[Wallet]) -> Result<()> {
    // Завантажуємо існуючі гаманці якщо файл існує
    let mut all_wallets = if Path::new(WALLETS_FILE).exists() {
        load_wallets()?
    } else {
        Vec::new()
    };

    // Додаємо нові гаманці
    all_wallets.extend_from_slice(wallets);

    // Створюємо новий workbook
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Налаштування для заголовків (жирний шрифт)
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xD3D3D3));

    // Записуємо заголовки
    worksheet.write_string_with_format(0, 0, "Address", &header_format)?;
    worksheet.write_string_with_format(0, 1, "Private Key", &header_format)?;
    worksheet.write_string_with_format(0, 2, "Status", &header_format)?;
    worksheet.write_string_with_format(0, 3, "Balance (SOL)", &header_format)?;
    worksheet.write_string_with_format(0, 4, "Created At", &header_format)?;
    worksheet.write_string_with_format(0, 5, "Last Claim", &header_format)?;

    // Налаштування ширини колонок
    worksheet.set_column_width(0, 45)?; // Address
    worksheet.set_column_width(1, 90)?; // Private Key
    worksheet.set_column_width(2, 12)?; // Status
    worksheet.set_column_width(3, 15)?; // Balance
    worksheet.set_column_width(4, 25)?; // Created At
    worksheet.set_column_width(5, 25)?; // Last Claim

    // Записуємо дані
    for (i, wallet) in all_wallets.iter().enumerate() {
        let row = (i + 1) as u32;

        worksheet.write_string(row, 0, &wallet.address)?;
        worksheet.write_string(row, 1, &wallet.private_key)?;
        worksheet.write_string(row, 2, &wallet.status)?;
        worksheet.write_number(row, 3, wallet.balance)?;
        worksheet.write_string(row, 4, &wallet.created_at)?;
        worksheet.write_string(row, 5, wallet.last_claim.as_deref().unwrap_or(""))?;
    }

    // Зберігаємо файл
    workbook.save(WALLETS_FILE)?;
    Ok(())
}

/// Завантажує всі гаманці з XLSX файлу
pub fn load_wallets() -> Result<Vec<Wallet>> {
    if !Path::new(WALLETS_FILE).exists() {
        return Ok(Vec::new());
    }

    let mut workbook: Xlsx<_> = open_workbook(WALLETS_FILE)
        .context("Не вдалося відкрити файл гаманців")?;

    let mut wallets = Vec::new();

    // Читаємо перший лист
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        // Пропускаємо заголовок (перший рядок)
        for row in range.rows().skip(1) {
            // Перевіряємо що рядок не пустий
            if row.is_empty() {
                continue;
            }

            let address_str = row[0].to_string();
            if address_str.is_empty() {
                continue;
            }

            let wallet = Wallet {
                address: address_str,
                private_key: row[1].to_string(),
                status: row[2].to_string(),
                balance: row[3].get_float().unwrap_or_else(|| {
                    row[3].to_string().parse().unwrap_or(0.0)
                }),
                created_at: row[4].to_string(),
                last_claim: if row.len() > 5 {
                    let claim_str = row[5].to_string();
                    if claim_str.is_empty() {
                        None
                    } else {
                        Some(claim_str)
                    }
                } else {
                    None
                },
            };

            wallets.push(wallet);
        }
    }

    Ok(wallets)
}

/// Оновлює статус гаманця
pub fn update_wallet_status(address: &str, status: &str) -> Result<()> {
    let mut wallets = load_wallets()?;

    for wallet in &mut wallets {
        if wallet.address == address {
            wallet.status = status.to_string();
            break;
        }
    }

    // Перезаписуємо файл
    save_all_wallets(&wallets)?;
    Ok(())
}

/// Оновлює баланс гаманця
pub fn update_wallet_balance(address: &str, amount: f64) -> Result<()> {
    let mut wallets = load_wallets()?;

    for wallet in &mut wallets {
        if wallet.address == address {
            wallet.balance += amount;
            wallet.last_claim = Some(chrono::Utc::now().to_rfc3339());
            break;
        }
    }

    // Перезаписуємо файл
    save_all_wallets(&wallets)?;
    Ok(())
}

/// Зберігає ВСІ гаманці (повністю перезаписує файл)
fn save_all_wallets(wallets: &[Wallet]) -> Result<()> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xD3D3D3));

    worksheet.write_string_with_format(0, 0, "Address", &header_format)?;
    worksheet.write_string_with_format(0, 1, "Private Key", &header_format)?;
    worksheet.write_string_with_format(0, 2, "Status", &header_format)?;
    worksheet.write_string_with_format(0, 3, "Balance (SOL)", &header_format)?;
    worksheet.write_string_with_format(0, 4, "Created At", &header_format)?;
    worksheet.write_string_with_format(0, 5, "Last Claim", &header_format)?;

    worksheet.set_column_width(0, 45)?;
    worksheet.set_column_width(1, 90)?;
    worksheet.set_column_width(2, 12)?;
    worksheet.set_column_width(3, 15)?;
    worksheet.set_column_width(4, 25)?;
    worksheet.set_column_width(5, 25)?;

    for (i, wallet) in wallets.iter().enumerate() {
        let row = (i + 1) as u32;

        worksheet.write_string(row, 0, &wallet.address)?;
        worksheet.write_string(row, 1, &wallet.private_key)?;
        worksheet.write_string(row, 2, &wallet.status)?;
        worksheet.write_number(row, 3, wallet.balance)?;
        worksheet.write_string(row, 4, &wallet.created_at)?;
        worksheet.write_string(row, 5, wallet.last_claim.as_deref().unwrap_or(""))?;
    }

    workbook.save(WALLETS_FILE)?;
    Ok(())
}

/// Отримує статистику по гаманцях
pub fn get_stats() -> Result<(usize, usize, f64)> {
    let wallets = load_wallets()?;

    let total = wallets.len();
    let connected = wallets.iter().filter(|w| w.status == "connected").count();
    let total_balance: f64 = wallets.iter().map(|w| w.balance).sum();

    Ok((total, connected, total_balance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load() {
        use crate::wallet::generate_wallet;

        let wallet = generate_wallet().unwrap();
        save_wallets(&[wallet.clone()]).unwrap();

        let loaded = load_wallets().unwrap();
        assert!(!loaded.is_empty());
    }
}
