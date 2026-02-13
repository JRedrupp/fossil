# TODO.md to GitHub Issues Migration

This directory contains scripts and data to migrate the TODO.md file into GitHub issues.

## Overview

The TODO.md file has been parsed and converted into structured data that can be used to create GitHub issues programmatically. This allows better tracking, discussion, and prioritization of features through GitHub's issue system.

## Files

- **todo_to_issues.json** - Structured JSON containing all TODO items with titles, descriptions, and labels
- **create_github_issues.sh** - Bash script using GitHub CLI (gh) to create issues
- **create_github_issues.py** - Python script using GitHub API to create issues

## Option 1: Using GitHub CLI (Recommended)

### Prerequisites
- Install GitHub CLI: https://cli.github.com/
- Authenticate: `gh auth login`

### Usage
```bash
# Dry run to see what would be created
./create_github_issues.sh --dry-run

# Create all issues
./create_github_issues.sh
```

## Option 2: Using Python Script

### Prerequisites
- Python 3.6+
- requests library: `pip install requests`
- GitHub Personal Access Token with `repo` scope

### Usage
```bash
# Dry run
python3 create_github_issues.py --dry-run

# Create issues
export GITHUB_TOKEN=your_token_here
python3 create_github_issues.py

# Or specify token directly
python3 create_github_issues.py --token your_token_here
```

## Option 3: Manual Creation

You can manually create issues by:
1. Opening `todo_to_issues.json`
2. Copying the title and body for each issue
3. Creating issues at https://github.com/JRedrupp/fossil/issues/new
4. Adding the appropriate labels

## Issue Structure

Each issue includes:
- **Title**: Concise feature description
- **Body**: Detailed information with implementation notes
- **Labels**: Priority level and categorization
  - Priority: `high-priority`, `medium-priority`, `nice-to-have`, `infrastructure`, `community`
  - Categories: `enhancement`, `feature`, `output-format`, `filtering`, `github-integration`, etc.

## Total Issues

The migration will create **40 GitHub issues** covering:
- 5 High Priority features
- 7 Medium Priority features  
- 23 Low Priority / Nice-to-Have features
- 3 Infrastructure items
- 2 Community items

## After Migration

Once the issues are created, the TODO.md file can be:
1. Archived or removed
2. Updated to reference the GitHub issues
3. Kept as a lightweight roadmap with links to issues

## Labels to Create

Before running the scripts, you may want to create these labels in your repository:

**Priority Labels:**
- `high-priority` (color: #d73a4a)
- `medium-priority` (color: #fbca04)
- `nice-to-have` (color: #0e8a16)
- `infrastructure` (color: #5319e7)
- `community` (color: #d876e3)

**Category Labels:**
- `enhancement` (usually exists by default)
- `feature` (color: #a2eeef)
- `output-format` (color: #bfdadc)
- `filtering` (color: #c5def5)
- `github-integration` (color: #0052cc)
- `ci-cd` (color: #006b75)
- `ux` (color: #d4c5f9)
- `developer-experience` (color: #c2e0c6)
- `language-support` (color: #bfd4f2)
- `export-format` (color: #d4c5f9)
- `ai-powered` (color: #ff6b6b)
- `integrations` (color: #84b6eb)
- `performance` (color: #fef2c0)
- `monorepo` (color: #c5def5)
- `git-integration` (color: #0052cc)
- `configuration` (color: #bfd4f2)
- `ide-integration` (color: #d4c5f9)
- `metrics` (color: #fbca04)
- `analytics` (color: #f9d0c4)
- `collaboration` (color: #c2e0c6)
- `testing` (color: #d93f0b)
- `distribution` (color: #0e8a16)
- `documentation` (color: #0075ca)
- `marketing` (color: #ff6b6b)
- `extensibility` (color: #5319e7)

You can create labels using:
```bash
gh label create "label-name" --color "hexcode" --description "description"
```

Or use the GitHub web interface: https://github.com/JRedrupp/fossil/labels
