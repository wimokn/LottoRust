use reqwest;
use std::error::Error;
use crate::types::{LotteryRequest, LotteryResponse, LotteryResult};
use crate::database::{save_multiple_lottery_results, check_existing_dates};
use rusqlite::Connection;

pub async fn fetch_lottery_result(
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

pub async fn fetch_and_save_multiple_results(
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