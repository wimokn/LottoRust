use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
use tracing::{info, warn};

use crate::use_cases::{LotteryUseCase, ApiUseCase, ReportUseCase};

#[derive(Debug, serde::Deserialize)]
struct JsonRpcRequest {
    #[serde(default = "default_jsonrpc")]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

#[derive(Debug, serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Debug, serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, serde::Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

pub struct MCPHandler {
    lottery_use_case: Arc<LotteryUseCase>,
    api_use_case: Arc<ApiUseCase>,
    report_use_case: Arc<ReportUseCase>,
}

impl MCPHandler {
    pub fn new(
        lottery_use_case: Arc<LotteryUseCase>,
        api_use_case: Arc<ApiUseCase>,
        report_use_case: Arc<ReportUseCase>,
    ) -> Self {
        Self {
            lottery_use_case,
            api_use_case,
            report_use_case,
        }
    }

    pub async fn serve<R, W>(self, reader: R, mut writer: W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => req,
                Err(e) => {
                    warn!("Failed to parse request: {} - Line: {}", e, line);
                    // Send proper error response for malformed JSON
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: Some(json!(e.to_string())),
                        }),
                        id: None,
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    writeln!(writer, "{}", response_json)?;
                    writer.flush()?;
                    continue;
                }
            };

            // Check if this is a notification (no id field or method starts with notifications/)
            let is_notification = request.id.is_none() || request.method.starts_with("notifications/");
            
            if is_notification {
                // For notifications, just handle them but don't send any response
                if request.method == "notifications/initialized" {
                    info!("🎰 Client initialized");
                }
                continue;
            }
            
            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;
            writeln!(writer, "{}", response_json)?;
            writer.flush()?;
        }

        Ok(())
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
                id: Some(request.id.unwrap_or(json!(1))),
            },
        }
    }

    async fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("🎰 Initializing lottery MCP server");
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
            id: Some(id.unwrap_or(json!(1))),
        }
    }

    async fn handle_list_tools(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = self.get_tools();
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({ "tools": tools })),
            error: None,
            id: Some(id.unwrap_or(json!(1))),
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
                    id: Some(id.unwrap_or(json!(1))),
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
                    id: Some(id.unwrap_or(json!(1))),
                };
            }
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
        let arguments_map: HashMap<String, Value> = serde_json::from_value(arguments).unwrap_or_default();

        let result = self.execute_tool(tool_name, &arguments_map).await;

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
                id: Some(id.unwrap_or(json!(1))),
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Tool execution error: {}", e),
                    data: None,
                }),
                id: Some(id.unwrap_or(json!(1))),
            },
        }
    }

    async fn execute_tool(&self, tool_name: &str, arguments: &HashMap<String, Value>) -> Result<String> {
        match tool_name {
            "parse_and_insert_raw_json" => self.lottery_use_case.parse_and_insert_raw_json(arguments).await,
            "fetch_and_save_multiple_results" => self.api_use_case.fetch_and_save_multiple_results(arguments).await,
            "get_lottery_results_after_date" => self.lottery_use_case.get_lottery_results_after_date(arguments).await,
            "get_lottery_results_before_date" => self.lottery_use_case.get_lottery_results_before_date(arguments).await,
            "get_lottery_results_by_date_range" => self.lottery_use_case.get_lottery_results_by_date_range(arguments).await,
            "get_lottery_results_by_year" => self.lottery_use_case.get_lottery_results_by_year(arguments).await,
            "get_lottery_results_by_month" => self.lottery_use_case.get_lottery_results_by_month(arguments).await,
            "get_latest_lottery_results" => self.lottery_use_case.get_latest_lottery_results(arguments).await,
            "get_lottery_by_date" => self.lottery_use_case.get_lottery_by_date(arguments).await,
            "search_number" => self.lottery_use_case.search_number(arguments).await,
            "get_complete_lottery_data" => self.lottery_use_case.get_complete_lottery_data(arguments).await,
            "generate_and_save_report" => self.report_use_case.generate_and_save_report(arguments).await,
            "create_database" => self.lottery_use_case.create_database(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    fn get_tools(&self) -> Vec<Tool> {
        vec![
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
        ]
    }
}

pub fn stdio() -> (BufReader<io::Stdin>, io::Stdout) {
    (BufReader::new(io::stdin()), io::stdout())
}