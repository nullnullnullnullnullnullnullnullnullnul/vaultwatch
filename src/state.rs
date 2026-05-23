//! On-disk persistence for the cooldown flag.
//!
//! Without this, an unexpected restart (crash, redeploy, host
//! reboot) resets the in-memory `alerted` flag to false. The next
//! poll then re-alerts immediately if the slot is still open,
//! which spams the chat for what is the same event.
//!
//! The state is a single boolean serialised to a tiny JSON file.
//! JSON over `bool::to_string()` is deliberately overkill: when we
//! grow the state (last_alert_at for cooldown windows, a per-vault
//! history of seen values, etc.), the schema is forward-compatible
//! and serde handles the migration without bespoke parsing.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::log;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    /// Whether an alert has already been emitted for the current
    /// above-threshold window. Matches the in-memory flag in
    /// `contract::poll_once`.
    pub alerted: bool,
}

/// Load state from `path`. On any failure (missing file,
/// permission error, malformed JSON) returns the default state
/// and logs a warning. We never fail the startup over a state
/// file: the worst that happens is one spurious alert after a
/// reset.
pub fn load(path: &Path) -> State {
    match fs::read_to_string(path) {
        Ok(s) => match serde_json::from_str::<State>(&s) {
            Ok(state) => {
                log::info(&format!(
                    "state loaded from {} (alerted={})",
                    path.display(),
                    state.alerted,
                ));
                state
            }
            Err(e) => {
                log::warn(&format!(
                    "state file {} unparseable ({e}), starting fresh",
                    path.display(),
                ));
                State::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
        Err(e) => {
            log::warn(&format!(
                "state file {} unreadable ({e}), starting fresh",
                path.display(),
            ));
            State::default()
        }
    }
}

/// Atomically write `state` to `path`: write to a sibling temp
/// file, fsync, then rename over the target. The rename is atomic
/// on POSIX (and on NTFS) for files on the same filesystem, so a
/// crash mid-write leaves either the old or the new content -
/// never a truncated mix.
pub fn save(path: &Path, state: &State) -> std::io::Result<()> {
    let tmp = tmp_sibling(path);
    let bytes = serde_json::to_vec(state).map_err(std::io::Error::other)?;
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

fn tmp_sibling(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}
