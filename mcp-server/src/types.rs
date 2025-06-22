use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LotteryRequest {
    pub date: String,
    pub month: String,
    pub year: String,
}

#[derive(Deserialize, Debug)]
pub struct LotteryResponse {
    #[serde(rename = "statusMessage")]
    pub status_message: String,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
    pub status: bool,
    pub response: Option<ResponseData>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseData {
    pub result: Option<LotteryResult>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LotteryResult {
    pub date: String,
    pub period: Vec<i32>,
    pub data: LotteryData,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LotteryData {
    pub first: PrizeCategory,
    pub second: PrizeCategory,
    pub third: PrizeCategory,
    pub fourth: PrizeCategory,
    pub fifth: PrizeCategory,
    pub last2: PrizeCategory,
    pub last3f: PrizeCategory,
    pub last3b: PrizeCategory,
    pub near1: PrizeCategory,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PrizeCategory {
    pub price: String,
    pub number: Vec<PrizeNumber>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PrizeNumber {
    pub round: i32,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct LotteryResultRow {
    pub id: i64,
    pub draw_date: String,
    pub period: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PrizeNumberRow {
    pub id: i64,
    pub lottery_id: i64,
    pub category: String,
    pub prize_amount: String,
    pub number_value: String,
    pub round_number: i32,
}