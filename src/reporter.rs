use crate::cli::OutputFormat;
use crate::models::DebtReport;
use anyhow::{Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use std::fs;
use std::path::Path;

/// Generate and output a report in the specified format
pub fn generate_report(
    report: &DebtReport,
    format: OutputFormat,
    output_path: Option<&Path>,
    top_n: usize,
) -> Result<()> {
    let output = match format {
        OutputFormat::Terminal => format_terminal(report, top_n),
        OutputFormat::Markdown => format_markdown(report, top_n),
        OutputFormat::Json => format_json(report)?,
    };

    if let Some(path) = output_path {
        fs::write(path, output)
            .with_context(|| format!("Failed to write output to {}", path.display()))?;
        println!("Report written to {}", path.display());
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Format report as terminal table
fn format_terminal(report: &DebtReport, top_n: usize) -> String {
    let mut output = String::new();

    // Header
    let line = "─".repeat(58);
    output.push_str(&format!("╭{}╮\n", line));
    output.push_str(&format!("│ {:^56} │\n", "Fossil - Technical Debt Report"));
    output.push_str(&format!("│ Scanned: {:<47} │\n", report.scan_path.display()));
    output.push_str(&format!("│ Total Markers: {:<41} │\n", report.total_count));
    output.push_str(&format!("╰{}╯\n\n", line));

    // Summary by type
    if !report.by_type.is_empty() {
        output.push_str("Summary by Type:\n");
        let mut type_table = Table::new();
        type_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Type").fg(Color::Cyan),
                Cell::new("Count").fg(Color::Cyan),
            ]);

        let mut types: Vec<_> = report.by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

        for (marker_type, count) in types {
            type_table.add_row(vec![marker_type.as_str(), &count.to_string()]);
        }

        output.push_str(&format!("{}\n\n", type_table));
    }

    // Summary by author
    if !report.by_author.is_empty() {
        output.push_str("Summary by Author:\n");
        let mut author_table = Table::new();
        author_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Author").fg(Color::Cyan),
                Cell::new("Count").fg(Color::Cyan),
            ]);

        let mut authors: Vec<_> = report.by_author.iter().collect();
        authors.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        authors.truncate(10); // Show top 10 authors

        for (author, count) in authors {
            author_table.add_row(vec![author.as_str(), &count.to_string()]);
        }

        output.push_str(&format!("{}\n\n", author_table));
    }

    // Top N oldest markers
    let oldest = report.oldest_markers(top_n);
    if !oldest.is_empty() {
        output.push_str(&format!("Top {} Oldest Markers:\n", oldest.len()));
        let mut oldest_table = Table::new();
        oldest_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Type").fg(Color::Cyan),
                Cell::new("File").fg(Color::Cyan),
                Cell::new("Line").fg(Color::Cyan),
                Cell::new("Author").fg(Color::Cyan),
                Cell::new("Age").fg(Color::Cyan),
            ]);

        for marker in oldest {
            let git_info = marker.git_info.as_ref().unwrap();
            oldest_table.add_row(vec![
                Cell::new(&marker.marker_type),
                Cell::new(marker.file_path.display().to_string()),
                Cell::new(marker.line_number.to_string()),
                Cell::new(&git_info.author),
                Cell::new(git_info.age_display()),
            ]);
        }

        output.push_str(&format!("{}\n", oldest_table));
    }

    output
}

