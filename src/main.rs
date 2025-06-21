use reqwest;
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
    status: String,
    data: Option<LotteryData>,
}

#[derive(Deserialize, Debug)]
struct LotteryData {
    #[serde(rename = "drawDate")]
    draw_date: String,
    #[serde(rename = "drawNo")]
    draw_no: String,
    #[serde(rename = "first")]
    first_prize: Option<String>,
    #[serde(rename = "last2")]
    last_two_digits: Option<String>,
    #[serde(rename = "last3")]
    last_three_digits: Option<Vec<String>>,
    #[serde(rename = "near1")]
    near_first: Option<Vec<String>>,
    #[serde(rename = "second")]
    second_prize: Option<Vec<String>>,
    #[serde(rename = "third")]
    third_prize: Option<Vec<String>>,
    #[serde(rename = "fourth")]
    fourth_prize: Option<Vec<String>>,
    #[serde(rename = "fifth")]
    fifth_prize: Option<Vec<String>>,
}

fn create_database() -> Result<Connection> {
    let conn = Connection::open("lottery.db")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS lottery_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            draw_date TEXT NOT NULL,
            draw_no TEXT NOT NULL,
            first_prize TEXT,
            last_two_digits TEXT,
            last_three_digits TEXT,
            near_first TEXT,
            second_prize TEXT,
            third_prize TEXT,
            fourth_prize TEXT,
            fifth_prize TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    Ok(conn)
}

fn save_lottery_result(conn: &Connection, data: &LotteryData) -> Result<()> {
    conn.execute(
        "INSERT INTO lottery_results (
            draw_date, draw_no, first_prize, last_two_digits, last_three_digits,
            near_first, second_prize, third_prize, fourth_prize, fifth_prize
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &data.draw_date,
            &data.draw_no,
            &data.first_prize,
            &data.last_two_digits,
            &data.last_three_digits.as_ref().map(|v| v.join(",")),
            &data.near_first.as_ref().map(|v| v.join(",")),
            &data.second_prize.as_ref().map(|v| v.join(",")),
            &data.third_prize.as_ref().map(|v| v.join(",")),
            &data.fourth_prize.as_ref().map(|v| v.join(",")),
            &data.fifth_prize.as_ref().map(|v| v.join(",")),
        ),
    )?;
    Ok(())
}

async fn fetch_lottery_result(date: &str, month: &str, year: &str) -> Result<LotteryResponse, Box<dyn Error>> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = create_database()?;
    
    let date = "01";
    let month = "03";
    let year = "2024";
    
    println!("Fetching lottery results for {}/{}/{}", date, month, year);
    
    match fetch_lottery_result(date, month, year).await {
        Ok(response) => {
            if response.status == "success" {
                if let Some(data) = response.data {
                    println!("Lottery results fetched successfully!");
                    println!("Draw Date: {}", data.draw_date);
                    println!("Draw No: {}", data.draw_no);
                    if let Some(first) = &data.first_prize {
                        println!("First Prize: {}", first);
                    }
                    if let Some(last2) = &data.last_two_digits {
                        println!("Last Two Digits: {}", last2);
                    }
                    
                    save_lottery_result(&conn, &data)?;
                    println!("Results saved to database successfully!");
                } else {
                    println!("No lottery data found for the specified date.");
                }
            } else {
                println!("API returned error status: {}", response.status);
            }
        }
        Err(e) => {
            eprintln!("Error fetching lottery results: {}", e);
        }
    }
    
    Ok(())
}