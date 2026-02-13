# How to Complete the TODO.md Migration

## What Has Been Done

This PR provides the infrastructure to migrate TODO.md to GitHub issues:

1. **todo_to_issues.json** - Contains 40 structured issues extracted from TODO.md
2. **create_github_issues.sh** - Bash script using GitHub CLI 
3. **create_github_issues.py** - Python script using GitHub API
4. **MIGRATION_README.md** - Comprehensive guide with all options
5. **TODO.md** - Updated to reference the new GitHub issue system
6. **.gitignore** - Updated to include migration files

## Next Steps

To complete the migration, you need to run one of the scripts to actually create the GitHub issues:

### Option 1: Using GitHub CLI (Easiest)

```bash
# Make sure you're authenticated
gh auth login

# Run the script
./create_github_issues.sh
```

### Option 2: Using Python

```bash
# Install dependencies
pip install requests

# Create a GitHub Personal Access Token with 'repo' scope
# at https://github.com/settings/tokens

# Run the script
export GITHUB_TOKEN=your_token_here
python3 create_github_issues.py
```

## What Will Happen

The script will create **40 GitHub issues** with:
- Appropriate titles and descriptions
- Labels for priority and category
- All details from the TODO.md file

## After Running the Script

1. Review the created issues at https://github.com/JRedrupp/fossil/issues
2. Consider organizing them with GitHub Projects or Milestones
3. The TODO.md file can remain as a quick reference, but GitHub issues will be the source of truth

## Important Notes

- The scripts are idempotent-safe - they will create new issues each time you run them
- If you want to test first, use `--dry-run` flag to see what would be created
- You may want to create the label categories in your repository first (see MIGRATION_README.md)

## Verification

All code changes have been tested:
- ✅ Project builds successfully (`cargo build`)
- ✅ All tests pass (`cargo test`)
- ✅ Scripts validated with `--dry-run`
- ✅ No code functionality changes
