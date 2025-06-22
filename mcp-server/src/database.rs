use crate::types::{LotteryData, LotteryResult, LotteryResultRow, PrizeNumberRow};
use rusqlite::{Connection, OptionalExtension, Result};
use serde_json::Value;
use std::error::Error;
use std::fs;

pub fn ensure_directories() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all("data")?;
    fs::create_dir_all("reports")?;
    println!("📁 Ensured directories: data/, reports/");
    Ok(())
}

pub fn create_database() -> Result<Connection> {
    ensure_directories().map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(format!("Failed to create directories: {}", e)),
        )
    })?;

    let conn = Connection::open("data/lottery.db")?;

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

pub fn save_lottery_result(conn: &Connection, result: &LotteryResult) -> Result<()> {
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

pub fn save_multiple_lottery_results(conn: &Connection, results: &[LotteryResult]) -> Result<()> {
    for result in results {
        save_lottery_result(conn, result)?;
    }
    Ok(())
}

pub fn get_all_lottery_results(conn: &Connection) -> Result<Vec<LotteryResultRow>> {
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

pub fn get_lottery_by_date(conn: &Connection, date: &str) -> Result<Option<LotteryResultRow>> {
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

pub fn get_prize_numbers_by_lottery_id(
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

pub fn get_prize_numbers_by_category(
    conn: &Connection,
    category: &str,
) -> Result<Vec<PrizeNumberRow>> {
    let mut stmt = conn.prepare(
        "SELECT pn.id, pn.lottery_id, pn.category, pn.prize_amount, pn.number_value, pn.round_number 
         FROM prize_numbers pn 
         JOIN lottery_results lr ON pn.lottery_id = lr.id 
         WHERE pn.category = ?1 
         ORDER BY lr.draw_date DESC, pn.round_number",
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

pub fn search_number(
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

pub fn get_latest_lottery_results(conn: &Connection, limit: i32) -> Result<Vec<LotteryResultRow>> {
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

pub fn get_lottery_results_by_date_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<LotteryResultRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, draw_date, period, created_at 
         FROM lottery_results 
         WHERE draw_date >= ?1 AND draw_date <= ?2
         ORDER BY draw_date DESC",
    )?;
    let lottery_iter = stmt.query_map([start_date, end_date], |row| {
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

pub fn get_lottery_results_by_year(conn: &Connection, year: &str) -> Result<Vec<LotteryResultRow>> {
    let start_date = format!("{}-01-01", year);
    let end_date = format!("{}-12-31", year);
    get_lottery_results_by_date_range(conn, &start_date, &end_date)
}

pub fn get_lottery_results_by_month(
    conn: &Connection,
    year: &str,
    month: &str,
) -> Result<Vec<LotteryResultRow>> {
    let start_date = format!("{}-{:0>2}-01", year, month);
    let end_date = format!("{}-{:0>2}-31", year, month);
    get_lottery_results_by_date_range(conn, &start_date, &end_date)
}

pub fn get_lottery_results_after_date(
    conn: &Connection,
    date: &str,
    limit: Option<i32>,
) -> Result<Vec<LotteryResultRow>> {
    let query = if let Some(limit_val) = limit {
        format!(
            "SELECT id, draw_date, period, created_at 
             FROM lottery_results 
             WHERE draw_date >= ?1
             LIMIT {}",
            limit_val
        )
    } else {
        "SELECT id, draw_date, period, created_at 
         FROM lottery_results 
         WHERE draw_date >= ?1
         ORDER BY draw_date DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let lottery_iter = stmt.query_map([date], |row| {
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

pub fn get_lottery_results_before_date(
    conn: &Connection,
    date: &str,
    limit: Option<i32>,
) -> Result<Vec<LotteryResultRow>> {
    let query = if let Some(limit_val) = limit {
        format!(
            "SELECT id, draw_date, period, created_at 
             FROM lottery_results 
             WHERE draw_date <= ?1
             ORDER BY draw_date DESC
             LIMIT {}",
            limit_val
        )
    } else {
        "SELECT id, draw_date, period, created_at 
         FROM lottery_results 
         WHERE draw_date <= ?1
         ORDER BY draw_date DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let lottery_iter = stmt.query_map([date], |row| {
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

pub fn get_complete_lottery_data(
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

pub fn lottery_exists_for_date(conn: &Connection, date: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM lottery_results WHERE draw_date = ?1")?;
    let count: i64 = stmt.query_row([date], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn check_existing_dates(
    conn: &Connection,
    dates: &[(String, String, String)],
) -> Result<(Vec<(String, String, String)>, Vec<String>)> {
    let mut dates_to_fetch = Vec::new();
    let mut existing_dates = Vec::new();

    for (date, month, year) in dates {
        let formatted_date = crate::utils::format_date_for_api(date, month, year);
        if lottery_exists_for_date(conn, &formatted_date)? {
            existing_dates.push(formatted_date);
        } else {
            dates_to_fetch.push((date.clone(), month.clone(), year.clone()));
        }
    }

    Ok((dates_to_fetch, existing_dates))
}

pub fn parse_and_insert_raw_json(conn: &Connection, raw_json: &str) -> Result<i64> {
    let json_value: Value = serde_json::from_str(raw_json).map_err(|e| {
        rusqlite::Error::InvalidColumnType(
            0,
            format!("Invalid JSON: {}", e),
            rusqlite::types::Type::Text,
        )
    })?;

    if !json_value
        .get("status")
        .and_then(|s| s.as_bool())
        .unwrap_or(false)
    {
        return Err(rusqlite::Error::InvalidColumnType(
            0,
            "JSON status is false".to_string(),
            rusqlite::types::Type::Text,
        ));
    }

    let result = json_value
        .get("response")
        .and_then(|r| r.get("result"))
        .ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                0,
                "Missing result in JSON".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    let draw_date = result.get("date").and_then(|d| d.as_str()).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(
            0,
            "Missing date in JSON".to_string(),
            rusqlite::types::Type::Text,
        )
    })?;

    let period_array = result
        .get("period")
        .and_then(|p| p.as_array())
        .ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                0,
                "Missing or invalid period in JSON".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    let period_str = period_array
        .iter()
        .filter_map(|p| p.as_i64())
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    conn.execute(
        "INSERT OR IGNORE INTO lottery_results (draw_date, period) VALUES (?1, ?2)",
        (draw_date, &period_str),
    )?;

    let lottery_id: i64 = if conn.changes() > 0 {
        conn.last_insert_rowid()
    } else {
        let mut stmt = conn.prepare("SELECT id FROM lottery_results WHERE draw_date = ?1")?;
        stmt.query_row([draw_date], |row| row.get::<_, i64>(0))?
    };

    let data = result.get("data").ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(
            0,
            "Missing data in JSON".to_string(),
            rusqlite::types::Type::Text,
        )
    })?;

    insert_prize_categories_from_json(conn, lottery_id, data)?;

    Ok(lottery_id)
}

fn insert_prize_categories_from_json(
    conn: &Connection,
    lottery_id: i64,
    data: &Value,
) -> Result<()> {
    let categories = [
        "first", "second", "third", "fourth", "fifth", "last2", "last3f", "last3b", "near1",
    ];

    for category_name in categories {
        if let Some(category) = data.get(category_name) {
            let price = category
                .get("price")
                .and_then(|p| p.as_str())
                .unwrap_or("0.00");

            if let Some(numbers_array) = category.get("number").and_then(|n| n.as_array()) {
                for number_obj in numbers_array {
                    let round_number = number_obj
                        .get("round")
                        .and_then(|r| r.as_i64())
                        .unwrap_or(0) as i32;

                    let number_value = number_obj
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    conn.execute(
                        "INSERT OR IGNORE INTO prize_numbers (
                            lottery_id, category, prize_amount, number_value, round_number
                        ) VALUES (?1, ?2, ?3, ?4, ?5)",
                        (lottery_id, category_name, price, number_value, round_number),
                    )?;
                }
            }
        }
    }
    Ok(())
}
