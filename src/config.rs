//! Application configuration loaded from environment variables.

use std::time::Duration;

use crate::log;
use crate::telegram::TelegramConfig;

/// All runtime settings for VaultWatch.
///
/// Constructed once at startup via [`Config::from_env`] and passed
/// by reference throughout the application.
pub struct Config {
    pub rpc_url: String,
    pub vault_address: String,
    pub vault_name: String,
    pub decimals: u32,
    pub poll_interval: Duration,
    pub threshold: f64,
    pub telegram: Option<TelegramConfig>,
}

impl Config {
    /// Build a [`Config`] from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if `RPC_URL` or `VAULT_ADDRESS` are missing (no sensible default).
    pub fn from_env() -> Self {
        Self {
            rpc_url: env_var("RPC_URL", None),
            vault_address: env_var("VAULT_ADDRESS", None),
            vault_name: env_var("VAULT_NAME", Some("ERC-4626 Vault")),
            decimals: env_var("TOKEN_DECIMALS", Some("18")).parse().unwrap(),
            poll_interval: Duration::from_secs(
                env_var("POLL_INTERVAL_SECS", Some("30")).parse().unwrap(),
            ),
            threshold: env_var("DEPOSIT_THRESHOLD", Some("2000")).parse().unwrap(),
            telegram: TelegramConfig::from_env(),
        }
    }

    /// Print a startup banner summarizing the active configuration.
    pub fn print_banner(&self) {
        let tg_status = if self.telegram.is_some() {
            "enabled"
        } else {
            "disabled"
        };
        println!("          VAULTWATCH");
        println!("\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}");
        log::info(&format!(
            "Vault     : {} ({})",
            self.vault_name, self.vault_address
        ));
        log::info(&format!("RPC       : {}", self.rpc_url));
        println!("\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}");
        log::info(&format!("Decimals  : {}", self.decimals));
        log::info(&format!("Poll      : {}s", self.poll_interval.as_secs()));
        log::info(&format!("Threshold : {}", self.threshold));
        log::info(&format!("Telegram  : {tg_status}"));
        println!("\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}");
        println!();
    }
}

/// Read an environment variable, falling back to `fallback` if unset.
///
/// # Panics
///
/// Panics when the variable is unset and no fallback is provided.
pub fn env_var(key: &str, fallback: Option<&str>) -> String {
    match (std::env::var(key), fallback) {
        (Ok(val), _) => val,
        (Err(_), Some(def)) => def.to_owned(),
        (Err(_), None) => panic!("{key} must be set in .env"),
    }
}
