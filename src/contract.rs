//! ERC-4626 vault contract interaction.
//!
//! Provides the ABI binding, connection factory, and polling logic
//! for querying on-chain deposit availability.

use ethers::prelude::*;
use std::sync::Arc;

use crate::config::Config;
use crate::fmt;
use crate::log;
use crate::telegram;

abigen!(
    Erc4626Vault,
    r#"[
    function maxDeposit(address receiver) external view returns (uint256)
    function totalAssets() external view returns (uint256)
  ]"#
);

/// Create a provider and bind it to the vault contract.
///
/// # Errors
///
/// Returns an error if the RPC URL is malformed or the vault address
/// cannot be parsed as a valid Ethereum address.
pub fn connect(cfg: &Config) -> eyre::Result<Erc4626Vault<Provider<Http>>> {
    let provider = Provider::<Http>::try_from(cfg.rpc_url.as_str())?;
    let address: Address = cfg.vault_address.parse()?;
    Ok(Erc4626Vault::new(address, Arc::new(provider)))
}

/// Query the vault and log results.
///
/// Sends a Telegram alert when the threshold is first crossed.
/// `alerted` tracks whether a notification has already been sent
/// for the current above-threshold window, preventing spam.
pub async fn poll_once(
    contract: &Erc4626Vault<Provider<Http>>,
    cfg: &Config,
    http: &reqwest::Client,
    alerted: &mut bool,
) {
    let max_call = contract.max_deposit(Address::zero());
    let total_call = contract.total_assets();
    let result = tokio::try_join!(max_call.call(), total_call.call());
    let (available, total_assets) = match result {
        Ok(vals) => vals,
        Err(e) => {
            log::error(&format!("{e}"));
            return;
        }
    };
    let avail = fmt::tokens(available, cfg.decimals);
    let total = fmt::tokens(total_assets, cfg.decimals);
    log::info(&format!("available: {avail} | totalAssets: {total}"));
    let above = fmt::parse_f64(&avail) >= cfg.threshold;
    if !above {
        // Reset cooldown when available drops below threshold.
        *alerted = false;
        return;
    }
    log::positive(&format!(
        "DEPOSIT SLOT OPEN! {avail} available (>= {} threshold)",
        cfg.threshold
    ));
    if *alerted {
        return;
    }
    *alerted = true;
    if let Some(tg) = &cfg.telegram {
        let msg = build_alert_message(cfg, &avail, &total);
        telegram::send(http, tg, &msg).await;
    }
}

/// Build the styled HTML alert message sent to Telegram.
fn build_alert_message(cfg: &Config, avail: &str, total: &str) -> String {
    let fill_pct = fmt::fill_percentage(avail, total);
    format!(
    "\u{1F6A8} <b>VaultWatch Alert</b>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{1F3E6} <b>Vault:</b>  {}\n\
     \u{1F517} <a href=\"https://etherscan.io/address/{}\">{}</a>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{2705} <b>Available:</b>  {avail}\n\
     \u{1F4B0} <b>Total Assets:</b>  {total}\n\
     \u{1F4CA} <b>Fill:</b>  {fill_pct}\n\
     \u{1F3AF} <b>Threshold:</b>  {}\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{23F0} <i>{}</i>",
    cfg.vault_name,
    cfg.vault_address,
    &cfg.vault_address[..10],
    cfg.threshold,
    fmt::timestamp(),
  )
}
