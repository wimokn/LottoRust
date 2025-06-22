use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod api;
mod config;
mod connection;
mod database;
mod mcp_handler;
mod reports;
mod types;
mod use_cases;
mod utils;

use connection::conn;
use mcp_handler::{MCPHandler, stdio};
use use_cases::{ApiUseCase, LotteryUseCase, ReportUseCase};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Let's roll your lottery numbers.");

    let db_conn = conn(&config.database_url)?;
    let db_conn_arc = Arc::new(db_conn);

    let lottery_use_case = { LotteryUseCase::new(Arc::clone(&db_conn_arc)) };

    let api_use_case = { ApiUseCase::new(Arc::clone(&db_conn_arc)) };

    let report_use_case =
        { ReportUseCase::new(Arc::clone(&db_conn_arc), config.report_path.clone()) };

    let handler = MCPHandler::new(
        Arc::new(lottery_use_case),
        Arc::new(api_use_case),
        Arc::new(report_use_case),
    );

    let (reader, writer) = stdio();

    handler.serve(reader, writer).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    Ok(())
}
