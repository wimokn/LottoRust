use chrono::{Datelike, NaiveDate};
use std::error::Error;
use std::fs;
use std::path::Path;

pub fn format_date_for_api(date: &str, month: &str, year: &str) -> String {
    format!("{}-{:0>2}-{:0>2}", year, month, date)
}

pub fn show_project_structure() {
    println!("\n📂 Project Structure:");
    println!("LottoRust/");
    println!("├── src/");
    println!("│   ├── main.rs          # Main application code");
    println!("│   ├── database.rs      # Database operations");
    println!("│   ├── api.rs           # API calls and data fetching");
    println!("│   ├── reports.rs       # HTML report generation");
    println!("│   ├── types.rs         # Data structures");
    println!("│   └── utils.rs         # Utility functions");
    println!("├── data/");
    println!("│   └── lottery.db       # SQLite database");
    println!("├── reports/");
    println!("│   └── lottery_report_*.html  # Generated HTML reports");
    println!("├── Cargo.toml");
    println!("└── README.md");
    println!();
}

pub fn list_generated_files() -> Result<(), Box<dyn Error>> {
    println!("📋 Generated Files:");

    if Path::new("data/lottery.db").exists() {
        let metadata = fs::metadata("data/lottery.db")?;
        println!("  🗄️  data/lottery.db ({} bytes)", metadata.len());
    } else {
        println!("  ⚠️  data/lottery.db (not found)");
    }

    if Path::new("reports").exists() {
        let entries = fs::read_dir("reports")?;
        let mut report_count = 0;
        for entry in entries {
            let entry = entry?;
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".html") {
                    let metadata = entry.metadata()?;
                    println!("  📄 reports/{} ({} bytes)", filename, metadata.len());
                    report_count += 1;
                }
            }
        }
        if report_count == 0 {
            println!("  ⚠️  No HTML reports found in reports/");
        }
    }

    println!();
    Ok(())
}

pub fn generate_lottery_dates(year: i32) -> Vec<(String, String, String)> {
    let mut dates_to_fetch = Vec::new();

    for month in 1..=12 {
        for day in [1, 16] {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                dates_to_fetch.push((
                    format!("{:02}", date.day()),
                    format!("{:02}", date.month()),
                    date.year().to_string(),
                ));
            }
        }
    }

    dates_to_fetch
}
