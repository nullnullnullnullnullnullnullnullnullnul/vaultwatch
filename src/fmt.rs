//! Formatting utilities for token amounts and timestamps.

use ethers::prelude::U256;
use ethers::utils::format_units;

/// Format a [`U256`] token value as a human-readable string with
/// thousand separators and two decimal places.
///
/// # Examples
///
/// `195996854302000000000000000` with 18 decimals becomes `"195,996,854.30"`.
pub fn tokens(value: U256, decimals: u32) -> String {
    let raw = format_units(value, decimals).unwrap_or_else(|_| "0".into());
    let (whole, frac) = match raw.find('.') {
        Some(dot) => {
            let end = std::cmp::min(dot + 3, raw.len());
            (&raw[..dot], &raw[dot..end])
        }
        None => (raw.as_str(), ""),
    };
    let with_commas = add_thousands(whole);
    format!("{with_commas}{frac}")
}

/// Insert comma separators every three digits from the right.
fn add_thousands(s: &str) -> String {
    let bytes: Vec<u8> = s.bytes().collect();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        let remaining = bytes.len() - i;
        if i > 0 && remaining.is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// Compute the vault fill percentage from formatted available and
/// total asset strings.
///
/// Returns a string like `"99.9984%"` or `"N/A"` when total is zero.
pub fn fill_percentage(avail: &str, total: &str) -> String {
    let t = parse_f64(total);
    let a = parse_f64(avail);
    if t <= 0.0 {
        return "N/A".to_owned();
    }
    format!("{:.4}%", (t / (t + a)) * 100.0)
}

/// Parse a formatted token string (may contain commas) into an `f64`.
///
/// Returns `0.0` on any parse failure.
pub fn parse_f64(s: &str) -> f64 {
    s.replace(',', "").parse().unwrap_or(0.0)
}

/// Escape a string for inclusion in a Telegram HTML message.
///
/// Telegram's HTML parse_mode honours the same five entities the
/// HTML spec defines: `& < > " '`. Any of these in operator-
/// supplied text (e.g. `VAULT_NAME="A&B <test>"`) breaks the
/// parser visually and, in the worst case for a hostile-input
/// scenario, lets an attacker inject anchor or pre tags that the
/// chat will render. Escape preemptively.
///
/// Hand-rolled (5 substitutions, ~10 lines) rather than pulling
/// in `html_escape` as a new dep for one call site.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Current wall-clock time as `HH:MM:SS UTC`.
///
/// Uses [`std::time::SystemTime`] to avoid pulling in the `chrono` crate.
pub fn timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!(
        "{:02}:{:02}:{:02} UTC",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60,
    )
}
