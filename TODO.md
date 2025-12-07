# Fossil - Future Development Roadmap

This document tracks potential features and improvements for future versions of Fossil.

## High Priority

### Output Format Enhancements

#### Compact JSON Flag
- Add `--compact-json` flag for minified JSON output
- Currently JSON is always pretty-printed with `to_string_pretty()`
- Useful for piping to other tools or reducing output size
- Simple implementation: add boolean parameter to `format_json()`

#### Sort Order Control
- Add `--sort-by` flag to control sort order for terminal/markdown output
- Options: `age` (default), `author`, `type`, `file`
- Add `SortOrder` enum in `models.rs`
- Add `sorted_markers()` method to `DebtReport`
- Update `format_terminal()` and `format_markdown()` to accept sort parameter
- Update table headers dynamically based on sort order

### Historical Tracking (`fossil history`)
- Store debt snapshots over time in `.fossil/` directory
- Show trends: debt increasing or decreasing?
- Graph debt evolution with ASCII charts or export data for visualization
- Track velocity: how quickly is debt being addressed?
- Commands:
  - `fossil history init` - Initialize tracking
  - `fossil history snapshot` - Take a snapshot
  - `fossil history show` - Display trend chart
  - `fossil history compare <date1> <date2>` - Compare two snapshots

### Progress Indicator
- Add progress bar for large scans using `indicatif` crate
- Show: files scanned, markers found, current file being processed
- Estimated time remaining based on file count

### Multi-Directory Support
- Scan multiple directories in a single command
- `fossil scan dir1/ dir2/ dir3/`
- Parallel scanning across directories
- Aggregate report combining all scans

## Medium Priority

### Enhanced Filtering

#### By File Pattern
- `fossil scan --files="*.rs"` - Only scan Rust files
- `fossil scan --exclude="test/**"` - Exclude test directories
- Glob pattern support for fine-grained control

#### By Date Range
- `fossil scan --since=2024-01-01` - Only show markers added after date
- `fossil scan --between=2024-01-01,2024-06-30` - Date range

#### By Severity
- `fossil scan --severity=high` - Filter by configured severity levels
- `fossil scan --critical-only` - Show only critical markers

### GitHub Integration

#### GitHub Action
- Pre-built GitHub Action for CI/CD
- Automatically comment on PRs with debt introduced
- Fail builds if debt exceeds configured threshold
- Example:
  ```yaml
  - uses: fossil-action@v1
    with:
      fail-on-new-debt: true
      max-debt: 50
  ```

#### GitHub Issues
- `fossil github create-issues` - Create GitHub issues from markers
- Link markers to existing issues
- Auto-close issues when markers are removed

### HTML Report
- Generate single-file HTML report with interactive features
- Sortable tables (by age, author, file, type)
- Filterable views
- Charts and visualizations (using Chart.js or similar)
- Dark mode support
- Export as static site for hosting

### Pre-commit Hook
- Example pre-commit hook script
- Warn developers when adding new high-severity debt
- Optional: block commits with certain marker types
- Configurable threshold

## Low Priority / Nice-to-Have

### Language-Specific Intelligence
- Detect TODO patterns specific to languages:
  - Python: `# FIXME(author): description`
  - Java: `@todo` annotations
  - Rust: `todo!()` macro calls
  - TypeScript: `// @ts-ignore` with explanation
- Extract assignee from patterns like `TODO(jake):`

### Diff Mode
- `fossil diff` - Compare current state with previous scan
- Show new debt added, old debt removed
- Highlight changes since last commit/branch/tag

### Export Formats

#### CSV Export
- For importing into spreadsheets
- Columns: Type, File, Line, Author, Age, Content

#### SARIF Format
- For integration with GitHub Code Scanning
- Industry-standard format for static analysis

#### SQLite Database
- Export to SQLite for complex queries
- Enable custom reporting and analysis

### AI-Powered Features

#### Smart Categorization
- Use LLM to categorize debt by impact/effort
- Suggest which TODOs to tackle first
- Detect patterns in technical debt

#### Auto-Documentation
- Generate documentation from NOTE markers
- Create API docs from TODO comments about missing features

### Webhooks & Notifications

#### Slack/Discord Integration
- Post weekly debt reports to channels
- Notify when debt exceeds thresholds
- Celebrate debt reduction milestones

#### Email Digests
- Weekly/monthly email summaries
- Per-author debt reports
- Team-wide debt metrics

### Performance Optimizations

#### Incremental Scanning
- Only scan changed files (using git diff)
- Cache results for unchanged files
- Dramatically faster re-scans

#### Distributed Scanning
- Split large repos across multiple machines
- Aggregate results from distributed scans
- For monorepo support

### Advanced Git Features

#### Blame Improvements
- Track when TODO was actually added vs when line was modified
- Use git log to find original TODO introduction
- Show full commit message for context

#### Branch Comparison
- `fossil compare main..feature-branch`
- Show debt introduced in feature branch
- Use in PR reviews

### Configuration Enhancements

#### Global Ignores
- Define patterns in config, not just directories
- `ignore_patterns = ["*.generated.rs", "vendor/**"]`

#### Per-Directory Config
- Support `.fossilrc` in subdirectories
- Inherit and override parent configs
- Useful for monorepos

#### Team Presets
- Shareable config presets
- `fossil init --preset=rust-team`
- `fossil init --preset=javascript-strict`

### Developer Experience

#### Watch Mode
- `fossil watch` - Auto-scan on file changes
- Live updates during development
- Real-time debt tracking

#### IDE Integration
- VS Code extension showing debt inline
- IntelliJ plugin
- Neovim/Emacs LSP integration

#### Interactive Mode
- `fossil scan --interactive`
- Browse markers with TUI
- Jump to files, mark as resolved
- ncurses-based interface

### Metrics & Analytics

#### Debt Metrics
- Calculate "debt score" based on age and severity
- Technical debt index (TDI)
- Team leaderboards (gamification)

#### Trend Analysis
- Predict future debt growth
- Identify debt hot-spots
- Recommend refactoring targets

### Collaboration Features

#### Debt Assignments
- Parse `TODO(username):` format
- Generate per-developer reports
- Track individual debt accountability

#### Code Review Integration
- Comment on pull requests
- Suggest reviewers based on debt authorship
- Block merges with unaddressed high-severity debt

## Infrastructure

### Testing
- Add integration tests with real repositories
- Benchmark suite for performance regression testing
- Fuzzing for scanner edge cases

### CI/CD
- Automated releases with GitHub Actions
- Multi-platform binaries (Linux, macOS, Windows)
- Homebrew tap for easy installation
- Cargo publish to crates.io

### Documentation
- Video tutorials
- Blog post series on technical debt management
- Conference talk materials

## Community

### Website
- Project website with live demo
- Showcase real-world examples
- User testimonials

### Ecosystem
- Plugin system for custom markers
- Extension API for third-party integrations
- Community-contributed configs and presets

---

## How to Contribute

If you'd like to work on any of these features:

1. Open an issue to discuss the approach
2. Reference this TODO.md in your PR
3. Update this file to move items from TODO to DONE
4. Add tests and documentation

## Prioritization

Features will be prioritized based on:
- Community demand (GitHub issues, stars)
- Implementation complexity
- Alignment with core mission
- Maintainability impact

---

**Last Updated**: 2025-12-07
