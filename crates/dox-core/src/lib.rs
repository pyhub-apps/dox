//! Core abstractions and shared types for the dox workspace.
//!
//! Note: In this initial scaffolding stage, existing code remains in `dox-cli`.
//! Modules will be migrated here incrementally.

// Placeholder for future spreadsheet traits and shared types.
pub mod spreadsheet {
    /// Placeholder Range type using A1 notation.
    #[derive(Debug, Clone)]
    pub struct RangeRef(pub String);

    /// Placeholder cell type.
    #[derive(Debug, Clone)]
    pub struct Cell {
        pub value: String,
    }
}
