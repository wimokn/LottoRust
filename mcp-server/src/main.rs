use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use tokio::main;
use tracing::{info, warn};

mod api;
mod database;
mod reports;
mod types;
mod utils;

use database::*;
use api::*;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Clone)]
struct LotteryMcpServer {
    db_path: String,
}

impl LotteryMcpServer {
    fn new(db_path: String) -> Self {
        Self { db_path }
    }

    fn get_connection(&self) -> Result<rusqlite::Connection> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id).await,
            "tools/list" => self.handle_list_tools(request.id).await,
            "tools/call" => self.handle_call_tool(request.params, request.id).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
                id: request.id,
            },
        }
    }

    async fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Initializing lottery MCP server");
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "lottery-mcp-server",
                    "version": "0.1.0"
                }
            })),
            error: None,
            id,
        }
    }

    async fn handle_list_tools(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "parse_and_insert_raw_json".to_string(),
                description: "Parse raw JSON lottery data and insert into database".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "raw_json": {
                            "type": "string",
                            "description": "Raw JSON string containing lottery result data"
                        }
                    },
                    "required": ["raw_json"]
                }),
            },
            Tool {
                name: "fetch_and_save_multiple_results".to_string(),
                description: "Fetch lottery results from API for multiple dates and save to database".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "dates": {
                            "type": "array",
                            "description": "Array of date tuples [day, month, year]",
                            "items": {
                                "type": "array",
                                "items": {"type": "string"},
                                "minItems": 3,
                                "maxItems": 3
                            }
                        }
                    },
                    "required": ["dates"]
                }),
            },
            Tool {
                name: "get_lottery_results_after_date".to_string(),
                description: "Get lottery results after a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Optional limit for number of results"
                        }
                    },
                    "required": ["date"]
                }),
            },
            Tool {
                name: "get_lottery_results_before_date".to_string(),
                description: "Get lottery results before a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Optional limit for number of results"
                        }
                    },
                    "required": ["date"]
                }),
            },
            Tool {
                name: "get_lottery_results_by_date_range".to_string(),
                description: "Get lottery results within a date range".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "start_date": {
                            "type": "string",
                            "description": "Start date in YYYY-MM-DD format"
                        },
                        "end_date": {
                            "type": "string",
                            "description": "End date in YYYY-MM-DD format"
                        }
                    },
                    "required": ["start_date", "end_date"]
                }),
            },
            Tool {
                name: "get_lottery_results_by_year".to_string(),
                description: "Get all lottery results for a specific year".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "year": {
                            "type": "string",
                            "description": "Year in YYYY format"
                        }
                    },
                    "required": ["year"]
                }),
            },
            Tool {
                name: "get_lottery_results_by_month".to_string(),
                description: "Get lottery results for a specific month and year".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "year": {
                            "type": "string",
                            "description": "Year in YYYY format"
                        },
                        "month": {
                            "type": "string",
                            "description": "Month in MM format"
                        }
                    },
                    "required": ["year", "month"]
                }),
            },
            Tool {
                name: "get_latest_lottery_results".to_string(),
                description: "Get the latest lottery results".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Number of results to return (default: 10)"
                        }
                    }
                }),
            },
            Tool {
                name: "get_lottery_by_date".to_string(),
                description: "Get lottery result for a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format"
                        }
                    },
                    "required": ["date"]
                }),
            },
            Tool {
                name: "search_number".to_string(),
                description: "Search for a specific lottery number across all results".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "number": {
                            "type": "string",
                            "description": "Lottery number to search for"
                        }
                    },
                    "required": ["number"]
                }),
            },
            Tool {
                name: "get_complete_lottery_data".to_string(),
                description: "Get complete lottery data including all prize numbers for a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format"
                        }
                    },
                    "required": ["date"]
                }),
            },
            Tool {
                name: "generate_and_save_report".to_string(),
                description: "Generate and save HTML report for a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format"
                        }
                    },
                    "required": ["date"]
                }),
            },
            Tool {
                name: "create_database".to_string(),
                description: "Create and initialize the lottery database".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({ "tools": tools })),
            error: None,
            id,
        }
    }

    async fn handle_call_tool(&self, params: Option<Value>, id: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                    id,
                };
            }
        };

        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(name) => name,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing tool name".to_string(),
                        data: None,
                    }),
                    id,
                };
            }
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
        let arguments_map: HashMap<String, Value> = serde_json::from_value(arguments).unwrap_or_default();

        let result = match tool_name {
            "parse_and_insert_raw_json" => self.parse_and_insert_raw_json_tool(&arguments_map).await,
            "fetch_and_save_multiple_results" => self.fetch_and_save_multiple_results_tool(&arguments_map).await,
            "get_lottery_results_after_date" => self.get_lottery_results_after_date_tool(&arguments_map).await,
            "get_lottery_results_before_date" => self.get_lottery_results_before_date_tool(&arguments_map).await,
            "get_lottery_results_by_date_range" => self.get_lottery_results_by_date_range_tool(&arguments_map).await,
            "get_lottery_results_by_year" => self.get_lottery_results_by_year_tool(&arguments_map).await,
            "get_lottery_results_by_month" => self.get_lottery_results_by_month_tool(&arguments_map).await,
            "get_latest_lottery_results" => self.get_latest_lottery_results_tool(&arguments_map).await,
            "get_lottery_by_date" => self.get_lottery_by_date_tool(&arguments_map).await,
            "search_number" => self.search_number_tool(&arguments_map).await,
            "get_complete_lottery_data" => self.get_complete_lottery_data_tool(&arguments_map).await,
            "generate_and_save_report" => self.generate_and_save_report_tool(&arguments_map).await,
            "create_database" => self.create_database_tool(&arguments_map).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        };

        match result {
            Ok(content) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                })),
                error: None,
                id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Tool execution error: {}", e),
                    data: None,
                }),
                id,
            },
        }
    }

    async fn parse_and_insert_raw_json_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let raw_json = arguments
            .get("raw_json")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing raw_json parameter"))?;

        let conn = self.get_connection()?;
        let lottery_id = parse_and_insert_raw_json(&conn, raw_json)?;
        
        Ok(json!({
            "success": true,
            "lottery_id": lottery_id,
            "message": format!("Successfully inserted lottery with ID: {}", lottery_id)
        }).to_string())
    }

    async fn fetch_and_save_multiple_results_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let dates_json = arguments
            .get("dates")
            .ok_or_else(|| anyhow::anyhow!("Missing dates parameter"))?;

        let dates: Vec<(String, String, String)> = serde_json::from_value(dates_json.clone())?;
        let conn = self.get_connection()?;
        let results = fetch_and_save_multiple_results(&conn, &dates).await
            .map_err(|e| anyhow::anyhow!("API error: {}", e))?;
        
        Ok(json!({
            "success": true,
            "results_count": results.len(),
            "results": results
        }).to_string())
    }

    async fn get_lottery_results_after_date_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);
        let conn = self.get_connection()?;
        let results = get_lottery_results_after_date(&conn, date, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_lottery_results_before_date_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);
        let conn = self.get_connection()?;
        let results = get_lottery_results_before_date(&conn, date, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_lottery_results_by_date_range_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let start_date = arguments
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing start_date parameter"))?;

        let end_date = arguments
            .get("end_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing end_date parameter"))?;

        let conn = self.get_connection()?;
        let results = get_lottery_results_by_date_range(&conn, start_date, end_date)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_lottery_results_by_year_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        let conn = self.get_connection()?;
        let results = get_lottery_results_by_year(&conn, year)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_lottery_results_by_month_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        let month = arguments
            .get("month")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing month parameter"))?;

        let conn = self.get_connection()?;
        let results = get_lottery_results_by_month(&conn, year, month)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_latest_lottery_results_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_i64())
            .map(|l| l as i32)
            .unwrap_or(10);

        let conn = self.get_connection()?;
        let results = get_latest_lottery_results(&conn, limit)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_lottery_by_date_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let conn = self.get_connection()?;
        let result = get_lottery_by_date(&conn, date)?;
        
        Ok(json!({
            "success": true,
            "result": result
        }).to_string())
    }

    async fn search_number_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let number = arguments
            .get("number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing number parameter"))?;

        let conn = self.get_connection()?;
        let results = search_number(&conn, number)?;
        
        Ok(json!({
            "success": true,
            "results": results
        }).to_string())
    }

    async fn get_complete_lottery_data_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let conn = self.get_connection()?;
        let result = get_complete_lottery_data(&conn, date)?;
        
        Ok(json!({
            "success": true,
            "result": result
        }).to_string())
    }

    async fn generate_and_save_report_tool(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let conn = self.get_connection()?;
        reports::generate_and_save_report(&conn, date)
            .map_err(|e| anyhow::anyhow!("Report generation error: {}", e))?;
        
        Ok(json!({
            "success": true,
            "message": format!("Report generated successfully for date: {}", date)
        }).to_string())
    }

    async fn create_database_tool(&self, _arguments: &HashMap<String, Value>) -> Result<String> {
        create_database()?;
        
        Ok(json!({
            "success": true,
            "message": "Database created successfully"
        }).to_string())
    }
}

#[main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("LOTTERY_DB_PATH").unwrap_or_else(|_| "data/lottery.db".to_string());
    let server = LotteryMcpServer::new(db_path);

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut stdout = io::stdout();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = server.handle_request(request).await;
        let response_json = serde_json::to_string(&response)?;
        
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}