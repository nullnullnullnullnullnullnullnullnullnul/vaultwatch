# VaultWatch - ERC-4626 Deposit Monitor

[![ci](https://github.com/nullnullnullnullnullnullnullnullnullnul/vaultwatch/actions/workflows/ci.yml/badge.svg)](https://github.com/nullnullnullnullnullnullnullnullnullnul/vaultwatch/actions/workflows/ci.yml)
[![Rust 2021](https://img.shields.io/badge/Rust-2021-CE422B.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A lightweight Rust bot that monitors any ERC-4626 vault for deposit availability and sends a Telegram notification when capacity opens up.

## How It Works

1. Polls the configured vault contract at a configurable interval (default: 30s).
2. Calls `maxDeposit(address)` to get the current available deposit capacity.
3. Calls `totalAssets()` for informational context.
4. If `available >= DEPOSIT_THRESHOLD`, fires a Telegram alert and logs to console.
5. Alert cooldown: Telegram only fires once per above-threshold window. Resets when available drops below threshold.
6. Sends a startup notification to Telegram on launch so you know the bot is online.

## Configuration

All configuration is done via environment variables (`.env` file):

| Variable             | Required | Description                                    | Example                                      |
|----------------------|----------|------------------------------------------------|----------------------------------------------|
| `RPC_URL`            | yes      | Ethereum JSON-RPC endpoint                     | `https://ethereum.publicnode.com`             |
| `VAULT_ADDRESS`      | yes      | ERC-4626 vault contract address                | `0x99CD4Ec3f88A45940936F469E4bB72A2A701EEB9` |
| `VAULT_NAME`         | no       | Display name for logs (default: `ERC-4626 Vault`) | `stUSDS`                                  |
| `TOKEN_DECIMALS`     | no       | Token decimal precision (default: 18)          | `18`                                         |
| `DEPOSIT_THRESHOLD`  | no       | Min available to trigger alert (default: 2000) | `2000`                                       |
| `POLL_INTERVAL_SECS` | no       | Seconds between polls (default: 30)            | `30`                                         |
| `TELEGRAM_BOT_TOKEN` | no       | Telegram Bot API token                         | `123456:ABC-DEF...`                          |
| `TELEGRAM_CHAT_ID`   | no       | Target chat/channel ID                         | `-1001234567890`                             |

Telegram fields are optional. If both are set, the bot sends alerts; otherwise it runs in console-only mode.

## Project Structure

```text
src/
  main.rs        -- entry point, wiring only
  config.rs      -- Config struct, env loading, banner
  contract.rs    -- ERC-4626 ABI binding, connect, poll logic
  fmt.rs         -- token formatting, fill percentage, timestamp
  log.rs         -- colored console logging with prefixes
  telegram.rs    -- Telegram Bot API (startup + alert messages)
```

## Building and Running

```bash
cargo build --release
cargo run --release
```

## Log Prefixes

```text
[*] info      (cyan)
[!] warning   (yellow)
[+] positive  (green)
[-] error     (red)
[?] input     (yellow)
```

## Tech Stack

| Crate     | Role                                        |
|-----------|---------------------------------------------|
| `ethers`  | ERC-4626 contract calls via JSON-RPC        |
| `tokio`   | Single-threaded async runtime + timer       |
| `reqwest` | Reusable HTTP client for Telegram Bot API   |
| `serde`   | JSON serialization for Telegram payloads    |
| `dotenvy` | Load `.env` file at startup                 |
| `eyre`    | Error handling                              |
