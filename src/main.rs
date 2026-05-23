//! Application entry point.
//!
//! Initializes configuration, connects to the vault, and runs the
//! polling loop on a single-threaded tokio runtime.

mod config;
mod contract;
mod fmt;
mod log;
mod state;
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
    let mut persisted = state::load(&cfg.state_file);
    let mut interval = tokio::time::interval(cfg.poll_interval);
    loop {
        interval.tick().await;
        let previous = persisted.alerted;
        contract::poll_once(&vault, &cfg, &http, &mut persisted.alerted).await;
        if persisted.alerted != previous {
            if let Err(e) = state::save(&cfg.state_file, &persisted) {
                log::warn(&format!(
                    "failed to persist state to {}: {e}",
                    cfg.state_file.display(),
                ));
            }
        }
    }
}
