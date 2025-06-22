use anyhow::Result;
use mcp_core::{
    protocol::{
        CallToolRequest, CallToolResult, GetToolsRequest, GetToolsResult, InitializeRequest,
        InitializeResult, ListToolsRequest, ListToolsResult, Tool, ToolInput, ToolResult,
    },
    Client, RequestId, Server,
};
use mcp_server::stdio::StdioServer;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::main;
use tracing::{info, warn};

mod api;
mod database;
mod reports;
mod types;
mod utils;

use database::*;
use api::*;
use types::*;

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

    async fn handle_call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let tool_name = &request.params.name;
        let arguments = &request.params.arguments.unwrap_or_default();

        match tool_name.as_str() {
            "parse_and_insert_raw_json" => self.parse_and_insert_raw_json_tool(arguments).await,
            "fetch_and_save_multiple_results" => {
                self.fetch_and_save_multiple_results_tool(arguments).await
            }
            "get_lottery_results_after_date" => {
                self.get_lottery_results_after_date_tool(arguments).await
            }
            "get_lottery_results_before_date" => {
                self.get_lottery_results_before_date_tool(arguments).await
            }
            "get_lottery_results_by_date_range" => {
                self.get_lottery_results_by_date_range_tool(arguments).await
            }
            "get_lottery_results_by_year" => {
                self.get_lottery_results_by_year_tool(arguments).await
            }
            "get_lottery_results_by_month" => {
                self.get_lottery_results_by_month_tool(arguments).await
            }
            "get_latest_lottery_results" => {
                self.get_latest_lottery_results_tool(arguments).await
            }
            "get_lottery_by_date" => self.get_lottery_by_date_tool(arguments).await,
            "search_number" => self.search_number_tool(arguments).await,
            "get_complete_lottery_data" => self.get_complete_lottery_data_tool(arguments).await,
            "generate_and_save_report" => self.generate_and_save_report_tool(arguments).await,
            "create_database" => self.create_database_tool(arguments).await,
            _ => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: format!("Unknown tool: {}", tool_name),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn parse_and_insert_raw_json_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let raw_json = arguments
            .get("raw_json")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing raw_json parameter"))?;

        match self.get_connection() {
            Ok(conn) => match parse_and_insert_raw_json(&conn, raw_json) {
                Ok(lottery_id) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "lottery_id": lottery_id,
                            "message": format!("Successfully inserted lottery with ID: {}", lottery_id)
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn fetch_and_save_multiple_results_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let dates_json = arguments
            .get("dates")
            .ok_or_else(|| anyhow::anyhow!("Missing dates parameter"))?;

        let dates: Vec<(String, String, String)> = serde_json::from_value(dates_json.clone())
            .map_err(|e| anyhow::anyhow!("Invalid dates format: {}", e))?;

        match self.get_connection() {
            Ok(conn) => match fetch_and_save_multiple_results(&conn, &dates).await {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results_count": results.len(),
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Fetch error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_results_after_date_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);

        match self.get_connection() {
            Ok(conn) => match get_lottery_results_after_date(&conn, date, limit) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_results_before_date_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        let limit = arguments.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);

        match self.get_connection() {
            Ok(conn) => match get_lottery_results_before_date(&conn, date, limit) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_results_by_date_range_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let start_date = arguments
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing start_date parameter"))?;

        let end_date = arguments
            .get("end_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing end_date parameter"))?;

        match self.get_connection() {
            Ok(conn) => match get_lottery_results_by_date_range(&conn, start_date, end_date) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_results_by_year_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        match self.get_connection() {
            Ok(conn) => match get_lottery_results_by_year(&conn, year) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_results_by_month_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let year = arguments
            .get("year")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing year parameter"))?;

        let month = arguments
            .get("month")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing month parameter"))?;

        match self.get_connection() {
            Ok(conn) => match get_lottery_results_by_month(&conn, year, month) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_latest_lottery_results_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_i64())
            .map(|l| l as i32)
            .unwrap_or(10);

        match self.get_connection() {
            Ok(conn) => match get_latest_lottery_results(&conn, limit) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_lottery_by_date_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        match self.get_connection() {
            Ok(conn) => match get_lottery_by_date(&conn, date) {
                Ok(result) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "result": result
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn search_number_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let number = arguments
            .get("number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing number parameter"))?;

        match self.get_connection() {
            Ok(conn) => match search_number(&conn, number) {
                Ok(results) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "results": results
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn get_complete_lottery_data_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        match self.get_connection() {
            Ok(conn) => match get_complete_lottery_data(&conn, date) {
                Ok(result) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "result": result
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Database error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn generate_and_save_report_tool(
        &self,
        arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let date = arguments
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing date parameter"))?;

        match self.get_connection() {
            Ok(conn) => match reports::generate_and_save_report(&conn, date) {
                Ok(_) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": true,
                            "message": format!("Report generated successfully for date: {}", date)
                        })
                        .to_string(),
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(CallToolResult {
                    content: vec![ToolResult::Text {
                        text: json!({
                            "success": false,
                            "error": format!("Report generation error: {}", e)
                        })
                        .to_string(),
                    }],
                    is_error: Some(true),
                }),
            },
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Connection error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn create_database_tool(
        &self,
        _arguments: &HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        match create_database() {
            Ok(_) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": true,
                        "message": "Database created successfully"
                    })
                    .to_string(),
                }],
                is_error: Some(false),
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![ToolResult::Text {
                    text: json!({
                        "success": false,
                        "error": format!("Database creation error: {}", e)
                    })
                    .to_string(),
                }],
                is_error: Some(true),
            }),
        }
    }
}

#[main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("LOTTERY_DB_PATH").unwrap_or_else(|_| "data/lottery.db".to_string());
    let server = LotteryMcpServer::new(db_path);

    let stdio_server = StdioServer::new();

    stdio_server
        .run(move |request| {
            let server = server.clone();
            async move {
                match request {
                    mcp_core::protocol::Request::Initialize(req) => {
                        info!("Initializing lottery MCP server");
                        Ok(mcp_core::protocol::Response::Initialize(InitializeResult {
                            protocol_version: "0.1.0".to_string(),
                            capabilities: mcp_core::protocol::ServerCapabilities {
                                tools: Some(mcp_core::protocol::ToolsCapability { list_changed: Some(false) }),
                                ..Default::default()
                            },
                            server_info: mcp_core::protocol::ServerInfo {
                                name: "lottery-mcp-server".to_string(),
                                version: "0.1.0".to_string(),
                            },
                        }))
                    }
                    mcp_core::protocol::Request::ListTools(req) => {
                        Ok(mcp_core::protocol::Response::ListTools(ListToolsResult {
                            tools: vec![
                                Tool {
                                    name: "parse_and_insert_raw_json".to_string(),
                                    description: Some("Parse raw JSON lottery data and insert into database".to_string()),
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
                                    description: Some("Fetch lottery results from API for multiple dates and save to database".to_string()),
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
                                    description: Some("Get lottery results after a specific date".to_string()),
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
                                    description: Some("Get lottery results before a specific date".to_string()),
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
                                    description: Some("Get lottery results within a date range".to_string()),
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
                                    description: Some("Get all lottery results for a specific year".to_string()),
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
                                    description: Some("Get lottery results for a specific month and year".to_string()),
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
                                    description: Some("Get the latest lottery results".to_string()),
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
                                    description: Some("Get lottery result for a specific date".to_string()),
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
                                    description: Some("Search for a specific lottery number across all results".to_string()),
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
                                    description: Some("Get complete lottery data including all prize numbers for a specific date".to_string()),
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
                                    description: Some("Generate and save HTML report for a specific date".to_string()),
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
                                    description: Some("Create and initialize the lottery database".to_string()),
                                    input_schema: json!({
                                        "type": "object",
                                        "properties": {}
                                    }),
                                },
                            ],
                        }))
                    }
                    mcp_core::protocol::Request::CallTool(req) => {
                        match server.handle_call_tool(req).await {
                            Ok(result) => Ok(mcp_core::protocol::Response::CallTool(result)),
                            Err(e) => {
                                warn!("Tool call error: {}", e);
                                Ok(mcp_core::protocol::Response::CallTool(CallToolResult {
                                    content: vec![ToolResult::Text {
                                        text: format!("Error: {}", e),
                                    }],
                                    is_error: Some(true),
                                }))
                            }
                        }
                    }
                    _ => {
                        warn!("Unsupported request type");
                        Err(anyhow::anyhow!("Unsupported request type"))
                    }
                }
            }
        })
        .await?;

    Ok(())
}