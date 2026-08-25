//! # Foundry
//!
//! Foundry is a modern, modular, decoupled Rust backend platform and framework.
//!
//! ## Quick Start
//!
//! ```no_run
//! use foundry::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let app = FoundryApp::builder()
//!         .build()
//!         .await?;
//!
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

pub mod app;
pub mod prelude;

pub use app::{FoundryApp, FoundryBuilder, FoundryConfig};
pub use foundry_core as core;

#[cfg(feature = "storage")]
pub use foundry_storage as storage;

#[cfg(feature = "auth")]
pub use foundry_auth as auth;

#[cfg(feature = "server")]
pub use foundry_engine as engine;

#[cfg(feature = "extension")]
pub use foundry_extension as extension;
