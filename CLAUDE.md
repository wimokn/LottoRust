# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Run
- `cargo build` - Build the project
- `cargo run` - Build and run the main application
- `cargo check` - Quick syntax and type checking
- `cargo test` - Run tests (if any exist)

### Development
- `cargo clippy` - Run linter for code quality checks
- `cargo fmt` - Format code according to Rust standards

## Architecture

This is a Thai Government Lottery result fetcher and report generator written in Rust. The application consists of several key modules:

### Core Components

- **main.rs** - Entry point that orchestrates lottery data fetching and report generation
- **api.rs** - HTTP client for fetching lottery results from glo.or.th API with rate limiting
- **database.rs** - SQLite database operations for storing lottery results and prize numbers
- **types.rs** - Serde data structures for API requests/responses and database models
- **reports.rs** - HTML report generation with Thai language support and styling
- **utils.rs** - Utility functions for date formatting and file operations

### Data Flow

1. **API Layer**: Fetches lottery results from `https://www.glo.or.th/api/checking/getLotteryResult`
2. **Database Layer**: Two-table schema - `lottery_results` (draw metadata) and `prize_numbers` (individual winning numbers)
3. **Report Layer**: Generates styled HTML reports in Thai language with responsive design

### Database Schema

- **lottery_results**: Stores draw dates, periods, and metadata
- **prize_numbers**: Stores individual winning numbers linked to lottery results via foreign key
- Prize categories: first, second, third, fourth, fifth, last2, last3f, last3b, near1

### Key Features

- Batch fetching with duplicate detection to avoid re-fetching existing data
- Rate limiting (1 second delay between API calls)
- Comprehensive HTML report generation with CSS styling
- Thai language support for prize category names
- Raw JSON parsing capability for manual data insertion

### File Structure

- `data/` - Contains SQLite database file
- `reports/` - Generated HTML reports named by date
- `src/` - All Rust source code modules
- `mcp-server/` - Model Context Protocol server for Claude integration

## MCP Server

The project includes an MCP (Model Context Protocol) server that allows Claude to interact with the lottery database through standardized tools.

### MCP Commands
- `cd mcp-server && cargo build` - Build the MCP server
- `cd mcp-server && cargo run --bin lottery-mcp-server` - Run the MCP server

### MCP Configuration
Add to Claude desktop config (`~/.config/claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "lottery": {
      "command": "cargo",
      "args": ["run", "--bin", "lottery-mcp-server"],
      "cwd": "/path/to/LottoRust/mcp-server",
      "env": {
        "LOTTERY_DB_PATH": "../data/lottery.db"
      }
    }
  }
}
```

### Available MCP Tools
- Database operations: get/search lottery results by date, year, month, etc.
- API operations: fetch and save lottery data from official API
- Report generation: create HTML reports for specific dates
- Raw data insertion: parse and insert JSON lottery data