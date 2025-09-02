//! Core abstractions and shared types for the dox workspace.
//!
//! This crate provides the foundational traits, types, and utilities
//! shared across all dox providers and the CLI.

pub mod spreadsheet;
pub mod generate;
pub mod error;
pub mod logging;
pub mod utils;
pub mod i18n;

pub use spreadsheet::*;
pub use generate::*;
pub use error::*;
pub use logging::*;
pub use utils::*;
