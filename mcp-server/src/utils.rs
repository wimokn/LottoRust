use chrono::{Datelike, NaiveDate};
use std::error::Error;
use std::fs;
use std::path::Path;

pub fn format_date_for_api(date: &str, month: &str, year: &str) -> String {
    format!("{}-{:0>2}-{:0>2}", year, month, date)
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
