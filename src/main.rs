use reqwest;
use rusqlite::OptionalExtension;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct LotteryRequest {
    date: String,
    month: String,
    year: String,
}

#[derive(Deserialize, Debug)]
struct LotteryResponse {
    #[serde(rename = "statusMessage")]
    status_message: String,
    #[serde(rename = "statusCode")]
    status_code: i32,
    status: bool,
    response: Option<ResponseData>,
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    result: Option<LotteryResult>,
}

#[derive(Deserialize, Debug)]
struct LotteryResult {
    date: String,
    period: Vec<i32>,
    data: LotteryData,
}

#[derive(Deserialize, Debug)]
struct LotteryData {
    first: PrizeCategory,
    second: PrizeCategory,
    third: PrizeCategory,
    fourth: PrizeCategory,
    fifth: PrizeCategory,
    last2: PrizeCategory,
    last3f: PrizeCategory,
    last3b: PrizeCategory,
    near1: PrizeCategory,
}

#[derive(Deserialize, Debug)]
struct PrizeCategory {
    price: String,
    number: Vec<PrizeNumber>,
}

#[derive(Deserialize, Debug)]
struct PrizeNumber {
    round: i32,
    value: String,
}

fn create_database() -> Result<Connection> {
    let conn = Connection::open("lottery.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS lottery_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            draw_date TEXT NOT NULL UNIQUE,
            period TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prize_numbers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            lottery_id INTEGER NOT NULL,
            category TEXT NOT NULL,
            prize_amount TEXT NOT NULL,
            number_value TEXT NOT NULL,
            round_number INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (lottery_id) REFERENCES lottery_results (id)
        )",
        [],
    )?;

    Ok(conn)
}

fn save_lottery_result(conn: &Connection, result: &LotteryResult) -> Result<()> {
    let period_str = result
        .period
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    conn.execute(
        "INSERT OR IGNORE INTO lottery_results (draw_date, period) VALUES (?1, ?2)",
        (&result.date, &period_str),
    )?;

    let lottery_id: i64 = conn.last_insert_rowid();
    if lottery_id == 0 {
        let mut stmt = conn.prepare("SELECT id FROM lottery_results WHERE draw_date = ?1")?;
        let row = stmt.query_row([&result.date], |row| row.get::<_, i64>(0))?;
        return save_prize_numbers(conn, row, &result.data);
    }

    save_prize_numbers(conn, lottery_id, &result.data)?;
    Ok(())
}

fn save_prize_numbers(conn: &Connection, lottery_id: i64, data: &LotteryData) -> Result<()> {
    let categories = [
        ("first", &data.first),
        ("second", &data.second),
        ("third", &data.third),
        ("fourth", &data.fourth),
        ("fifth", &data.fifth),
        ("last2", &data.last2),
        ("last3f", &data.last3f),
        ("last3b", &data.last3b),
        ("near1", &data.near1),
    ];

    for (category_name, category) in categories {
        for prize_number in &category.number {
            conn.execute(
                "INSERT OR IGNORE INTO prize_numbers (
                    lottery_id, category, prize_amount, number_value, round_number
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    lottery_id,
                    category_name,
                    &category.price,
                    &prize_number.value,
                    prize_number.round,
                ),
            )?;
        }
    }
    Ok(())
}

fn save_multiple_lottery_results(conn: &Connection, results: &[LotteryResult]) -> Result<()> {
    for result in results {
        save_lottery_result(conn, result)?;
    }
    Ok(())
}

#[derive(Debug)]
struct LotteryResultRow {
    id: i64,
    draw_date: String,
    period: String,
    created_at: String,
}

#[derive(Debug)]
struct PrizeNumberRow {
    id: i64,
    lottery_id: i64,
    category: String,
    prize_amount: String,
    number_value: String,
    round_number: i32,
}

