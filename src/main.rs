mod api;
mod database;
mod reports;
mod types;
mod utils;

use api::fetch_and_save_multiple_results;
use database::{
    create_database, get_latest_lottery_results, get_lottery_results_after_date,
    get_lottery_results_before_date, get_lottery_results_by_date_range,
    get_lottery_results_by_month, get_lottery_results_by_year,
};
use reports::generate_and_save_report;
use rusqlite::Connection;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = create_database()?;

    {
        let dir_path = "./json_data"; // Path to your directory

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                println!("Reading file: {:?}", path);

                let mut file = File::open(&path)?;
                let mut raw_json_string = String::new();
                file.read_to_string(&mut raw_json_string)?;

                // println!("Contents:\n{}\n", contents);

                // Uncomment if you want to parse into a struct
                // let data: YourStruct = serde_json::from_str(&contents)?;
                // RAW JSON
                //   let raw_json_string = r#"{}"#;
                let lottery_id = database::parse_and_insert_raw_json(&conn, &raw_json_string)?;
                println!("🎟️ Lottery ID {} inserted successfully.", lottery_id);
            }
        }

        // RAW JSON
        //  let raw_json_string = r#"{}"#;
        //  let lottery_id = database::parse_and_insert_raw_json(&conn, raw_json_string)?;
        //  println!("🎟️ Lottery ID {} inserted successfully.", lottery_id);
        return Ok(());
    }

    // let n_year = 2025;

    // // let dates_to_fetch = utils::generate_lottery_dates(n_year);

    // let n_year_as_string = n_year.to_string();
    // let dates_to_fetch = vec![
    //     ("02".to_string(), "01".to_string(), n_year_as_string.clone()),
    //     ("16".to_string(), "02".to_string(), n_year_as_string.clone()),
    // ];

    // println!("🎲 Starting lottery results batch fetch and save...\n");

    // match fetch_and_save_multiple_results(&conn, &dates_to_fetch).await {
    //     Ok(_) => {
    //         println!("\n✅ All operations completed successfully!");
    //     }
    //     Err(e) => {
    //         eprintln!("❌ Error during batch operation: {}", e);
    //     }
    // }

    println!("\n📋 Generating HTML reports for available dates...");
    match get_lottery_results_after_date(&conn, "2016-12-30", Some(1)) {
        Ok(latest_results) => {
            if latest_results.is_empty() {
                println!("⚠ No lottery data found in database for report generation.");
            } else {
                for (_, result) in latest_results.iter().enumerate() {
                    match generate_and_save_report(&conn, &result.draw_date) {
                        Ok(()) => println!("✅ Report generated for {}", result.draw_date),
                        Err(e) => println!(
                            "❌ Failed to generate report for {}: {}",
                            result.draw_date, e
                        ),
                    }
                }
            }
        }
        Err(e) => println!("❌ Error getting latest results: {}", e),
    }

    Ok(())
}
