use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a single technical debt marker found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtMarker {
    /// Type of marker (TODO, FIXME, HACK, XXX, NOTE, etc.)
    pub marker_type: String,

    /// Path to the file containing the marker
    pub file_path: PathBuf,

    /// Line number where the marker was found (1-indexed)
    pub line_number: usize,

    /// The actual line content containing the marker
    pub line_content: String,

    /// Lines of code before the marker for context
    pub context_before: Vec<String>,

    /// Lines of code after the marker for context
    pub context_after: Vec<String>,

    /// Git blame information if available
    pub git_info: Option<GitBlameInfo>,
}

/// Git blame information for a debt marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlameInfo {
    /// Author name from git
    pub author: String,

    /// Author email from git
    pub author_email: String,

    /// Commit hash (abbreviated)
    pub commit_hash: String,

    /// When the line was committed
    pub commit_time: DateTime<Utc>,

    /// Age in days since commit
    pub age_days: i64,
}

impl GitBlameInfo {
    /// Format age as human-readable string (e.g., "347d", "2m", "1y")
    pub fn age_display(&self) -> String {
        if self.age_days < 30 {
            format!("{}d", self.age_days)
        } else if self.age_days < 365 {
            format!("{}m", self.age_days / 30)
        } else {
            format!("{}y", self.age_days / 365)
        }
    }
}

/// Complete report of technical debt found in a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtReport {
    /// All markers found
    pub markers: Vec<DebtMarker>,

    /// Total number of markers
    pub total_count: usize,

    /// Count of markers by type
    pub by_type: HashMap<String, usize>,

    /// Count of markers by author
    pub by_author: HashMap<String, usize>,

    /// Count of markers by file
    pub by_file: HashMap<PathBuf, usize>,

    /// Path that was scanned
    pub scan_path: PathBuf,

    /// When the scan was performed
    pub scan_time: DateTime<Utc>,
}

impl DebtReport {
    /// Create a new report from a collection of markers
    pub fn new(markers: Vec<DebtMarker>, scan_path: PathBuf) -> Self {
        let total_count = markers.len();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_author: HashMap<String, usize> = HashMap::new();
        let mut by_file: HashMap<PathBuf, usize> = HashMap::new();

        for marker in &markers {
            *by_type.entry(marker.marker_type.clone()).or_insert(0) += 1;
            *by_file.entry(marker.file_path.clone()).or_insert(0) += 1;

            if let Some(ref git_info) = marker.git_info {
                *by_author.entry(git_info.author.clone()).or_insert(0) += 1;
            }
        }

        Self {
            markers,
            total_count,
            by_type,
            by_author,
            by_file,
            scan_path,
            scan_time: Utc::now(),
        }
    }

    /// Get markers sorted by age (oldest first)
    pub fn oldest_markers(&self, limit: usize) -> Vec<&DebtMarker> {
        let mut markers_with_age: Vec<&DebtMarker> = self
            .markers
            .iter()
            .filter(|m| m.git_info.is_some())
            .collect();

        markers_with_age.sort_by(|a, b| {
            let age_a = a.git_info.as_ref().unwrap().age_days;
            let age_b = b.git_info.as_ref().unwrap().age_days;
            age_b.cmp(&age_a) // Descending order (oldest first)
        });

        markers_with_age.into_iter().take(limit).collect()
    }
}

/// Configuration for the fossil scanner
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Debt markers to search for
    #[serde(default = "default_markers")]
    pub markers: Vec<String>,

    /// Directories to ignore during scanning
    #[serde(default = "default_ignored_dirs")]
    pub ignored_dirs: Vec<String>,

    /// Number of context lines to capture before/after marker
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,

    /// Optional severity mapping for markers
    #[serde(default)]
    pub severity: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            markers: default_markers(),
            ignored_dirs: default_ignored_dirs(),
            context_lines: default_context_lines(),
            severity: HashMap::new(),
        }
    }
}

fn default_markers() -> Vec<String> {
    vec![
        "TODO".to_string(),
        "FIXME".to_string(),
        "HACK".to_string(),
        "XXX".to_string(),
        "NOTE".to_string(),
    ]
}

fn default_ignored_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "vendor".to_string(),
        ".next".to_string(),
        "__pycache__".to_string(),
        ".pytest_cache".to_string(),
        "coverage".to_string(),
    ]
}

fn default_context_lines() -> usize {
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_display() {
        let info = GitBlameInfo {
            author: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            commit_hash: "abc123".to_string(),
            commit_time: Utc::now(),
            age_days: 15,
        };
        assert_eq!(info.age_display(), "15d");

        let info2 = GitBlameInfo {
            age_days: 60,
            ..info.clone()
        };
        assert_eq!(info2.age_display(), "2m");

        let info3 = GitBlameInfo {
            age_days: 400,
            ..info
        };
        assert_eq!(info3.age_display(), "1y");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.markers.contains(&"TODO".to_string()));
        assert!(config.markers.contains(&"FIXME".to_string()));
        assert_eq!(config.context_lines, 2);
        assert!(config.ignored_dirs.contains(&".git".to_string()));
    }

    #[test]
    fn test_debt_report_creation() {
        let markers = vec![
            DebtMarker {
                marker_type: "TODO".to_string(),
                file_path: PathBuf::from("test.rs"),
                line_number: 1,
                line_content: "// TODO: test".to_string(),
                context_before: vec![],
                context_after: vec![],
                git_info: None,
            },
            DebtMarker {
                marker_type: "TODO".to_string(),
                file_path: PathBuf::from("test.rs"),
                line_number: 2,
                line_content: "// TODO: test2".to_string(),
                context_before: vec![],
                context_after: vec![],
                git_info: None,
            },
        ];

        let report = DebtReport::new(markers, PathBuf::from("."));
        assert_eq!(report.total_count, 2);
        assert_eq!(*report.by_type.get("TODO").unwrap(), 2);
        assert_eq!(*report.by_file.get(&PathBuf::from("test.rs")).unwrap(), 2);
    }
}