fn get_all_lottery_results(conn: &Connection) -> Result<Vec<LotteryResultRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, draw_date, period, created_at FROM lottery_results ORDER BY draw_date DESC",
    )?;
    let lottery_iter = stmt.query_map([], |row| {
        Ok(LotteryResultRow {
            id: row.get(0)?,
            draw_date: row.get(1)?,
            period: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut results = Vec::new();
    for lottery in lottery_iter {
        results.push(lottery?);
    }
    Ok(results)
}

fn get_lottery_by_date(conn: &Connection, date: &str) -> Result<Option<LotteryResultRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, draw_date, period, created_at FROM lottery_results WHERE draw_date = ?1",
    )?;
    let result = stmt
        .query_row([date], |row| {
            Ok(LotteryResultRow {
                id: row.get(0)?,
                draw_date: row.get(1)?,
                period: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .optional()?;
    Ok(result)
}

fn get_prize_numbers_by_lottery_id(
    conn: &Connection,
    lottery_id: i64,
) -> Result<Vec<PrizeNumberRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, lottery_id, category, prize_amount, number_value, round_number 
         FROM prize_numbers WHERE lottery_id = ?1 ORDER BY category, round_number",
    )?;
    let prize_iter = stmt.query_map([lottery_id], |row| {
        Ok(PrizeNumberRow {
            id: row.get(0)?,
            lottery_id: row.get(1)?,
            category: row.get(2)?,
            prize_amount: row.get(3)?,
            number_value: row.get(4)?,
            round_number: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for prize in prize_iter {
        results.push(prize?);
    }
    Ok(results)
}

fn get_prize_numbers_by_category(conn: &Connection, category: &str) -> Result<Vec<PrizeNumberRow>> {
    let mut stmt = conn.prepare(
        "SELECT pn.id, pn.lottery_id, pn.category, pn.prize_amount, pn.number_value, pn.round_number 
         FROM prize_numbers pn 
         JOIN lottery_results lr ON pn.lottery_id = lr.id 
         WHERE pn.category = ?1 
         ORDER BY lr.draw_date DESC, pn.round_number"
    )?;
    let prize_iter = stmt.query_map([category], |row| {
        Ok(PrizeNumberRow {
            id: row.get(0)?,
            lottery_id: row.get(1)?,
            category: row.get(2)?,
            prize_amount: row.get(3)?,
            number_value: row.get(4)?,
            round_number: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for prize in prize_iter {
        results.push(prize?);
    }
    Ok(results)
}

fn search_number(
    conn: &Connection,
    number: &str,
) -> Result<Vec<(LotteryResultRow, PrizeNumberRow)>> {
    let mut stmt = conn.prepare(
        "SELECT lr.id, lr.draw_date, lr.period, lr.created_at,
                pn.id, pn.lottery_id, pn.category, pn.prize_amount, pn.number_value, pn.round_number
         FROM lottery_results lr
         JOIN prize_numbers pn ON lr.id = pn.lottery_id
         WHERE pn.number_value LIKE ?1
         ORDER BY lr.draw_date DESC",
    )?;

    let search_pattern = format!("%{}%", number);
    let result_iter = stmt.query_map([&search_pattern], |row| {
        Ok((
            LotteryResultRow {
                id: row.get(0)?,
                draw_date: row.get(1)?,
                period: row.get(2)?,
                created_at: row.get(3)?,
            },
            PrizeNumberRow {
                id: row.get(4)?,
                lottery_id: row.get(5)?,
                category: row.get(6)?,
                prize_amount: row.get(7)?,
                number_value: row.get(8)?,
                round_number: row.get(9)?,
            },
        ))
    })?;

    let mut results = Vec::new();
    for result in result_iter {
        results.push(result?);
    }
    Ok(results)
}

fn get_latest_lottery_results(conn: &Connection, limit: i32) -> Result<Vec<LotteryResultRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, draw_date, period, created_at 
         FROM lottery_results 
         ORDER BY draw_date DESC 
         LIMIT ?1",
    )?;
    let lottery_iter = stmt.query_map([limit], |row| {
        Ok(LotteryResultRow {
            id: row.get(0)?,
            draw_date: row.get(1)?,
            period: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut results = Vec::new();
    for lottery in lottery_iter {
        results.push(lottery?);
    }
    Ok(results)
}

fn get_complete_lottery_data(
    conn: &Connection,
    date: &str,
) -> Result<Option<(LotteryResultRow, Vec<PrizeNumberRow>)>> {
    if let Some(lottery) = get_lottery_by_date(conn, date)? {
        let prizes = get_prize_numbers_by_lottery_id(conn, lottery.id)?;
        Ok(Some((lottery, prizes)))
    } else {
        Ok(None)
    }
}

fn lottery_exists_for_date(conn: &Connection, date: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM lottery_results WHERE draw_date = ?1")?;
    let count: i64 = stmt.query_row([date], |row| row.get(0))?;
    Ok(count > 0)
}

fn format_date_for_api(date: &str, month: &str, year: &str) -> String {
    format!("{}-{:0>2}-{:0>2}", year, month, date)
}

fn check_existing_dates(
    conn: &Connection,
    dates: &[(String, String, String)],
) -> Result<(Vec<(String, String, String)>, Vec<String>)> {
    let mut dates_to_fetch = Vec::new();
    let mut existing_dates = Vec::new();

    for (date, month, year) in dates {
        let formatted_date = format_date_for_api(date, month, year);
        if lottery_exists_for_date(conn, &formatted_date)? {
            existing_dates.push(formatted_date);
        } else {
            dates_to_fetch.push((date.clone(), month.clone(), year.clone()));
        }
    }

    Ok((dates_to_fetch, existing_dates))
}

async fn fetch_lottery_result(
    date: &str,
    month: &str,
    year: &str,
) -> Result<LotteryResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let request_body = LotteryRequest {
        date: date.to_string(),
        month: month.to_string(),
        year: year.to_string(),
    };

    let response = client
        .post("https://www.glo.or.th/api/checking/getLotteryResult")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let lottery_response: LotteryResponse = response.json().await?;
    Ok(lottery_response)
}

async fn fetch_and_save_multiple_results(
    conn: &Connection,
    dates: &[(String, String, String)],
) -> Result<Vec<LotteryResult>, Box<dyn Error>> {
    println!("🔍 Checking existing data in database...");
    let (dates_to_fetch, existing_dates) = check_existing_dates(conn, dates)?;

    if !existing_dates.is_empty() {
        println!(
            "📋 Found {} existing dates in database:",
            existing_dates.len()
        );
        for date in &existing_dates {
            println!("   ✓ {} (already exists)", date);
        }
    }

    if dates_to_fetch.is_empty() {
        println!("🎯 All requested dates already exist in database. No fetching needed!");
        return Ok(Vec::new());
    }

    println!("📥 Need to fetch {} new dates:", dates_to_fetch.len());
    for (date, month, year) in &dates_to_fetch {
        println!("   → {}/{}/{}", date, month, year);
    }
    println!();

    let mut all_results = Vec::new();

    for (date, month, year) in dates_to_fetch {
        println!("Fetching lottery results for {}/{}/{}", date, month, year);

        match fetch_lottery_result(&date, &month, &year).await {
            Ok(response) => {
                if response.status && response.status_code == 200 {
                    if let Some(response_data) = response.response {
                        if let Some(result) = response_data.result {
                            println!(
                                "✓ Results fetched for {}/{}/{} - Date: {}",
                                date, month, year, result.date
                            );
                            all_results.push(result);
                        } else {
                            println!("⚠ No lottery result found for {}/{}/{}", date, month, year);
                        }
                    } else {
                        println!("⚠ No response data found for {}/{}/{}", date, month, year);
                    }
                } else {
                    println!(
                        "✗ API error for {}/{}/{}: {} ({})",
                        date, month, year, response.status_message, response.status_code
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "✗ Error fetching results for {}/{}/{}: {}",
                    date, month, year, e
                );
            }
        }
    }

    if !all_results.is_empty() {
        save_multiple_lottery_results(conn, &all_results)?;
        println!(
            "\n🎯 Successfully saved {} new lottery results to database!",
            all_results.len()
        );
    } else {
        println!("\n⚠ No new results were fetched and saved.");
    }

    Ok(all_results)
}

fn demonstrate_query_functions(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Demonstrating database query functions...\n");

    println!("1️⃣  Getting all lottery results:");
    match get_all_lottery_results(conn) {
        Ok(results) => {
            println!("   Found {} lottery results in database", results.len());
            for (i, result) in results.iter().take(3).enumerate() {
                println!(
                    "   {}. Date: {} | Period: {}",
                    i + 1,
                    result.draw_date,
                    result.period
                );
            }
            if results.len() > 3 {
                println!("   ... and {} more", results.len() - 3);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n2️⃣  Getting latest 2 lottery results:");
    match get_latest_lottery_results(conn, 2) {
        Ok(results) => {
            for result in results {
                println!("   • Date: {} | ID: {}", result.draw_date, result.id);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n3️⃣  Getting first prize numbers:");
    match get_prize_numbers_by_category(conn, "first") {
        Ok(prizes) => {
            println!("   Found {} first prize numbers", prizes.len());
            for prize in prizes.iter().take(3) {
                println!(
                    "   • Number: {} | Amount: {}",
                    prize.number_value, prize.prize_amount
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n4️⃣  Searching for numbers containing '25':");
    match search_number(conn, "25") {
        Ok(results) => {
            println!("   Found {} matches", results.len());
            for (lottery, prize) in results.iter().take(3) {
                println!(
                    "   • Date: {} | Category: {} | Number: {}",
                    lottery.draw_date, prize.category, prize.number_value
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n5️⃣  Getting complete lottery data for specific date:");
    if let Ok(results) = get_latest_lottery_results(conn, 1) {
        if let Some(latest) = results.first() {
            match get_complete_lottery_data(conn, &latest.draw_date) {
                Ok(Some((lottery, prizes))) => {
                    println!(
                        "   Lottery Date: {} | Total prizes: {}",
                        lottery.draw_date,
                        prizes.len()
                    );

                    let mut category_counts = std::collections::HashMap::new();
                    for prize in &prizes {
                        *category_counts.entry(&prize.category).or_insert(0) += 1;
                    }

                    for (category, count) in category_counts {
                        println!("   • {}: {} numbers", category, count);
                    }
                }
                Ok(None) => println!("   No data found for date: {}", latest.draw_date),
                Err(e) => println!("   Error: {}", e),
            }
        }
    }

    println!("\n✅ Query demonstration completed!\n");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = create_database()?;

    let dates_to_fetch = vec![
        ("01".to_string(), "03".to_string(), "2024".to_string()),
        ("16".to_string(), "03".to_string(), "2024".to_string()),
        ("01".to_string(), "04".to_string(), "2024".to_string()),
        ("16".to_string(), "04".to_string(), "2024".to_string()),
        ("01".to_string(), "05".to_string(), "2024".to_string()),
    ];

    println!("🎲 Starting lottery results batch fetch and save...\n");

    match fetch_and_save_multiple_results(&conn, &dates_to_fetch).await {
        Ok(results) => {
            println!("\n📊 Summary:");
            for result in &results {
                let first_prize_num = if let Some(first_num) = result.data.first.number.first() {
                    &first_num.value
                } else {
                    "N/A"
                };
                println!(
                    "  • Date: {} | Period: {:?} | First Prize: {} ({})",
                    result.date, result.period, first_prize_num, result.data.first.price
                );
            }
            println!("\n✅ All operations completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Error during batch operation: {}", e);
        }
    }

    demonstrate_query_functions(&conn)?;

    Ok(())
}
