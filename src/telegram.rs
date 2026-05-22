//! Telegram Bot API integration for startup and alert notifications.

use serde::Serialize;

use crate::config::Config;
use crate::fmt;
use crate::log;

/// Credentials needed to send messages via the Telegram Bot API.
#[derive(Clone)]
pub struct TelegramConfig {
    bot_token: String,
    chat_id: String,
}

impl TelegramConfig {
    /// Read `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` from the
    /// environment. Returns `None` if either is empty or unset,
    /// allowing the bot to run in console-only mode.
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
        if token.is_empty() || chat_id.is_empty() {
            return None;
        }
        Some(Self {
            bot_token: token,
            chat_id,
        })
    }

    /// Build the `sendMessage` endpoint URL for this bot.
    fn api_url(&self) -> String {
        format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token)
    }
}

/// Payload for the Telegram `sendMessage` API call.
#[derive(Serialize)]
struct SendMessage<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
    disable_web_page_preview: bool,
}

/// Send an HTML-formatted message to the configured Telegram chat.
///
/// Uses the provided [`reqwest::Client`] for connection reuse.
/// Logs the outcome (success or failure) to the console.
pub async fn send(client: &reqwest::Client, tg: &TelegramConfig, message: &str) {
    let body = SendMessage {
        chat_id: &tg.chat_id,
        text: message,
        parse_mode: "HTML",
        disable_web_page_preview: false,
    };
    match client.post(tg.api_url()).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            log::positive("Telegram alert sent");
        }
        Ok(resp) => {
            log::error(&format!("Telegram API returned {}", resp.status()));
        }
        Err(e) => {
            log::error(&format!("Telegram request failed: {e}"));
        }
    }
}

/// Send a one-time startup notification so the user knows
/// the bot is online and monitoring.
pub async fn send_startup(client: &reqwest::Client, tg: &TelegramConfig, cfg: &Config) {
    let msg = format!(
    "\u{2705} <b>VaultWatch Online</b>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{1F3E6} <b>Vault:</b>  {}\n\
     \u{1F517} <a href=\"https://etherscan.io/address/{}\">{}</a>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{2699}\u{FE0F} <b>Polling:</b>  every {}s\n\
     \u{1F3AF} <b>Threshold:</b>  {}\n\
     \u{1F522} <b>Decimals:</b>  {}\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{23F0} <i>Started at {}</i>",
    cfg.vault_name,
    cfg.vault_address,
    &cfg.vault_address[..10],
    cfg.poll_interval.as_secs(),
    cfg.threshold,
    cfg.decimals,
    fmt::timestamp(),
  );
    send(client, tg, &msg).await;
}
