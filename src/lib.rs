//! Fossil - Unearth your technical debt
//!
//! A CLI tool that excavates buried technical debt from codebases by extracting
//! and tracking debt markers (TODO, FIXME, HACK, XXX, NOTE).
//!
//! # Features
//!
//! - Extract all tech debt comments from a codebase with context
//! - Calculate "debt age" using git blame
//! - Group and categorize debt by author, file, severity, and age
//! - Output reports in multiple formats (terminal, markdown, JSON)
//! - Language-agnostic (works with any codebase)
//!
//! # Example
//!
//! ```rust,no_run
//! use fossil::*;
//! use std::path::Path;
//!
//! // Load configuration
//! let config = config::load_config(None).unwrap();
//!
//! // Scan directory
//! let markers = scanner::scan_directory(Path::new("."), &config).unwrap();
//!
//! // Create report
//! let report = models::DebtReport::new(markers, Path::new(".").to_path_buf());
//! ```

pub mod cli;
pub mod config;
pub mod filters;
pub mod git;
pub mod models;
pub mod reporter;
pub mod scanner;

// Re-export commonly used types
pub use models::{Config, DebtMarker, DebtReport, GitBlameInfo};
