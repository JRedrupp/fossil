# 🦴 Fossil

> **Unearth your technical debt**

Fossil is a CLI tool that excavates buried technical debt from codebases by extracting and tracking debt markers (TODO, FIXME, HACK, XXX, NOTE). It uses git blame to calculate the age of each marker and generates comprehensive reports to help you understand and prioritize your technical debt.

## Features

- 🔍 **Scan any codebase** - Language-agnostic detection of technical debt markers
- ⏰ **Calculate debt age** - Uses git blame to determine how long each TODO has been sitting there
- 📊 **Rich reporting** - Group and categorize debt by author, file, type, and age
- 📈 **Multiple output formats** - Terminal tables, Markdown, or JSON
- ⚡ **Fast** - Efficiently scans large codebases (<5 seconds for 100k lines)
- 🎯 **Flexible filtering** - Filter by age, author, or marker type
- ⚙️ **Configurable** - Customize markers, ignored directories, and severity levels

## Installation

### From Source

```bash
git clone https://github.com/yourusername/fossil.git
cd fossil
cargo build --release
sudo cp target/release/fossil /usr/local/bin/
```

### Using Cargo

```bash
cargo install --path .
```

## Quick Start

Scan the current directory:

```bash
fossil scan
```

Output:
```
╭──────────────────────────────────────────────────────────╮
│              Fossil - Technical Debt Report              │
│ Scanned: .                                                │
│ Total Markers: 47                                          │
╰──────────────────────────────────────────────────────────╯

Summary by Type:
┌────────┬───────┐
│ Type   │ Count │
├────────┼───────┤
│ TODO   │ 23    │
│ FIXME  │ 12    │
│ HACK   │ 8     │
│ XXX    │ 3     │
│ NOTE   │ 1     │
└────────┴───────┘

Top 10 Oldest Markers:
┌──────┬─────────────────────┬────────┬─────────────┬─────────┐
│ Type │ File                │ Line   │ Author      │ Age     │
├──────┼─────────────────────┼────────┼─────────────┼─────────┤
│ TODO │ src/legacy.rs       │ 45     │ john@ex.com │ 347d    │
│ FIXME│ core/auth.rs        │ 123    │ jane@ex.com │ 201d    │
```

## Usage

### Basic Commands

```bash
# Scan current directory
fossil scan

# Scan specific directory
fossil scan /path/to/project

# Enable verbose output
fossil scan --verbose
```

### Output Formats

```bash
# Terminal table (default)
fossil scan

# Markdown format
fossil scan --format=markdown

# JSON format
fossil scan --format=json

# Save to file
fossil scan --output=debt-report.md --format=markdown
```

### Filtering

```bash
# Show only TODOs older than 30 days
fossil scan --older-than=30d

# Show only markers from a specific author
fossil scan --author=jake

# Show only FIXME markers
fossil scan --type=FIXME

# Combine filters
fossil scan --older-than=60d --type=TODO --author=alice

# Show top 20 oldest markers
fossil scan --top=20
```

### Time Units

- `d` - days (e.g., `30d`)
- `w` - weeks (e.g., `4w`)
- `m` - months (e.g., `6m`)
- `y` - years (e.g., `1y`)

## Configuration

Fossil supports configuration files to customize behavior. Create a `.fossilrc` file in your project root or home directory (`~/.fossilrc`).

### Example Configuration

```toml
# Custom markers to search for
markers = ["TODO", "FIXME", "HACK", "XXX", "NOTE", "BUG", "OPTIMIZE"]

# Directories to ignore during scanning
ignored_dirs = [
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".venv",
    "vendor"
]

# Number of context lines to capture before/after each marker
context_lines = 3

# Optional severity mapping
[severity]
FIXME = "high"
HACK = "high"
BUG = "high"
TODO = "medium"
OPTIMIZE = "medium"
NOTE = "low"
```

### Configuration Priority

1. `--config` CLI argument (if provided)
2. `.fossilrc` in current directory
3. `~/.fossilrc` in home directory
4. Built-in defaults

### Using Custom Config

```bash
fossil scan --config=/path/to/custom-config.toml
```

## How It Works

