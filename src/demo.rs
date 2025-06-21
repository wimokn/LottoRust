use std::error::Error;
use std::collections::HashMap;
use rusqlite::Connection;
use crate::database::{
    get_all_lottery_results, get_latest_lottery_results, get_prize_numbers_by_category,
    search_number, get_complete_lottery_data
};

pub fn demonstrate_query_functions(conn: &Connection) -> Result<(), Box<dyn Error>> {
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

                    let mut category_counts = HashMap::new();
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