/// Format report as Markdown
fn format_markdown(report: &DebtReport, top_n: usize) -> String {
    let mut output = String::new();

    // Header
    output.push_str("# Fossil - Technical Debt Report\n\n");
    output.push_str(&format!("**Scanned**: `{}`\n", report.scan_path.display()));
    output.push_str(&format!("**Total Markers**: {}\n", report.total_count));
    output.push_str(&format!("**Generated**: {}\n\n", report.scan_time.format("%Y-%m-%d %H:%M:%S UTC")));

    // Summary by type
    if !report.by_type.is_empty() {
        output.push_str("## Summary by Type\n\n");
        let mut types: Vec<_> = report.by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));

        for (marker_type, count) in types {
            output.push_str(&format!("- **{}**: {}\n", marker_type, count));
        }
        output.push('\n');
    }

    // Summary by author
    if !report.by_author.is_empty() {
        output.push_str("## Summary by Author (Top 10)\n\n");
        let mut authors: Vec<_> = report.by_author.iter().collect();
        authors.sort_by(|a, b| b.1.cmp(a.1));
        authors.truncate(10);

        for (author, count) in authors {
            output.push_str(&format!("- **{}**: {}\n", author, count));
        }
        output.push('\n');
    }

    // Top N oldest markers
    let oldest = report.oldest_markers(top_n);
    if !oldest.is_empty() {
        output.push_str(&format!("## Top {} Oldest Markers\n\n", oldest.len()));

        for (idx, marker) in oldest.iter().enumerate() {
            let git_info = marker.git_info.as_ref().unwrap();
            output.push_str(&format!(
                "{}. **{}** in `{}:{}`\n",
                idx + 1,
                marker.marker_type,
                marker.file_path.display(),
                marker.line_number
            ));
            output.push_str(&format!("   - Author: {}\n", git_info.author));
            output.push_str(&format!("   - Age: {} ({} days)\n", git_info.age_display(), git_info.age_days));
            output.push_str(&format!("   - Commit: {}\n", git_info.commit_hash));
            output.push_str(&format!("   - Line: `{}`\n", marker.line_content.trim()));

            // Add context if available
            if !marker.context_before.is_empty() || !marker.context_after.is_empty() {
                output.push_str("   - Context:\n```\n");
                for line in &marker.context_before {
                    output.push_str(&format!("{}\n", line));
                }
                output.push_str(&format!("{} <-- MARKER\n", marker.line_content));
                for line in &marker.context_after {
                    output.push_str(&format!("{}\n", line));
                }
                output.push_str("```\n");
            }
            output.push('\n');
        }
    }

    output
}

/// Format report as JSON
fn format_json(report: &DebtReport) -> Result<String> {
    serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DebtMarker, GitBlameInfo};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_report() -> DebtReport {
        let marker1 = DebtMarker {
            marker_type: "TODO".to_string(),
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            line_content: "// TODO: implement this".to_string(),
            context_before: vec!["fn main() {".to_string()],
            context_after: vec!["    println!(\"hello\");".to_string()],
            git_info: Some(GitBlameInfo {
                author: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                commit_hash: "abc123".to_string(),
                commit_time: Utc::now(),
                age_days: 100,
            }),
        };

        let marker2 = DebtMarker {
            marker_type: "FIXME".to_string(),
            file_path: PathBuf::from("src/lib.rs"),
            line_number: 10,
            line_content: "// FIXME: broken".to_string(),
            context_before: vec![],
            context_after: vec![],
            git_info: Some(GitBlameInfo {
                author: "Bob".to_string(),
                author_email: "bob@example.com".to_string(),
                commit_hash: "def456".to_string(),
                commit_time: Utc::now(),
                age_days: 50,
            }),
        };

        DebtReport::new(vec![marker1, marker2], PathBuf::from("/test/project"))
    }

    #[test]
    fn test_format_terminal() {
        let report = create_test_report();
        let output = format_terminal(&report, 10);

        assert!(output.contains("Fossil - Technical Debt Report"));
        assert!(output.contains("Total Markers: 2"));
        assert!(output.contains("TODO"));
        assert!(output.contains("FIXME"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_format_markdown() {
        let report = create_test_report();
        let output = format_markdown(&report, 10);

        assert!(output.contains("# Fossil - Technical Debt Report"));
        assert!(output.contains("**Total Markers**: 2"));
        assert!(output.contains("## Summary by Type"));
        assert!(output.contains("TODO"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_format_json() {
        let report = create_test_report();
        let output = format_json(&report).unwrap();

        assert!(output.contains("\"marker_type\""));
        assert!(output.contains("TODO"));
        assert!(output.contains("alice@example.com"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_count"], 2);
    }
}
