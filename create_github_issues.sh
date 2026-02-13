#!/bin/bash
# Script to create GitHub issues from TODO.md using GitHub CLI (gh)
#
# Prerequisites:
#   - GitHub CLI (gh) must be installed
#   - You must be authenticated: gh auth login
#
# Usage:
#   ./create_github_issues.sh [--dry-run]

set -e

REPO="JRedrupp/fossil"
JSON_FILE="todo_to_issues.json"
DRY_RUN=false

# Parse arguments
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
fi

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed."
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI."
    echo "Run: gh auth login"
    exit 1
fi

# Check if JSON file exists
if [[ ! -f "$JSON_FILE" ]]; then
    echo "Error: $JSON_FILE not found."
    exit 1
fi

# Count total issues
TOTAL=$(jq length "$JSON_FILE")
echo "Found $TOTAL issues to create in $JSON_FILE"

if [[ "$DRY_RUN" == true ]]; then
    echo ""
    echo "=== DRY RUN MODE - No issues will be created ==="
    echo ""
    jq -r '.[] | "\(.title)\n  Labels: \(.labels | join(", "))\n"' "$JSON_FILE"
    exit 0
fi

# Create issues
CREATED=0
FAILED=0

for i in $(seq 0 $((TOTAL - 1))); do
    # Extract issue data
    TITLE=$(jq -r ".[$i].title" "$JSON_FILE")
    BODY=$(jq -r ".[$i].body" "$JSON_FILE")
    LABELS=$(jq -r ".[$i].labels | join(\",\")" "$JSON_FILE")
    
    echo -n "Creating issue $((i + 1))/$TOTAL: $TITLE... "
    
    # Create the issue
    ERROR_OUTPUT=$(gh issue create \
        --repo "$REPO" \
        --title "$TITLE" \
        --body "$BODY" \
        --label "$LABELS" 2>&1)
    
    if [ $? -eq 0 ]; then
        echo "✓"
        ((CREATED++))
    else
        echo "✗ Failed"
        echo "  Error: $ERROR_OUTPUT"
        ((FAILED++))
    fi
done

echo ""
echo "=== Summary ==="
echo "Successfully created: $CREATED"
echo "Failed: $FAILED"
echo "Total: $TOTAL"
