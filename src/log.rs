//! Colored console logging with prefix indicators.
//!
//! | Prefix | Meaning  | Color  |
//! |--------|----------|--------|
//! | `[*]`  | info     | cyan   |
//! | `[!]`  | warning  | yellow |
//! | `[+]`  | positive | green  |
//! | `[-]`  | error    | red    |
//! | `[?]`  | input    | yellow |

#![allow(dead_code)]

use crate::fmt;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

/// Log an informational message prefixed with `[*]` in cyan.
pub fn info(msg: &str) {
    println!("{CYAN}[*]{RESET} [{}] {msg}", fmt::timestamp());
}

/// Log a warning message prefixed with `[!]` in yellow.
pub fn warn(msg: &str) {
    println!("{YELLOW}[!]{RESET} [{}] {msg}", fmt::timestamp());
}

/// Log a positive/success message prefixed with `[+]` in green.
pub fn positive(msg: &str) {
    println!("{GREEN}[+]{RESET} [{}] {msg}", fmt::timestamp());
}

/// Log an error message prefixed with `[-]` in red to stderr.
pub fn error(msg: &str) {
    eprintln!("{RED}[-]{RESET} [{}] {msg}", fmt::timestamp());
}

/// Print an input prompt prefixed with `[?]` in yellow (no newline).
pub fn input(msg: &str) {
    print!("{YELLOW}[?]{RESET} [{}] {msg}", fmt::timestamp());
}
