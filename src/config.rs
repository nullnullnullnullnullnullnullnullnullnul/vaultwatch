//! Application configuration loaded from environment variables.

use std::time::Duration;

use ethers::types::U256;

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
    /// Whole-token threshold as displayed (e.g. 2000 means 2000 USDC
    /// regardless of `decimals`). Kept around for the banner and the
    /// alert message; runtime comparisons use [`Self::threshold_atomic`].
    pub threshold: u64,
    /// `threshold` expressed in the token's atomic units
    /// (`threshold * 10^decimals`). This is the value compared
    /// against the U256 returned by `maxDeposit()` so the
    /// comparison happens at full on-chain precision without any
    /// f64 / string round-trip.
    pub threshold_atomic: U256,
    /// Hysteresis floor: the `alerted` flag is reset only when
    /// `available` drops below this value, NOT just below
    /// `threshold_atomic`. Without a gap between the two, a vault
    /// whose available capacity oscillates by even one wei around
    /// the threshold would alert every poll: alert, dip, reset,
    /// alert again. Default reset is 95% of the alert threshold;
    /// see [`HYSTERESIS_NUMERATOR`] / [`HYSTERESIS_DENOMINATOR`].
    pub reset_threshold_atomic: U256,
    pub telegram: Option<TelegramConfig>,
}

/// Numerator of the hysteresis ratio. With denominator 100, this
/// produces a 95% reset threshold: alert at T, reset only when
/// `available < 0.95 * T`. Hardcoded for now; if the project ever
/// needs per-deployment tuning, lift to an env var.
const HYSTERESIS_NUMERATOR: u32 = 95;
const HYSTERESIS_DENOMINATOR: u32 = 100;

impl Config {
    /// Build a [`Config`] from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if `RPC_URL` or `VAULT_ADDRESS` are missing (no sensible default).
    pub fn from_env() -> Self {
        let decimals: u32 = env_var("TOKEN_DECIMALS", Some("18")).parse().unwrap();
        let threshold: u64 = env_var("DEPOSIT_THRESHOLD", Some("2000")).parse().unwrap();
        let threshold_atomic = U256::from(threshold) * U256::exp10(decimals as usize);
        let reset_threshold_atomic =
            threshold_atomic * U256::from(HYSTERESIS_NUMERATOR) / U256::from(HYSTERESIS_DENOMINATOR);
        Self {
            rpc_url: env_var("RPC_URL", None),
            vault_address: env_var("VAULT_ADDRESS", None),
            vault_name: env_var("VAULT_NAME", Some("ERC-4626 Vault")),
            decimals,
            poll_interval: Duration::from_secs(
                env_var("POLL_INTERVAL_SECS", Some("30")).parse().unwrap(),
            ),
            threshold,
            threshold_atomic,
            reset_threshold_atomic,
            telegram: TelegramConfig::from_env(),
        }
    }

    /// Return the RPC URL stripped of path, query, and credentials,
    /// leaving only `scheme://host[:port]`. Hosted RPC providers
    /// (Alchemy, Infura, QuickNode) embed the API key in the path
    /// (`/v2/<KEY>`) or the userinfo segment; logging the full URL
    /// to stdout (banners, journald, hosting consoles) leaks it.
    /// Falls back to `<unparseable>` on a malformed URL, which is
    /// fine - the connection will fail loudly in connect() anyway.
    pub fn rpc_url_safe(&self) -> String {
        match reqwest::Url::parse(&self.rpc_url) {
            Ok(u) => match u.host_str() {
                Some(host) => match u.port() {
                    Some(p) => format!("{}://{host}:{p}", u.scheme()),
                    None => format!("{}://{host}", u.scheme()),
                },
                None => "<no host>".to_owned(),
            },
            Err(_) => "<unparseable>".to_owned(),
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
        log::info(&format!("RPC       : {}", self.rpc_url_safe()));
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
