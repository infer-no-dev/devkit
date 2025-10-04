//! Blueprint Evolution Test Suite
//!
//! Comprehensive testing framework for blueprint evolution functionality
//! including versioning, diffing, migration, and CLI operations.

pub mod fixtures;
pub mod test_utils;
pub mod version_tests;
// pub mod evolution_tests; // Temporarily disabled
pub mod diff_tests;
// TODO: Create these test modules
// pub mod migration_tests;
// pub mod cli_tests;
// pub mod integration_tests;
// pub mod performance_tests;

// Re-export commonly used testing utilities
pub use fixtures::*;
pub use test_utils::*;
