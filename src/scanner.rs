use crate::models::{Config, DebtMarker};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

/// Scan a directory for technical debt markers
pub fn scan_directory(path: &Path, config: &Config) -> Result<Vec<DebtMarker>> {
    let mut markers = Vec::new();

    // Build regex pattern from config markers
    let pattern = build_marker_regex(&config.markers)?;

    // Build the file walker
    let mut walker = WalkBuilder::new(path);
    walker.standard_filters(true); // Respect .gitignore

    // Add ignored directories from config
    let ignored_dirs = config.ignored_dirs.clone();
    walker.filter_entry(move |entry| {
        let name = entry.file_name().to_str().unwrap_or("");
        !ignored_dirs.iter().any(|ignored| ignored == name)
    });

    // Walk the directory tree
    for result in walker.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue, // Skip files we can't read
        };

        // Skip directories
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        // Skip if file is too large
        if let Ok(metadata) = entry.metadata() {
            if metadata.len() > MAX_FILE_SIZE {
                continue;
            }
        }

        // Scan the file for markers
        if let Ok(file_markers) = scan_file(entry.path(), &pattern, config.context_lines) {
            markers.extend(file_markers);
        }
    }

    Ok(markers)
}

/// Build regex pattern to match debt markers in comments
fn build_marker_regex(markers: &[String]) -> Result<Regex> {
    let markers_pattern = markers.join("|");

    // Match common comment styles with the markers
    // Handles: //, #, /*, *, <!--
    let pattern = format!(
        r"^\s*(?://|#|/\*|\*|<!--)\s*({})(?::|\s)?\s*(.*?)(?:-->|\*/)?$",
        markers_pattern
    );

    Regex::new(&pattern).context("Failed to compile marker regex")
}

/// Scan a single file for debt markers
fn scan_file(path: &Path, pattern: &Regex, context_lines: usize) -> Result<Vec<DebtMarker>> {
    let file = File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut markers = Vec::new();
    let mut line_buffer: VecDeque<(usize, String)> = VecDeque::new();
    let mut lines_after_marker: Option<(DebtMarker, usize)> = None;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue, // Skip lines we can't read (might be binary)
        };

        let line_number = line_num + 1; // Convert to 1-indexed

        // If we're collecting context after a marker
        if let Some((mut marker, remaining)) = lines_after_marker.take() {
            if remaining > 0 {
                marker.context_after.push(line.clone());
                lines_after_marker = Some((marker, remaining - 1));
            } else {
                markers.push(marker);
            }
        }

        // Maintain a rolling buffer of previous lines for context
        line_buffer.push_back((line_number, line.clone()));
        if line_buffer.len() > context_lines {
            line_buffer.pop_front();
        }

        // Check if this line contains a marker
        if let Some(captures) = pattern.captures(&line) {
            let marker_type = captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();

            // Extract context before (from buffer)
            let context_before: Vec<String> = line_buffer
                .iter()
                .take(line_buffer.len().saturating_sub(1)) // Exclude current line
                .map(|(_, l)| l.clone())
                .collect();

            let marker = DebtMarker {
                marker_type,
                file_path: path.to_path_buf(),
                line_number,
                line_content: line.clone(),
                context_before,
                context_after: Vec::new(),
                git_info: None, // Will be filled in by git module
            };

            // Start collecting context after
            if context_lines > 0 {
                lines_after_marker = Some((marker, context_lines));
            } else {
                markers.push(marker);
            }

            // Clear buffer to avoid including marker line in next context
            line_buffer.clear();
        }
    }

    // Don't forget the last marker if we were still collecting context
    if let Some((marker, _)) = lines_after_marker {
        markers.push(marker);
    }

    Ok(markers)
}

/// Check if a file is likely binary
pub fn is_likely_binary(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "ico" | "pdf" | "zip" | "tar" | "gz" | "exe" | "dll" | "so" | "dylib" | "bin" | "dat"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_build_marker_regex() {
        let markers = vec!["TODO".to_string(), "FIXME".to_string()];
        let regex = build_marker_regex(&markers).unwrap();

        assert!(regex.is_match("// TODO: fix this"));
        assert!(regex.is_match("# FIXME: broken"));
        assert!(regex.is_match("/* TODO something */"));
        assert!(regex.is_match("  * FIXME: stuff"));
        assert!(!regex.is_match("This is a TODO in prose"));
    }

    #[test]
    fn test_scan_file_with_markers() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let content = r#"
fn main() {
    // TODO: implement this
    println!("Hello");
    // FIXME: broken logic
}
"#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let markers = vec!["TODO".to_string(), "FIXME".to_string()];
        let pattern = build_marker_regex(&markers).unwrap();
        let found = scan_file(&file_path, &pattern, 1).unwrap();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].marker_type, "TODO");
        assert_eq!(found[0].line_number, 3);
        assert_eq!(found[1].marker_type, "FIXME");
        assert_eq!(found[1].line_number, 5);
    }

    #[test]
    fn test_context_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let content = r#"line 1
line 2
// TODO: fix
line 4
line 5"#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let markers = vec!["TODO".to_string()];
        let pattern = build_marker_regex(&markers).unwrap();
        let found = scan_file(&file_path, &pattern, 2).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].context_before.len(), 2);
        assert_eq!(found[0].context_after.len(), 2);
        assert_eq!(found[0].context_before[0], "line 1");
        assert_eq!(found[0].context_before[1], "line 2");
        assert_eq!(found[0].context_after[0], "line 4");
        assert_eq!(found[0].context_after[1], "line 5");
    }

    #[test]
    fn test_is_likely_binary() {
        assert!(is_likely_binary(Path::new("image.png")));
        assert!(is_likely_binary(Path::new("document.pdf")));
        assert!(!is_likely_binary(Path::new("code.rs")));
        assert!(!is_likely_binary(Path::new("script.py")));
    }
}
