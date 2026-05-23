//! Colored prefix logging built on top of `tracing`.
//!
//! The thin wrapper functions (`info`, `warn`, `positive`, `error`)
//! preserve the project's existing call sites and the styled
//! prefix output:
//!
//! | Prefix | Meaning  | Color  | tracing level                    |
//! |--------|----------|--------|----------------------------------|
//! | `[*]`  | info     | cyan   | INFO                             |
//! | `[!]`  | warning  | yellow | WARN                             |
//! | `[+]`  | positive | green  | INFO (target `vaultwatch::positive`) |
//! | `[-]`  | error    | red    | ERROR                            |
//!
//! Verbosity is configurable via `RUST_LOG`:
//!
//! ```text
//! RUST_LOG=vaultwatch=warn cargo run   # silence the per-poll info line
//! RUST_LOG=vaultwatch=error cargo run  # only errors
//! ```
//!
//! The default (no `RUST_LOG`) is `info`, matching the pre-tracing
//! behaviour.

#![allow(dead_code)]

use std::fmt as stdfmt;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;

use crate::fmt;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

/// Target string for `positive` events so the custom formatter
/// can distinguish them from regular INFO.
const POSITIVE_TARGET: &str = "vaultwatch::positive";

/// Install the global tracing subscriber. Call exactly once at
/// startup, after `dotenvy::dotenv()` so `RUST_LOG` from `.env`
/// is picked up.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .event_format(PrefixFormatter)
        .with_env_filter(filter)
        .init();
}

/// Event formatter that emits `<colored-prefix> [HH:MM:SS UTC] <msg>`
/// instead of tracing's default `<ts> <LEVEL> <target>: <msg>`.
struct PrefixFormatter;

impl<S, N> FormatEvent<S, N> for PrefixFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> stdfmt::Result {
        let meta = event.metadata();
        let prefix = if meta.target() == POSITIVE_TARGET {
            // Treated as INFO by the env filter, but rendered as [+].
            format!("{GREEN}[+]{RESET}")
        } else {
            match *meta.level() {
                Level::ERROR => format!("{RED}[-]{RESET}"),
                Level::WARN => format!("{YELLOW}[!]{RESET}"),
                Level::INFO => format!("{CYAN}[*]{RESET}"),
                // DEBUG / TRACE: same styling as INFO but the env
                // filter has to opt in (RUST_LOG=vaultwatch=debug),
                // so seeing these on stdout is intentional.
                _ => format!("{CYAN}[*]{RESET}"),
            }
        };
        write!(writer, "{prefix} [{}] ", fmt::timestamp())?;
        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

/// Log an informational message prefixed with `[*]` in cyan.
pub fn info(msg: &str) {
    tracing::info!("{msg}");
}

/// Log a warning message prefixed with `[!]` in yellow.
pub fn warn(msg: &str) {
    tracing::warn!("{msg}");
}

/// Log a positive/success message prefixed with `[+]` in green.
///
/// Emitted at INFO severity but with a distinct target so the
/// formatter can pick the green prefix. The env filter treats it
/// as plain INFO, so `RUST_LOG=vaultwatch=warn` silences it
/// alongside ordinary info lines.
pub fn positive(msg: &str) {
    tracing::info!(target: POSITIVE_TARGET, "{msg}");
}

/// Log an error message prefixed with `[-]` in red.
pub fn error(msg: &str) {
    tracing::error!("{msg}");
}

/// Print an input prompt prefixed with `[?]` in yellow (no newline).
///
/// Kept for compatibility; currently unused. Bypasses tracing
/// because it intentionally does NOT terminate the line - it
/// expects a follow-up `read_line`.
pub fn input(msg: &str) {
    print!("{YELLOW}[?]{RESET} [{}] {msg}", fmt::timestamp());
}
