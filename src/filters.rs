use crate::models::DebtMarker;
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

/// Filter markers by minimum age
pub fn filter_by_age(markers: Vec<DebtMarker>, min_age_str: &str) -> Result<Vec<DebtMarker>> {
    let min_age_duration = parse_duration(min_age_str)
        .with_context(|| format!("Invalid age format: {}", min_age_str))?;

    let min_age_days = min_age_duration.as_secs() / (24 * 60 * 60);

    Ok(markers
        .into_iter()
        .filter(|marker| {
            if let Some(ref git_info) = marker.git_info {
                git_info.age_days >= min_age_days as i64
            } else {
                false // Exclude markers without git info
            }
        })
        .collect())
}

/// Filter markers by author (case-insensitive partial match)
pub fn filter_by_author(markers: Vec<DebtMarker>, author: &str) -> Vec<DebtMarker> {
    let author_lower = author.to_lowercase();

    markers
        .into_iter()
        .filter(|marker| {
            if let Some(ref git_info) = marker.git_info {
                git_info.author.to_lowercase().contains(&author_lower)
                    || git_info.author_email.to_lowercase().contains(&author_lower)
            } else {
                false
            }
        })
        .collect()
}

/// Filter markers by type (case-insensitive exact match)
pub fn filter_by_type(markers: Vec<DebtMarker>, marker_type: &str) -> Vec<DebtMarker> {
    let marker_type_lower = marker_type.to_lowercase();

    markers
        .into_iter()
        .filter(|marker| marker.marker_type.to_lowercase() == marker_type_lower)
        .collect()
}

/// Parse duration string like "30d", "6m", "1y" into Duration
fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    let s = s.trim();
    let (num_str, unit) = s.split_at(s.len() - 1);

    let num: u64 = num_str
        .parse()
        .with_context(|| format!("Invalid number in duration: {}", num_str))?;

    let days = match unit {
        "d" => num,
        "w" => num * 7,
        "m" => num * 30,
        "y" => num * 365,
        _ => return Err(anyhow!("Invalid duration unit: {}. Use d, w, m, or y", unit)),
    };

    Ok(Duration::from_secs(days * 24 * 60 * 60))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GitBlameInfo;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_marker(marker_type: &str, age_days: i64, author: &str) -> DebtMarker {
        DebtMarker {
            marker_type: marker_type.to_string(),
            file_path: PathBuf::from("test.rs"),
            line_number: 1,
            line_content: format!("// {}: test", marker_type),
            context_before: vec![],
            context_after: vec![],
            git_info: Some(GitBlameInfo {
                author: author.to_string(),
                author_email: format!("{}@example.com", author.to_lowercase()),
                commit_hash: "abc123".to_string(),
                commit_time: Utc::now(),
                age_days,
            }),
        }
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1d").unwrap().as_secs(), 86400);
        assert_eq!(parse_duration("2w").unwrap().as_secs(), 14 * 86400);
        assert_eq!(parse_duration("3m").unwrap().as_secs(), 90 * 86400);
        assert_eq!(parse_duration("1y").unwrap().as_secs(), 365 * 86400);

        assert!(parse_duration("").is_err());
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("10x").is_err());
    }

    #[test]
    fn test_filter_by_age() {
        let markers = vec![
            create_test_marker("TODO", 10, "Alice"),
            create_test_marker("TODO", 50, "Bob"),
            create_test_marker("TODO", 100, "Charlie"),
        ];

        let filtered = filter_by_age(markers, "30d").unwrap();
        assert_eq!(filtered.len(), 2); // 50 and 100 days old

        let filtered2 = filter_by_age(
            vec![
                create_test_marker("TODO", 10, "Alice"),
                create_test_marker("TODO", 50, "Bob"),
            ],
            "60d",
        )
        .unwrap();
        assert_eq!(filtered2.len(), 0); // None old enough
    }

    #[test]
    fn test_filter_by_author() {
        let markers = vec![
            create_test_marker("TODO", 10, "Alice"),
            create_test_marker("TODO", 20, "Bob"),
            create_test_marker("TODO", 30, "Charlie"),
        ];

        let filtered = filter_by_author(markers, "alice");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].git_info.as_ref().unwrap().author, "Alice");

        let markers2 = vec![
            create_test_marker("TODO", 10, "Alice"),
            create_test_marker("TODO", 20, "Bob"),
        ];

        let filtered2 = filter_by_author(markers2, "ob");
        assert_eq!(filtered2.len(), 1);
        assert_eq!(filtered2[0].git_info.as_ref().unwrap().author, "Bob");
    }

    #[test]
    fn test_filter_by_type() {
        let markers = vec![
            create_test_marker("TODO", 10, "Alice"),
            create_test_marker("FIXME", 20, "Bob"),
            create_test_marker("TODO", 30, "Charlie"),
        ];

        let filtered = filter_by_type(markers, "todo");
        assert_eq!(filtered.len(), 2);

        let markers2 = vec![
            create_test_marker("TODO", 10, "Alice"),
            create_test_marker("FIXME", 20, "Bob"),
        ];

        let filtered2 = filter_by_type(markers2, "FIXME");
        assert_eq!(filtered2.len(), 1);
        assert_eq!(filtered2[0].marker_type, "FIXME");
    }

    #[test]
    fn test_filter_without_git_info() {
        let mut marker = create_test_marker("TODO", 100, "Alice");
        marker.git_info = None;

        let markers = vec![marker];
        let filtered = filter_by_age(markers.clone(), "30d").unwrap();
        assert_eq!(filtered.len(), 0); // No git info, filtered out

        let filtered2 = filter_by_author(markers, "alice");
        assert_eq!(filtered2.len(), 0); // No git info, filtered out
    }
}
