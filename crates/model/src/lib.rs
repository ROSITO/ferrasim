//! Versioned simulation contracts and model-level validation.
//!
//! Validation does not import a mesh or establish that a model is solvable.

mod types;
mod validation;

pub use types::*;
pub use validation::{IssueCode, ValidationErrors, ValidationIssue};

/// Only schema version accepted by this implementation.
pub const SCHEMA_VERSION: &str = "0.1";