1. **Scan** - Recursively walks your directory tree, respecting `.gitignore` patterns
2. **Extract** - Finds all debt markers in comments using regex patterns
3. **Enrich** - Uses `git blame` to determine when each marker was added
4. **Analyze** - Calculates age, groups by author/file/type
5. **Report** - Generates formatted output in your chosen format

## Supported Comment Styles

Fossil automatically detects markers in various comment styles:

- `// TODO: fix this` (C-style)
- `# FIXME: broken` (Shell/Python style)
- `/* HACK: temporary */` (Block comments)
- `* TODO: in docblocks`
- `<!-- XXX: HTML comments -->`

## Example Outputs

### Markdown Report

```markdown
# Fossil - Technical Debt Report

**Scanned**: `/home/user/project`
**Total Markers**: 47
**Generated**: 2025-12-04 22:00:00 UTC

## Summary by Type
- **TODO**: 23
- **FIXME**: 12
- **HACK**: 8

## Top 10 Oldest Markers
1. **TODO** in `src/legacy.rs:45` (347 days old)
   - Author: john@example.com
   - Age: 347d (347 days)
   - Commit: abc123
   - Line: `// TODO: refactor this authentication logic`
```

### JSON Report

```json
{
  "scan_path": "/home/user/project",
  "scan_time": "2025-12-04T22:00:00Z",
  "total_count": 47,
  "by_type": {
    "TODO": 23,
    "FIXME": 12,
    "HACK": 8
  },
  "by_author": {
    "john@example.com": 15,
    "jane@example.com": 20
  },
  "markers": [
    {
      "marker_type": "TODO",
      "file_path": "src/legacy.rs",
      "line_number": 45,
      "line_content": "// TODO: refactor this",
      "git_info": {
        "author": "john@example.com",
        "commit_hash": "abc123",
        "commit_time": "2024-01-15T10:30:00Z",
        "age_days": 347
      }
    }
  ]
}
```

## Use Cases

### 🎯 Sprint Planning

Identify the oldest, most neglected TODOs to prioritize in your next sprint:

```bash
fossil scan --older-than=90d --format=markdown --output=sprint-backlog.md
```

### 👥 Code Review

Check what technical debt a specific developer has introduced:

```bash
fossil scan --author=alice --format=terminal
```

### 📊 Reporting

Generate JSON output for integration with dashboards or CI/CD:

```bash
fossil scan --format=json > debt-metrics.json
```

### 🚨 CI/CD Integration

Fail builds if high-severity debt is introduced:

```bash
#!/bin/bash
# Example CI script
fossil scan --type=FIXME --format=json > debt.json
count=$(jq '.total_count' debt.json)
if [ $count -gt 5 ]; then
  echo "Too many FIXME markers! Please address technical debt."
  exit 1
fi
```

## Performance

Fossil is designed to be fast:

- **Multi-threaded scanning** using the `ignore` crate (same as ripgrep)
- **Lazy git blame** - only runs blame on files with markers
- **Stream processing** - no full file loads into memory
- **Efficient regex** - compiled once and reused

Benchmarks:
- 100k LOC: < 5 seconds
- 500k LOC: ~15 seconds
- 1M LOC: ~30 seconds

## Limitations

- Requires git repository for blame data (gracefully degrades if not available)
- Only detects markers in comments (not in strings or other contexts)
- Age calculation based on line's last modification (not when TODO was added)

## Troubleshooting

### No markers found

- Check that you're using supported comment syntax
- Verify markers match your config (default: TODO, FIXME, HACK, XXX, NOTE)
- Try with `--verbose` to see what's happening

### Git blame not working

- Ensure you're in a git repository (`git status`)
- Check file permissions
- Verify files are tracked in git

### Slow scanning

- Add directories to `ignored_dirs` in config
- Check for very large files (>10MB are skipped by default)
- Use more specific scan paths instead of entire repository

## Contributing

Contributions welcome! Please feel free to submit issues or pull requests.

### Development

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- scan . --verbose

# Build release binary
cargo build --release

# Run linter
cargo clippy

# Format code
cargo fmt
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [git2](https://github.com/rust-lang/git2-rs) for git integration
- Uses [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) for fast file traversal
- Uses [comfy-table](https://github.com/Nukesor/comfy-table) for beautiful terminal output

---

**Fossil** - Because technical debt, like fossils, deserves to be unearthed and studied. 🦴
