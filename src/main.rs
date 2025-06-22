mod api;
mod database;
mod demo;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // show_project_structure();
    let conn = create_database()?;

    let n_year = 2010;

    let dates_to_fetch = utils::generate_lottery_dates(n_year);

    // let n_year_as_string = n_year.to_string();
    // let dates_to_fetch = vec![
    //     ("16".to_string(), "01".to_string(), n_year_as_string.clone()),
    //     ("01".to_string(), "03".to_string(), n_year_as_string.clone()),
    //     ("02".to_string(), "05".to_string(), n_year_as_string.clone()),
    //     //("02".to_string(), "06".to_string(), n_year_as_string.clone()),
    //     // ("17".to_string(), "12".to_string(), n_year_as_string.clone()),
    //     ("30".to_string(), "12".to_string(), n_year_as_string.clone()),
    // ];

    println!("🎲 Starting lottery results batch fetch and save...\n");

    match fetch_and_save_multiple_results(&conn, &dates_to_fetch).await {
        Ok(results) => {
            println!("\n📊 Summary:");
            // for result in &results {
            //     let first_prize_num = if let Some(first_num) = result.data.first.number.first() {
            //         &first_num.value
            //     } else {
            //         "N/A"
            //     };
            //     println!(
            //         "  • Date: {} | Period: {:?} | First Prize: {} ({})",
            //         result.date, result.period, first_prize_num, result.data.first.price
            //     );
            // }
            println!("\n✅ All operations completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Error during batch operation: {}", e);
        }
    }

    // println!("\n📋 Generating HTML reports for available dates...");
    // match get_lottery_results_after_date(&conn, "2016-12-30", Some(1)) {
    //     Ok(latest_results) => {
    //         if latest_results.is_empty() {
    //             println!("⚠ No lottery data found in database for report generation.");
    //         } else {
    //             for (_, result) in latest_results.iter().enumerate() {
    //                 match generate_and_save_report(&conn, &result.draw_date) {
    //                     Ok(()) => println!("✅ Report generated for {}", result.draw_date),
    //                     Err(e) => println!(
    //                         "❌ Failed to generate report for {}: {}",
    //                         result.draw_date, e
    //                     ),
    //                 }
    //             }
    //         }
    //     }
    //     Err(e) => println!("❌ Error getting latest results: {}", e),
    // }

    // list_generated_files()?;

    Ok(())
}
