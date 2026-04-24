//! Official Rust SDK for the [Cryptohopper](https://www.cryptohopper.com) API.
//!
//! # Quickstart
//!
//! ```no_run
//! use cryptohopper::Client;
//!
//! # async fn run() -> Result<(), cryptohopper::Error> {
//! let ch = Client::new(std::env::var("CRYPTOHOPPER_TOKEN").unwrap())?;
//!
//! let me = ch.user.get().await?;
//! println!("{}", me);
//!
//! let ticker = ch
//!     .exchange
//!     .ticker(&serde_json::json!({"exchange": "binance", "market": "BTC/USDT"}))
//!     .await?;
//! println!("{}", ticker["last"]);
//! # Ok(())
//! # }
//! ```
//!
//! # Full configuration
//!
//! ```no_run
//! use cryptohopper::Client;
//! use std::time::Duration;
//!
//! # fn run() -> Result<(), cryptohopper::Error> {
//! let ch = Client::builder()
//!     .api_key("ch_...")
//!     .app_key("your_client_id")
//!     .base_url("https://api-staging.cryptohopper.com/v1")
//!     .timeout(Duration::from_secs(60))
//!     .max_retries(5)
//!     .user_agent("my-bot/1.0")
//!     .build()?;
//! # let _ = ch;
//! # Ok(())
//! # }
//! ```

#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod error;
pub mod resources;

mod client;

pub use client::{Client, ClientBuilder, VERSION};
pub use error::{Error, ErrorCode};
pub use serde_json::Value;
