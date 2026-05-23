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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_basic_18_decimals() {
        // 10 * 10^18, 18 decimals -> "10.00"
        let v = U256::exp10(18) * U256::from(10u64);
        assert_eq!(tokens(v, 18), "10.00");
    }

    #[test]
    fn tokens_with_thousands_separator() {
        // From the docstring example: 195_996_854.302 with 18 decimals.
        let v = U256::from_dec_str("195996854302000000000000000").unwrap();
        assert_eq!(tokens(v, 18), "195,996,854.30");
    }

    #[test]
    fn tokens_zero() {
        assert_eq!(tokens(U256::zero(), 18), "0.00");
    }

    #[test]
    fn tokens_six_decimals_usdc_style() {
        // 1,234.567890 USDC -> "1,234.56" (truncation, not rounding;
        // we lose the last 4 decimal places by design for the
        // human-readable column).
        let v = U256::from(1_234_567_890u64);
        assert_eq!(tokens(v, 6), "1,234.56");
    }

    #[test]
    fn add_thousands_short_strings() {
        assert_eq!(add_thousands(""), "");
        assert_eq!(add_thousands("1"), "1");
        assert_eq!(add_thousands("12"), "12");
        assert_eq!(add_thousands("123"), "123");
    }

    #[test]
    fn add_thousands_inserts_commas_every_three_from_the_right() {
        assert_eq!(add_thousands("1000"), "1,000");
        assert_eq!(add_thousands("1000000"), "1,000,000");
        assert_eq!(add_thousands("12345678"), "12,345,678");
        assert_eq!(add_thousands("123456789"), "123,456,789");
    }

    #[test]
    fn parse_f64_round_trips_human_strings() {
        assert_eq!(parse_f64("1,234.56"), 1234.56);
        assert_eq!(parse_f64("0"), 0.0);
        assert_eq!(parse_f64("0.00"), 0.0);
    }

    #[test]
    fn parse_f64_returns_zero_on_garbage() {
        // Documented behaviour, but worth pinning: callers cannot
        // distinguish "value was zero" from "parse failed".
        assert_eq!(parse_f64(""), 0.0);
        assert_eq!(parse_f64("not a number"), 0.0);
        // "NaN" parses as a valid f64 (f64::NAN), NOT as 0.0 - the
        // fallback only catches genuine parse failures. Worth pinning
        // so we notice if the upstream behaviour ever changes.
        assert!(parse_f64("NaN").is_nan());
    }

    #[test]
    fn fill_percentage_handles_zero_total() {
        assert_eq!(fill_percentage("0", "0"), "N/A");
        assert_eq!(fill_percentage("100", "0"), "N/A");
    }

    #[test]
    fn fill_percentage_half_full() {
        // current=50, free=50 -> fill = current / (current + free) = 50%.
        assert_eq!(fill_percentage("50", "50"), "50.0000%");
    }

    #[test]
    fn html_escape_passes_safe_strings_through() {
        assert_eq!(html_escape("plain ascii"), "plain ascii");
        assert_eq!(html_escape("USDC-vault-1"), "USDC-vault-1");
    }

    #[test]
    fn html_escape_handles_all_five_entities() {
        assert_eq!(
            html_escape("<a href=\"#\">A & B's 'test'</a>"),
            "&lt;a href=&quot;#&quot;&gt;A &amp; B&#39;s &#39;test&#39;&lt;/a&gt;",
        );
    }
}
