use crate::database::{check_existing_dates, save_multiple_lottery_results};
use crate::types::{LotteryRequest, LotteryResponse, LotteryResult};
use reqwest;
use rusqlite::Connection;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

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
    let (dates_to_fetch, existing_dates) = check_existing_dates(conn, dates)?;

    if !existing_dates.is_empty() {}

    if dates_to_fetch.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_results = Vec::new();

    for (date, month, year) in dates_to_fetch {
        match fetch_lottery_result(&date, &month, &year).await {
            Ok(response) => {
                if response.status && response.status_code == 200 {
                    if let Some(response_data) = response.response {
                        if let Some(result) = response_data.result {
                            all_results.push(result);
                        }
                    }
                }
            }
            Err(e) => {}
        }
        sleep(Duration::from_secs(1));
    }

    if !all_results.is_empty() {
        save_multiple_lottery_results(conn, &all_results)?;
    }
    Ok(all_results)
}
