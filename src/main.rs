//! Application entry point.
//!
//! Initializes configuration, connects to the vault, and runs the
//! polling loop on a single-threaded tokio runtime.

mod config;
mod contract;
mod fmt;
mod log;
mod telegram;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = config::Config::from_env();
    cfg.print_banner();
    let vault = contract::connect(&cfg)?;
    let http = reqwest::Client::new();
    if let Some(tg) = &cfg.telegram {
        telegram::send_startup(&http, tg, &cfg).await;
    }
    let mut alerted = false;
    let mut interval = tokio::time::interval(cfg.poll_interval);
    loop {
        interval.tick().await;
        contract::poll_once(&vault, &cfg, &http, &mut alerted).await;
    }
}
