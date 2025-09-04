//! Core abstractions and shared types for the dox workspace.
//!
//! This crate provides the foundational traits, types, and utilities
//! shared across all dox providers and the CLI.

pub mod create;
pub mod error;
pub mod generate;
pub mod i18n;
pub mod logging;
pub mod replace;
pub mod spreadsheet;
pub mod utils;

pub use create::*;
pub use error::*;
pub use generate::*;
pub use logging::*;
pub use spreadsheet::*;
pub use utils::*;
