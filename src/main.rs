//! Application entry point.
//!
//! Initializes configuration, connects to the vault, and runs the
//! polling loop on a single-threaded tokio runtime.

mod config;
mod contract;
mod fmt;
mod log;
mod telegram;

/// Telegram API request budget. The Bot API answers in <1s under
/// normal conditions; 10s catches DNS / TLS hangs without making
/// the user wait minutes for a failed alert.
const TELEGRAM_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = config::Config::from_env();
    cfg.print_banner();
    let vault = contract::connect(&cfg)?;
    let http = reqwest::Client::builder()
        .timeout(TELEGRAM_TIMEOUT)
        .build()?;
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
