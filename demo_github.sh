#!/bin/bash
# DevUtils GitHub Automation Script
# For Windows, use PowerShell equivalent commands

set -e

echo "================================================"
echo "DevUtils GitHub Automation Demo"
echo "================================================"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) not found"
    echo "   Install from: https://cli.github.com/"
    exit 1
fi

# Check if in git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not a git repository"
    echo "   Initialize with: git init"
    exit 1
fi

echo ""
echo "📊 Current Git Status:"
git status --short

echo ""
echo "🔀 Current Branch:"
git branch --show-current

echo ""
echo "🔗 Remote URL:"
git remote get-url origin 2>/dev/null || echo "No remote configured"

echo ""
echo "================================================"
echo "Available GitHub Commands:"
echo "================================================"
echo ""
echo "devutils github status     - Check CI status"
echo "devutils github commit     - AI auto-commit"
echo "devutils github push       - Push changes"
echo "devutils github pr [title] - Create PR"
echo "devutils github auto [msg] - Full: commit + push + PR"
echo "devutils github list       - List PRs"
echo "devutils github sync       - Sync with remote"
echo ""