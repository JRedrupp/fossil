#!/usr/bin/env python3
"""
Script to create GitHub issues from the TODO.md file.

This script reads the todo_to_issues.json file and creates GitHub issues
for each entry using the GitHub API.

Usage:
    export GITHUB_TOKEN=your_github_token
    python3 create_github_issues.py

Or:
    python3 create_github_issues.py --token your_github_token
"""

import json
import os
import sys
import argparse
import requests
from typing import List, Dict

REPO_OWNER = "JRedrupp"
REPO_NAME = "fossil"


def create_issue(token: str, issue_data: Dict) -> Dict:
    """Create a GitHub issue using the GitHub API."""
    url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/issues"
    
    headers = {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json"
    }
    
    # Extract labels - handle both string and list format
    labels = issue_data.get("labels", [])
    if isinstance(labels, str):
        try:
            labels = json.loads(labels)
        except json.JSONDecodeError:
            print(f"Warning: Could not parse labels for '{issue_data['title']}'")
            labels = []
    
    payload = {
        "title": issue_data["title"],
        "body": issue_data["body"],
        "labels": labels
    }
    
    response = requests.post(url, headers=headers, json=payload)
    
    if response.status_code == 201:
        return response.json()
    else:
        print(f"Error creating issue '{issue_data['title']}': {response.status_code}")
        print(f"Response: {response.text}")
        return None


def main():
    parser = argparse.ArgumentParser(description="Create GitHub issues from TODO.md")
    parser.add_argument("--token", help="GitHub personal access token")
    parser.add_argument("--dry-run", action="store_true", help="Don't actually create issues, just print what would be created")
    parser.add_argument("--json-file", default="todo_to_issues.json", help="Path to JSON file with issue data")
    args = parser.parse_args()
    
    # Get GitHub token
    token = args.token or os.environ.get("GITHUB_TOKEN")
    if not token and not args.dry_run:
        print("Error: GitHub token is required. Set GITHUB_TOKEN environment variable or use --token flag.")
        sys.exit(1)
    
    # Load issue data
    try:
        with open(args.json_file, 'r') as f:
            issues = json.load(f)
    except FileNotFoundError:
        print(f"Error: {args.json_file} not found.")
        sys.exit(1)
    except json.JSONDecodeError:
        print(f"Error: {args.json_file} is not valid JSON.")
        sys.exit(1)
    
    print(f"Found {len(issues)} issues to create.")
    
    if args.dry_run:
        print("\n=== DRY RUN MODE - No issues will be created ===\n")
        for i, issue in enumerate(issues, 1):
            print(f"{i}. {issue['title']}")
            labels = issue.get("labels", [])
            if isinstance(labels, str):
                try:
                    labels = json.loads(labels)
                except json.JSONDecodeError:
                    labels = []
            print(f"   Labels: {', '.join(labels)}")
            print()
        return
    
    # Create issues
    created_issues = []
    for i, issue_data in enumerate(issues, 1):
        print(f"Creating issue {i}/{len(issues)}: {issue_data['title']}...", end=" ")
        result = create_issue(token, issue_data)
        if result:
            print(f"✓ Created #{result['number']}")
            created_issues.append(result)
        else:
            print("✗ Failed")
    
    print(f"\n=== Summary ===")
    print(f"Successfully created {len(created_issues)} out of {len(issues)} issues.")
    
    if created_issues:
        print("\nCreated issues:")
        for issue in created_issues:
            print(f"  - #{issue['number']}: {issue['title']}")
            print(f"    {issue['html_url']}")


if __name__ == "__main__":
    main()
