use crate::models::{DebtMarker, GitBlameInfo};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{BlameOptions, Repository};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Get git repository for a given path
pub fn get_repository(path: &Path) -> Result<Option<Repository>> {
    match Repository::discover(path) {
        Ok(repo) => Ok(Some(repo)),
        Err(_) => Ok(None), // Not a git repository - this is okay
    }
}

/// Get git blame information for a specific line in a file
pub fn blame_line(
    repo: &Repository,
    file_path: &Path,
    line_number: usize,
) -> Result<Option<GitBlameInfo>> {
    // Get the file path relative to the repository root
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;

    // Canonicalize the file path to handle .. and . in the path
    let canonical_path = file_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", file_path.display()))?;

    let relative_path = canonical_path
        .strip_prefix(workdir)
        .unwrap_or(&canonical_path);

    // Remove leading "./" if present - git2 doesn't accept paths starting with "."
    let relative_path_str = relative_path
        .to_str()
        .context("Invalid UTF-8 in file path")?;
    let cleaned_path = relative_path_str
        .strip_prefix("./")
        .unwrap_or(relative_path_str);
    let relative_path = Path::new(cleaned_path);

    // Create blame options
    let mut opts = BlameOptions::new();
    opts.track_copies_same_file(true)
        .track_copies_same_commit_moves(true)
        .track_copies_same_commit_copies(true);

    // Run blame on the file
    let blame = match repo.blame_file(relative_path, Some(&mut opts)) {
        Ok(b) => b,
        Err(_) => return Ok(None), // File might not be in git, or other error
    };

    // Get the hunk for the specific line
    // Note: git blame uses 1-indexed lines, which matches our line_number
    let hunk = match blame.get_line(line_number) {
        Some(h) => h,
        None => return Ok(None), // Line not found
    };

    // Get the commit info
    let commit_id = hunk.final_commit_id();
    let commit = repo.find_commit(commit_id)?;

    // Extract author info
    let author = commit.author();
    let author_name = author.name().unwrap_or("Unknown").to_string();
    let author_email = author.email().unwrap_or("unknown@example.com").to_string();

    // Get commit time
    let commit_time_secs = commit.time().seconds();
    let commit_time = DateTime::from_timestamp(commit_time_secs, 0).unwrap_or_else(Utc::now);

    // Calculate age in days
    let now = Utc::now();
    let duration = now.signed_duration_since(commit_time);
    let age_days = duration.num_days();

    // Get short commit hash
    let commit_hash = format!("{:.7}", commit_id);

    Ok(Some(GitBlameInfo {
        author: author_name,
        author_email,
        commit_hash,
        commit_time,
        age_days,
    }))
}

/// Enrich a marker with git blame information
pub fn enrich_with_git_info(
    repo: Option<&Repository>,
    file_path: &Path,
    line_number: usize,
) -> Option<GitBlameInfo> {
    let repo = repo?;

    blame_line(repo, file_path, line_number).unwrap_or_default()
}

/// Batch enrich markers with git blame information
/// Groups markers by file and runs git blame once per file for better performance
pub fn enrich_markers_batch(markers: &mut [DebtMarker], repo: Option<&Repository>) -> Result<()> {
    let repo = match repo {
        Some(r) => r,
        None => return Ok(()), // No repository, skip enrichment
    };

    // Get repository working directory once
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;

    // Group markers by file path
    let mut markers_by_file: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for (idx, marker) in markers.iter().enumerate() {
        markers_by_file
            .entry(marker.file_path.clone())
            .or_default()
            .push(idx);
    }

    // Process each file once
    for (file_path, marker_indices) in markers_by_file {
        // Canonicalize and convert to relative path

        // Canonicalize the file path to handle .. and . in the path
        let canonical_path = match file_path.canonicalize() {
            Ok(p) => p,
            Err(_) => continue, // Skip files that can't be canonicalized
        };

        let relative_path = canonical_path
            .strip_prefix(workdir)
            .unwrap_or(&canonical_path);

        // Remove leading "./" if present
        let relative_path_str = match relative_path.to_str() {
            Some(s) => s,
            None => continue, // Skip files with invalid UTF-8
        };
        let cleaned_path = relative_path_str
            .strip_prefix("./")
            .unwrap_or(relative_path_str);
        let relative_path = Path::new(cleaned_path);

        // Create blame options
        let mut opts = BlameOptions::new();
        opts.track_copies_same_file(true)
            .track_copies_same_commit_moves(true)
            .track_copies_same_commit_copies(true);

        // Run blame once for this file
        let blame = match repo.blame_file(relative_path, Some(&mut opts)) {
            Ok(b) => b,
            Err(_) => continue, // Skip files that can't be blamed
        };

        // Cache blame info by line number
        let mut blame_cache: HashMap<usize, GitBlameInfo> = HashMap::new();

        // Process all markers for this file
        for &marker_idx in &marker_indices {
            let marker = &mut markers[marker_idx];
            let line_number = marker.line_number;

            // Check cache first
            if let Some(git_info) = blame_cache.get(&line_number) {
                marker.git_info = Some(git_info.clone());
                continue;
            }

            // Get the hunk for this line
            let hunk = match blame.get_line(line_number) {
                Some(h) => h,
                None => continue, // Skip lines not found in blame
            };

            // Get the commit info
            let commit_id = hunk.final_commit_id();
            let commit = match repo.find_commit(commit_id) {
                Ok(c) => c,
                Err(_) => continue, // Skip if commit not found
            };

            // Extract author info
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            let author_email = author.email().unwrap_or("unknown@example.com").to_string();

            // Get commit time
            let commit_time_secs = commit.time().seconds();
            let commit_time =
                DateTime::from_timestamp(commit_time_secs, 0).unwrap_or_else(Utc::now);

            // Calculate age in days
            let now = Utc::now();
            let duration = now.signed_duration_since(commit_time);
            let age_days = duration.num_days();

            // Get short commit hash
            let commit_hash = format!("{:.7}", commit_id);

            let git_info = GitBlameInfo {
                author: author_name,
                author_email,
                commit_hash,
                commit_time,
                age_days,
            };

            // Cache and assign
            blame_cache.insert(line_number, git_info.clone());
            marker.git_info = Some(git_info);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Configure git
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create a test file
        let test_file = repo_path.join("test.rs");
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(file, "// First line").unwrap();
        writeln!(file, "// TODO: test marker").unwrap();
        writeln!(file, "// Third line").unwrap();

        // Commit the file
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_get_repository() {
        let temp_dir = create_test_repo();
        let repo = get_repository(temp_dir.path()).unwrap();
        assert!(repo.is_some());

        // Test non-git directory
        let non_git = TempDir::new().unwrap();
        let no_repo = get_repository(non_git.path()).unwrap();
        assert!(no_repo.is_none());
    }

    #[test]
    fn test_blame_line() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let info = blame_line(&repo, &file_path, 2).unwrap();
        assert!(info.is_some());

        let git_info = info.unwrap();
        assert_eq!(git_info.author, "Test User");
        assert_eq!(git_info.author_email, "test@example.com");
        assert!(git_info.commit_hash.len() == 7);
        assert!(git_info.age_days >= 0);
    }

    #[test]
    fn test_enrich_with_git_info() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let info = enrich_with_git_info(Some(&repo), &file_path, 2);
        assert!(info.is_some());

        let git_info = info.unwrap();
        assert_eq!(git_info.author, "Test User");

        // Test with None repository
        let no_info = enrich_with_git_info(None, &file_path, 2);
        assert!(no_info.is_none());
    }
}
