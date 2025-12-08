use crate::models::{Config, DebtMarker};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

/// Scan a directory for technical debt markers
pub fn scan_directory(path: &Path, config: &Config) -> Result<Vec<DebtMarker>> {
    // Build regex pattern from config markers
    let pattern = Arc::new(build_marker_regex(&config.markers)?);
    let context_lines = config.context_lines;

    // Thread-safe vector to collect markers
    let markers = Arc::new(Mutex::new(Vec::new()));

    // Build the file walker
    let mut walker = WalkBuilder::new(path);
    walker.standard_filters(true); // Respect .gitignore

    // Add ignored directories from config
    let ignored_dirs = config.ignored_dirs.clone();
    walker.filter_entry(move |entry| {
        let name = entry.file_name().to_str().unwrap_or("");
        !ignored_dirs.iter().any(|ignored| ignored == name)
    });

    // Walk the directory tree in parallel
    walker.build_parallel().run(|| {
        let pattern = Arc::clone(&pattern);
        let markers = Arc::clone(&markers);

        Box::new(move |result| {
            use ignore::WalkState;

            let entry = match result {
                Ok(entry) => entry,
                Err(_) => return WalkState::Continue,
            };

            // Skip directories
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                return WalkState::Continue;
            }

            // Skip if file is too large
            if let Ok(metadata) = entry.metadata() {
                if metadata.len() > MAX_FILE_SIZE {
                    return WalkState::Continue;
                }
            }

            // Scan the file for markers
            if let Ok(file_markers) = scan_file(entry.path(), &pattern, context_lines) {
                if !file_markers.is_empty() {
                    if let Ok(mut markers) = markers.lock() {
                        markers.extend(file_markers);
                    }
                }
            }

            WalkState::Continue
        })
    });

    // Extract the markers from the Arc<Mutex<>>
    let markers = Arc::try_unwrap(markers)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap markers"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to extract markers"))?;

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
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    let mut markers = Vec::new();
    let mut line_buffer: VecDeque<String> = VecDeque::new();
    let mut lines_after_marker: Option<(DebtMarker, usize)> = None;

    // Reusable byte buffer
    let mut byte_buffer = Vec::with_capacity(256);
    let mut line_number = 0;

    loop {
        byte_buffer.clear();
        match reader.read_until(b'\n', &mut byte_buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break, // I/O error, stop reading this file
        };

        line_number += 1;

        // Try to convert to UTF-8 string (skip line if invalid)
        let line = match std::str::from_utf8(&byte_buffer) {
            Ok(s) => s.trim_end_matches('\n').trim_end_matches('\r'),
            Err(_) => continue, // Skip non-UTF-8 lines (binary data)
        };

        // If we're collecting context after a marker
        if let Some((mut marker, remaining)) = lines_after_marker.take() {
            if remaining > 0 {
                marker.context_after.push(line.to_string());
                lines_after_marker = Some((marker, remaining - 1));
            } else {
                markers.push(marker);
            }
        }

        // Check if this line contains a marker
        if let Some(captures) = pattern.captures(line) {
            let marker_type = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            // Extract context before (from buffer)
            let context_before: Vec<String> = line_buffer.iter().cloned().collect();

            let marker = DebtMarker {
                marker_type,
                file_path: path.to_path_buf(),
                line_number,
                line_content: line.to_string(),
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
        } else {
            // Only add to buffer if this wasn't a marker line
            // Only allocate String when we need to save for context
            if context_lines > 0 {
                line_buffer.push_back(line.to_string());
                if line_buffer.len() > context_lines {
                    line_buffer.pop_front();
                }
            }
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
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "ico"
                | "pdf"
                | "zip"
                | "tar"
                | "gz"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
                | "bin"
                | "dat"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

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
