use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
}

pub fn load() -> Result<Config> {
    let database_url = env::var("LOTTERY_DB_PATH")
        .unwrap_or_else(|_| "data/lottery.db".to_string());

    Ok(Config { database_url })
}