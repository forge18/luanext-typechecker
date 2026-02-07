//! Type checker state management
//!
//! This module provides the shared state structure for the type checker,
//! enabling better modularity and testability.

pub mod metrics;
pub mod stdlib_loader;
pub mod type_checker_state;

#[cfg(test)]
mod metrics_tests;

pub use metrics::{MetricSummary, Metrics};
pub use type_checker_state::TypeCheckerState;
