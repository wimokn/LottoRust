# Thai Lottery MCP Server

A Model Context Protocol (MCP) server that provides Claude with tools to interact with Thai Government Lottery data.

## Features

The MCP server exposes the following tools to Claude:

### Database Operations
- `create_database` - Initialize the lottery database
- `get_latest_lottery_results` - Get the most recent lottery results
- `get_lottery_by_date` - Get lottery result for a specific date
- `get_lottery_results_after_date` - Get results after a specific date
- `get_lottery_results_before_date` - Get results before a specific date
- `get_lottery_results_by_date_range` - Get results within a date range
- `get_lottery_results_by_year` - Get all results for a specific year
- `get_lottery_results_by_month` - Get results for a specific month
- `get_complete_lottery_data` - Get complete data including all prize numbers
- `search_number` - Search for specific lottery numbers across all results

### API Operations
- `fetch_and_save_multiple_results` - Fetch lottery data from official API for multiple dates
- `parse_and_insert_raw_json` - Parse and insert raw JSON lottery data

### Report Generation
- `generate_and_save_report` - Generate styled HTML reports for lottery results

## Setup

1. **Install Dependencies**
   ```bash
   cd mcp-server
   cargo build
   ```

2. **Configure Claude to use the MCP Server**
   
   Add the server configuration to your Claude settings file (typically `~/.config/claude/claude_desktop_config.json`):
   
   ```json
   {
     "mcpServers": {
       "lottery": {
         "command": "cargo",
         "args": ["run", "--bin", "lottery-mcp-server"],
         "cwd": "/Users/wimokn/Projects/W1m0k/LottoRust/mcp-server",
         "env": {
           "LOTTERY_DB_PATH": "../data/lottery.db"
         }
       }
     }
   }
   ```
   
   **Note**: Replace the `cwd` path with your actual project path.

3. **Restart Claude**
   
   Restart Claude desktop application to load the new MCP server.

## Usage Examples

Once configured, you can use Claude to interact with the lottery database:

- "What are the latest lottery results?"
- "Search for the number 123 in all lottery draws"
- "Get lottery results for March 2024"
- "Fetch lottery data for specific dates: [['01', '03', '2024'], ['16', '03', '2024']]"
- "Generate a report for the lottery draw on 2024-03-01"

## Environment Variables

- `LOTTERY_DB_PATH` - Path to the SQLite database file (default: "data/lottery.db")

## Database Schema

The server uses the same database schema as the main application:
- `lottery_results` - Stores draw dates, periods, and metadata
- `prize_numbers` - Stores individual winning numbers with categories

## Error Handling

All tools return JSON responses with:
- `success`: boolean indicating if the operation succeeded
- `error`: error message if the operation failed
- `result`/`results`: the actual data if successful

## Development

To test the MCP server directly:

```bash
cd mcp-server
cargo run --bin lottery-mcp-server
```

The server communicates via stdin/stdout using the MCP protocol.