//! # vika-cli
//!
//! A production-grade Rust CLI tool that generates TypeScript typings, Zod schemas,
//! and Fetch-based API clients from Swagger/OpenAPI specifications.
//!
//! ## Example
//!
//! ```no_run
//! use vika_cli::Config;
//!
//! // Load configuration
//! let config = vika_cli::config::loader::load_config().unwrap();
//! ```

pub mod cache;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod formatter;
pub mod generator;
pub mod progress;
pub mod templates;

/// Main configuration structure for vika-cli.
///
/// This struct represents the `.vika.json` configuration file.
///
/// # Example
///
/// ```no_run
/// use vika_cli::Config;
///
/// let config = vika_cli::config::loader::load_config().unwrap();
/// println!("Root directory: {}", config.root_dir);
/// ```
pub use config::model::Config;

/// Result type alias for vika-cli operations.
pub type Result<T> = std::result::Result<T, VikaError>;

/// Main error type for vika-cli.
///
/// All errors in vika-cli are wrapped in this enum, which provides
/// structured error handling with context.
pub use error::VikaError;
