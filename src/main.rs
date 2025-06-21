mod api;
mod database;
mod demo;
mod reports;
mod types;
mod utils;

use std::error::Error;
use api::fetch_and_save_multiple_results;
use database::{create_database, get_latest_lottery_results};
use demo::demonstrate_query_functions;
use reports::generate_and_save_report;
use utils::{show_project_structure, list_generated_files};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    show_project_structure();
    let conn = create_database()?;

    let dates_to_fetch = vec![
        ("16".to_string(), "06".to_string(), "2025".to_string()),
        ("16".to_string(), "03".to_string(), "2024".to_string()),
        //("01".to_string(), "04".to_string(), "2024".to_string()),
        //("16".to_string(), "04".to_string(), "2024".to_string()),
        //("01".to_string(), "05".to_string(), "2024".to_string()),
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

    println!("\n📋 Generating HTML reports for available dates...");
    match get_latest_lottery_results(&conn, 3) {
        Ok(latest_results) => {
            if latest_results.is_empty() {
                println!("⚠ No lottery data found in database for report generation.");
            } else {
                for (i, result) in latest_results.iter().enumerate() {
                    if i < 2 {
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
        }
        Err(e) => println!("❌ Error getting latest results: {}", e),
    }

    list_generated_files()?;

    Ok(())
}