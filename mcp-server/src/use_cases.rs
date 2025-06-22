use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::*;
use crate::api::*;
use crate::reports;

pub struct LotteryUseCase {
    connection: Arc<rusqlite::Connection>,
}

impl LotteryUseCase {
    pub fn new(connection: Arc<rusqlite::Connection>) -> Self {
        Self { connection }
    }

    pub async fn parse_and_insert_raw_json(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let raw_json = arguments
            .get("raw_json")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing raw_json parameter"))?;

        let lottery_id = parse_and_insert_raw_json(&self.connection, raw_json)?;
        
        Ok(json!({
            "success": true,
            "lottery_id": lottery_id,
            "message": format!("Successfully inserted lottery with ID: {}", lottery_id)
        }).to_string())
    }

    pub async fn get_lottery_results_after_date(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);
        let results = get_lottery_results_after_date(&self.connection, date, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_lottery_results_before_date(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);
        let results = get_lottery_results_before_date(&self.connection, date, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_lottery_results_by_date_range(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let start_date = arguments
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing start_date parameter"))?;

        let end_date = arguments
            .get("end_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing end_date parameter"))?;

        let results = get_lottery_results_by_date_range(&self.connection, start_date, end_date)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_lottery_results_by_year(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        let results = get_lottery_results_by_year(&self.connection, year)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_lottery_results_by_month(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        let month = arguments
            .get("month")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing month parameter"))?;

        let results = get_lottery_results_by_month(&self.connection, year, month)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_latest_lottery_results(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_i64())
            .map(|l| l as i32)
            .unwrap_or(10);

        let results = get_latest_lottery_results(&self.connection, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_lottery_by_date(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let result = get_lottery_by_date(&self.connection, date)?;
        
        Ok(json!({
            "success": true,
            "result": result
        }).to_string())
    }

    pub async fn search_number(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let number = arguments
            .get("number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing number parameter"))?;

        let results = search_number(&self.connection, number)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    pub async fn get_complete_lottery_data(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let result = get_complete_lottery_data(&self.connection, date)?;
        
        Ok(json!({
            "success": true,
            "result": result
        }).to_string())
    }

    pub async fn create_database(&self, _arguments: &HashMap<String, Value>) -> Result<String> {
        create_database()?;
        
        Ok(json!({
            "success": true,
            "message": "Database created successfully"
        }).to_string())
    }
}

pub struct ApiUseCase {
    connection: Arc<rusqlite::Connection>,
}

impl ApiUseCase {
    pub fn new(connection: Arc<rusqlite::Connection>) -> Self {
        Self { connection }
    }

    pub async fn fetch_and_save_multiple_results(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let dates_json = arguments
            .get("dates")
            .ok_or_else(|| anyhow::anyhow!("Missing dates parameter"))?;

        let dates: Vec<(String, String, String)> = serde_json::from_value(dates_json.clone())?;
        let results = fetch_and_save_multiple_results(&self.connection, &dates).await
            .map_err(|e| anyhow::anyhow!("API error: {}", e))?;
        
        Ok(json!({
            "success": true,
            "results_count": results.len(),
            "results": results
        }).to_string())
    }
}

pub struct ReportUseCase {
    connection: Arc<rusqlite::Connection>,
}

impl ReportUseCase {
    pub fn new(connection: Arc<rusqlite::Connection>) -> Self {
        Self { connection }
    }

    pub async fn generate_and_save_report(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        reports::generate_and_save_report(&self.connection, date)
            .map_err(|e| anyhow::anyhow!("Report generation error: {}", e))?;
        
        Ok(json!({
            "success": true,
            "message": format!("Report generated successfully for date: {}", date)
        }).to_string())
    }
}