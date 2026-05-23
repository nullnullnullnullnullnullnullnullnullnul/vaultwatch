//! ERC-4626 vault contract interaction.
//!
//! Provides the ABI binding, connection factory, and polling logic
//! for querying on-chain deposit availability.

use ethers::prelude::*;
use ethers::providers::Http;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::fmt;
use crate::log;
use crate::telegram;

/// Total RPC request budget. Picked at 10s because a healthy
/// public RPC answers a single eth_call in <500ms; anything past
/// 10s is the endpoint being unreachable, not slow.
const RPC_TIMEOUT: Duration = Duration::from_secs(10);

abigen!(
    Erc4626Vault,
    r#"[
    function maxDeposit(address receiver) external view returns (uint256)
    function totalAssets() external view returns (uint256)
  ]"#
);

/// Create a provider with a bounded request timeout and bind it to
/// the vault contract.
///
/// Important: `ethers::providers::Provider::<Http>::try_from(url)`
/// uses a default `reqwest::Client` with NO request timeout, so a
/// stuck RPC endpoint hangs the entire polling loop indefinitely.
/// We build our own client with [`RPC_TIMEOUT`] and inject it via
/// `Http::new_with_client`.
///
/// # Errors
///
/// Returns an error if the RPC URL is malformed, the vault address
/// is not a valid Ethereum address, or the underlying reqwest client
/// cannot be constructed.
pub fn connect(cfg: &Config) -> eyre::Result<Erc4626Vault<Provider<Http>>> {
    let url: reqwest::Url = cfg.rpc_url.parse()?;
    let client = reqwest::Client::builder().timeout(RPC_TIMEOUT).build()?;
    let provider = Provider::new(Http::new_with_client(url, client));
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
    // Hysteresis: alert when available crosses ABOVE the alert
    // threshold; reset the cooldown only when it falls BELOW a
    // strictly lower reset threshold. Without that gap, a vault
    // whose available oscillates by one wei around the threshold
    // would alert on every poll.
    if available < cfg.reset_threshold_atomic {
        *alerted = false;
        return;
    }
    if available < cfg.threshold_atomic {
        // In the hysteresis band: still cooling down from the last
        // alert (or never alerted in this window). Nothing to do.
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
///
/// `cfg.vault_name` and `cfg.vault_address` are operator-supplied,
/// so they get `html_escape`d before interpolation. The address
/// also flows into a `href="..."` attribute; escaping the quote
/// and ampersand is enough since the value is already constrained
/// to hex by the address parser at startup.
fn build_alert_message(cfg: &Config, avail: &str, total: &str) -> String {
    let fill_pct = fmt::fill_percentage(avail, total);
    let name = fmt::html_escape(&cfg.vault_name);
    let addr = fmt::html_escape(&cfg.vault_address);
    let addr_short = fmt::html_escape(&cfg.vault_address[..10]);
    format!(
    "\u{1F6A8} <b>VaultWatch Alert</b>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{1F3E6} <b>Vault:</b>  {name}\n\
     \u{1F517} <a href=\"https://etherscan.io/address/{addr}\">{addr_short}</a>\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{2705} <b>Available:</b>  {avail}\n\
     \u{1F4B0} <b>Total Assets:</b>  {total}\n\
     \u{1F4CA} <b>Fill:</b>  {fill_pct}\n\
     \u{1F3AF} <b>Threshold:</b>  {}\n\
     \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n\
     \u{23F0} <i>{}</i>",
    cfg.threshold,
    fmt::timestamp(),
  )
}
