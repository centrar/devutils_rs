# DevUtils GitHub Automation - Usage Examples

## Prerequisites
# Install GitHub CLI: https://cli.github.com/
# Windows: winget install GitHub.cli
# Mac: brew install gh
# Linux: sudo apt install gh

## 1. Check CI/CD Status
devutils github status

## 2. Auto-Commit Changes
# AI generates commit message based on your changes
devutils github commit

## 3. Push to Remote
devutils github push
# Force push (use carefully!)
devutils github push --force

## 4. Create Pull Request
# With AI-generated description based on changes
devutils github pr "Add user authentication"

## 5. FULL AUTOMATION - One Command Does Everything
# 1. Stage all changes
# 2. AI auto-generate commit message
# 3. Commit
# 4. Push
# 5. Create PR with AI description
devutils github auto "Implement feature X"

## 6. Sync with Remote (fetch + rebase)
devutils github sync main
devutils github sync develop

## 7. List Open PRs
devutils github list

## 8. Merge PR
devutils github merge 42

## 9. Create Issue
devutils github issue "Bug: login fails on mobile"

## 10. Create Release
devutils github release v1.0.0 "Initial release with all features"

## Complete Workflow Example

# Step 1: Let AI create new feature
devutils autonomous "create user profile management module"

# Step 2: AI auto-commits and pushes
devutils github auto "Add user profile management"

# Step 3: Check CI passes
devutils github status

# Step 4: Merge when ready
devutils github merge 42

# OR use resolve for merge conflicts
devutils resolve --strategy ai

## Demo: Modify + Commit + PR

# Modify a file atomically
devutils edit README.md "old text" "new text"

# Commit with AI message
devutils github commit

# Push
devutils github push

# Create PR
devutils github pr "Update README with new features"