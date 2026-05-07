//! GitHub Automation Module
//! 
//! Full GitHub repository automation:
//! - Auto-commit and push
//! - Create PRs with AI-generated descriptions
//! - Check CI/CD status
//! - Auto-merge when tests pass
//! - Sync with remote changes
//! - Manage issues
//! - Create releases
//! - Manage branches

use std::process::Command;

/// GitHub repository info
#[derive(Debug, Clone)]
pub struct Repo {
    pub owner: String,
    pub name: String,
    pub url: String,
}

/// GitHub PR info
#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub url: String,
}

/// GitHub CI status
#[derive(Debug, Clone)]
pub struct CIStatus {
    pub status: String,
    pub conclusion: Option<String>,
    pub checks: Vec<Check>,
}

/// A single check run
#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub status: String,
    pub conclusion: String,
}

/// Git configuration
#[derive(Debug, Clone, Default)]
pub struct GitConfig {
    pub remote: String,
    pub branch: String,
    pub auth_token: Option<String>,
}

/// Get current git remote URL and extract owner/repo
pub fn get_current_repo() -> Result<Repo, String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|e| format!("Failed to get remote: {}", e))?;
    
    if !output.status.success() {
        return Err("Not a git repository or no remote origin".to_string());
    }
    
    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_github_url(&remote_url)
}

/// Parse GitHub URL to extract owner and repo
pub fn parse_github_url(url: &str) -> Result<Repo, String> {
    // Handle SSH format: git@github.com:owner/repo.git
    // Handle HTTPS format: https://github.com/owner/repo.git
    
    let clean = url
        .trim()
        .strip_prefix("https://github.com/")
        .or_else(|| url.trim().strip_prefix("http://github.com/"))
        .or_else(|| url.trim().strip_prefix("git@github.com:"))
        .or_else(|| url.trim().strip_prefix("git://github.com/"))
        .unwrap_or(url);
    
    let parts: Vec<&str> = clean.trim_end_matches(".git").split('/').collect();
    
    if parts.len() < 2 {
        return Err(format!("Invalid GitHub URL: {}", url));
    }
    
let owner = parts[0].to_string();
    let name = parts[1].to_string();
    let url = format!("https://github.com/{}/{}", owner, name);

    Ok(Repo {
        owner,
        name,
        url,
    })
}

/// Get current branch name
pub fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to get current branch".to_string());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Stage all changes
pub fn stage_all() -> Result<(), String> {
    Command::new("git")
        .args(["add", "-A"])
        .output()
        .map_err(|e| format!("Failed to stage: {}", e))?;
    
    Ok(())
}

/// Commit with message
pub fn commit(message: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("Failed to commit: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Commit failed: {}", stderr));
    }
    
    Ok(())
}

/// Push to remote
pub fn push(branch: Option<&str>, force: bool) -> Result<(), String> {
    let branch_name = branch.unwrap_or("HEAD");
    let mut args = vec!["push"];
    
    if force {
        args.push("--force");
    }
    
    args.push("origin");
    args.push(branch_name);
    
    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to push: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Push failed: {}", stderr));
    }
    
    Ok(())
}

/// Create and push a new branch
pub fn create_branch(name: &str) -> Result<(), String> {
    Command::new("git")
        .args(["checkout", "-b", name])
        .output()
        .map_err(|e| format!("Failed to create branch: {}", e))?;
    
    Ok(())
}

/// Delete local branch
pub fn delete_branch(name: &str, force: bool) -> Result<(), String> {
    let mut args = vec!["branch"];
    if force {
        args.push("-D");
    } else {
        args.push("-d");
    }
    args.push(name);
    
    Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to delete branch: {}", e))?;
    
    Ok(())
}

/// Auto-commit all changes with AI-generated message
pub fn auto_commit(ai_client: &crate::ai::AIClient) -> Result<String, String> {
    // Get diff to understand what changed
    let diff = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .output()
        .map_err(|e| format!("Failed to get diff: {}", e))?;
    
    let diff_output = String::from_utf8_lossy(&diff.stdout).to_string();
    
    // Generate commit message with AI
    let prompt = format!(
        "Generate a concise git commit message for these changes:\n{}\n\n\
         Rules:\n\
         - Use imperative mood (add, fix, update, remove)\n\
         - Keep under 72 characters\n\
         - Start with feat:, fix:, docs:, style:, refactor:, test:, chore:\n\
         - Be specific about what changed\n\n\
         Example: 'feat: add user authentication via OAuth'",
        diff_output
    );
    
    let message = ai_client.generate_code(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e).trim().to_string();
    
    commit(&message)?;
    
    Ok(format!("✅ Committed: {}", message))
}

/// Pull latest changes with rebase
pub fn pull_rebase() -> Result<(), String> {
    let output = Command::new("git")
        .args(["pull", "--rebase", "origin"])
        .output()
        .map_err(|e| format!("Failed to pull: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Pull failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

/// Fetch and update all remotes
pub fn fetch_all() -> Result<(), String> {
    Command::new("git")
        .args(["fetch", "--all"])
        .output()
        .map_err(|e| format!("Failed to fetch: {}", e))?;
    
    Ok(())
}

/// Create a GitHub PR using gh CLI
pub fn create_pr(
    title: &str,
    body: &str,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<PullRequest, String> {
    // Check if gh CLI is available
    let gh_check = Command::new("gh").arg("--version").output();
    if gh_check.is_err() {
        return Err("GitHub CLI (gh) not installed. Install from: https://cli.github.com".to_string());
    }
    
    let mut args = vec!["pr", "create", "--title", title, "--body", body];
    
    if let Some(b) = base {
        args.push("--base");
        args.push(b);
    }
    
    if let Some(h) = head {
        args.push("--head");
        args.push(h);
    }
    
    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to create PR: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("PR creation failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Parse PR URL from output
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    // Extract PR number from URL
    let pr_url = stdout.lines().find(|l| l.contains("github.com")).unwrap_or(&stdout);
    let pr_number: u32 = pr_url
        .split('/')
        .last()
        .and_then(|s| s.split_whitespace().next())
        .map(|s| s.trim_start_matches('#'))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    
    Ok(PullRequest {
        number: pr_number,
        title: title.to_string(),
        state: "open".to_string(),
        url: pr_url.to_string(),
    })
}

/// Create PR with AI-generated description
pub fn create_pr_ai(
    ai_client: &crate::ai::AIClient,
    title: &str,
    base: Option<&str>,
) -> Result<PullRequest, String> {
    // Get changed files and diff
    let diff = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(|e| format!("Failed to get diff: {}", e))?;
    
    let diff_output = String::from_utf8_lossy(&diff.stdout).to_string();
    
    // Get commits on branch
    let commits = Command::new("git")
        .args(["log", "--oneline", "-10"])
        .output()
        .map_err(|e| format!("Failed to get commits: {}", e))?;
    
    let commits_output = String::from_utf8_lossy(&commits.stdout).to_string();
    
    // Generate PR body with AI
    let prompt = format!(
        "Generate a detailed GitHub PR description for this pull request.\n\n\
         Title: {}\n\n\
         Changed files summary:\n{}\n\n\
         Recent commits:\n{}\n\n\
         Include:\n\
         1. Summary of changes (2-3 sentences)\n\
         2. Key changes (bullet points)\n\
         3. Testing done\n\
         4. Breaking changes (if any)\n\
         5. Related issues (if any)\n\n\
         Format as Markdown.",
        title,
        diff_output.lines().take(50).collect::<Vec<_>>().join("\n"),
        commits_output
    );
    
    let body = ai_client.generate_code(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e);
    
    // Create the PR
    create_pr(title, &body, base, None)
}

/// Get CI/CD status for a PR
pub fn get_pr_status(pr_number: u32) -> Result<CIStatus, String> {
    let output = Command::new("gh")
        .args(["pr", "view", &pr_number.to_string(), "--json", "statusCheckRollup"])
        .output()
        .map_err(|e| format!("Failed to get PR status: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to get status: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Parse JSON response
    let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let checks: Vec<Check> = json["statusCheckRollup"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|c| Check {
                    name: c["name"].as_str().unwrap_or("unknown").to_string(),
                    status: c["status"].as_str().unwrap_or("UNKNOWN").to_string(),
                    conclusion: c["conclusion"].as_str().unwrap_or("UNKNOWN").to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    
    let overall = checks.iter()
        .find(|c| c.name == "total")
        .map(|c| c.conclusion.clone())
        .unwrap_or_else(|| "PENDING".to_string());
    
    Ok(CIStatus {
        status: if checks.iter().all(|c| c.conclusion == "SUCCESS") {
            "passing".to_string()
        } else if checks.iter().any(|c| c.conclusion == "FAILURE") {
            "failing".to_string()
        } else {
            "pending".to_string()
        },
        conclusion: Some(overall),
        checks,
    })
}

/// Merge a PR
pub fn merge_pr(pr_number: u32, method: Option<&str>, message: Option<&str>) -> Result<String, String> {
    let merge_method = method.unwrap_or("squash");
    let commit_message = message.unwrap_or("Merged via DevUtils");
    
    let output = match merge_method {
        "squash" => {
            Command::new("gh")
                .args(["pr", "merge", &pr_number.to_string(), "--squash", "--message", commit_message])
                .output()
        }
        "merge" => {
            Command::new("gh")
                .args(["pr", "merge", &pr_number.to_string(), "--admin", "--merge"])
                .output()
        }
        "rebase" => {
            Command::new("gh")
                .args(["pr", "merge", &pr_number.to_string(), "--rebase"])
                .output()
        }
        _ => return Err(format!("Unknown merge method: {}", merge_method)),
    }.map_err(|e| format!("Failed to merge PR: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Merge failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(format!("PR #{} merged successfully", pr_number))
}

/// Auto-merge when CI passes
pub fn auto_merge_when_ready(pr_number: u32, timeout_secs: u64) -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timeout = timeout_secs as u64;
    
    loop {
        // Check CI status
        let status = get_pr_status(pr_number)?;
        
        if status.conclusion.as_deref() == Some("SUCCESS") {
            return merge_pr(pr_number, Some("squash"), None);
        }
        
        if status.conclusion.as_deref() == Some("FAILURE") {
            return Err(format!("CI failed for PR #{}", pr_number));
        }
        
        // Check timeout
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let elapsed = now.as_secs() - start.as_secs();
        if elapsed > timeout {
            return Err("Timeout waiting for CI".to_string());
        }
        
        println!("⏳ Waiting for CI to finish... ({}s elapsed)", elapsed);
        
        // Wait 30 seconds before next check
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

/// List open PRs
pub fn list_prs(state: Option<&str>) -> Result<Vec<PullRequest>, String> {
    let state_filter = state.unwrap_or("open");
    
    let output = Command::new("gh")
        .args(["pr", "list", "--state", state_filter, "--json", "number,title,state,url"])
        .output()
        .map_err(|e| format!("Failed to list PRs: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to list PRs: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let json: Vec<serde_json::Value> = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| format!("Failed to parse PR list: {}", e))?;
    
    let prs: Vec<PullRequest> = json.iter()
        .map(|p| PullRequest {
            number: p["number"].as_u64().unwrap_or(0) as u32,
            title: p["title"].as_str().unwrap_or("").to_string(),
            state: p["state"].as_str().unwrap_or("").to_string(),
            url: p["url"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    
    Ok(prs)
}

/// Create an issue
pub fn create_issue(title: &str, body: &str, labels: Option<Vec<&str>>) -> Result<String, String> {
    let mut args = vec!["issue", "create", "--title", title, "--body", body];
    
    if let Some(lbs) = labels {
        for label in lbs {
            args.push("--label");
            args.push(label);
        }
    }
    
    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to create issue: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to create issue: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Create a release
pub fn create_release(tag: &str, name: &str, body: &str, draft: bool) -> Result<String, String> {
    let mut args = vec!["release", "create", tag, "--title", name, "--notes", body];
    
    if draft {
        args.push("--draft");
    }
    
    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to create release: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to create release: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Sync local branch with remote
pub fn sync_with_remote(branch: &str) -> Result<(), String> {
    // Fetch latest
    fetch_all()?;
    
    // Checkout branch if not current
    let current = get_current_branch()?;
    if current != branch {
        Command::new("git")
            .args(["checkout", branch])
            .output()
            .map_err(|e| format!("Failed to checkout: {}", e))?;
    }
    
    // Pull with rebase
    pull_rebase()
}

/// Full automation: commit + push + create PR + auto-merge
pub fn full_auto_pr(
    ai_client: &crate::ai::AIClient,
    title: &str,
    base: Option<&str>,
) -> Result<String, String> {
    // 1. Stage all changes
    let _ = stage_all();
    
    // 2. Check if there are changes to commit
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to get status: {}", e))?;
    
    if String::from_utf8_lossy(&status.stdout).trim().is_empty() {
        return Ok("No changes to commit".to_string());
    }
    
    // 3. Auto-commit
    let commit_msg = auto_commit(ai_client)?;
    
    // 4. Push
    push(None, false)?;
    
    // 5. Create PR
    let pr = create_pr_ai(ai_client, title, base)?;
    
    // 6. Return PR info
    Ok(format!(
        "✅ Created PR #{}: {}\n🌐 {}\n📝 Commit: {}",
        pr.number, pr.title, pr.url, commit_msg
    ))
}

/// CI/CD pipeline status
pub fn check_ci() -> Result<String, String> {
    let repo = get_current_repo()?;
    let branch = get_current_branch()?;
    
    let output = Command::new("gh")
        .args(["run", "list", "--branch", &branch, "--limit", "1", "--json", "status,conclusion,name"])
        .output()
        .map_err(|e| format!("Failed to check CI: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("CI check failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let json: Vec<serde_json::Value> = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .unwrap_or_default();
    
    if let Some(run) = json.first() {
        let status = run["status"].as_str().unwrap_or("unknown");
        let conclusion = run["conclusion"].as_str().unwrap_or("unknown");
        let name = run["name"].as_str().unwrap_or("unknown");
        
        Ok(format!(
            "📊 CI Status for {}/{} ({}):\n   Status: {}\n   Conclusion: {}",
            repo.owner, repo.name, name, status, conclusion
        ))
    } else {
        Ok("No recent CI runs found".to_string())
    }
}

/// Update files and create PR in one command
/// This is the main "Modify, Rework, and Post to GitHub" workflow
pub fn update_and_post(
    ai_client: &crate::ai::AIClient,
    changes: Vec<(String, String, String)>, // Vec<(file, old, new)>
    pr_title: &str,
    commit_type: Option<&str>,
) -> Result<String, String> {
    // 1. Apply all changes atomically
    println!("📝 Applying {} file changes...", changes.len());
    for (file, old, new) in &changes {
        match crate::atomic_edit::run_atomic_edit(file, old, new) {
            Ok(msg) => println!("  ✅ {}: {}", if msg.contains("successful") { "Modified" } else { "Updated" }, file),
            Err(e) => eprintln!("  ⚠️  {}: {}", file, e),
        }
    }

    // 2. Stage changes
    let _ = stage_all();

    // 3. Check for changes
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to get status: {}", e))?;

    if String::from_utf8_lossy(&status.stdout).trim().is_empty() {
        return Ok("No changes to commit".to_string());
    }

    // 4. Generate commit message
    let diff = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .output()
        .map_err(|e| format!("Failed to get diff: {}", e))?;

    let diff_output = String::from_utf8_lossy(&diff.stdout).to_string();
    let prefix = commit_type.unwrap_or("feat");

    let commit_prompt = format!(
        "Generate a git commit message for:\n{}\n\n\
         Rules:\n\
         - Start with '{}:' (e.g., feat:, fix:, docs:, refactor:)\n\
         - Keep under 72 characters\n\
         - Be specific about what changed\n\
         - Output ONLY the commit message, nothing else",
        diff_output, prefix
    );

    let commit_msg = ai_client.generate_code(&commit_prompt).map(|(s, _)| s).unwrap_or_else(|e| e).trim().to_string();

    // 5. Commit
    commit(&commit_msg)?;
    println!("✅ Committed: {}", commit_msg);

    // 6. Push
    push(None, false)?;
    println!("✅ Pushed to remote");

    // 7. Create PR
    let branch = get_current_branch()?;
    let pr = create_pr_ai(ai_client, pr_title, None)?;

    Ok(format!(
        "🎉 Full workflow complete!\n\n\
         📝 Commit: {}\n\
         🌿 Branch: {}\n\
         🔀 PR #{}: {}\n\
         🔗 URL: {}",
        commit_msg, branch, pr.number, pr.title, pr.url
    ))
}

/// Quick update - edit file, commit, push, PR
pub fn quick_update(
    ai_client: &crate::ai::AIClient,
    file: &str,
    find: &str,
    replace: &str,
    pr_title: &str,
) -> Result<String, String> {
    update_and_post(
        ai_client,
        vec![(file.to_string(), find.to_string(), replace.to_string())],
        pr_title,
        None,
    )
}

/// Rebase and update PR
pub fn rebase_and_update(
    _ai_client: &crate::ai::AIClient,
    base_branch: &str,
    _pr_title: &str,
) -> Result<String, String> {
    // 1. Fetch latest
    fetch_all()?;

    // 2. Get current branch
    let current = get_current_branch()?;

    // 3. Checkout base branch and pull
    Command::new("git")
        .args(["checkout", base_branch])
        .output()
        .map_err(|e| format!("Failed to checkout: {}", e))?;

    Command::new("git")
        .args(["pull", "--rebase", "origin", base_branch])
        .output()
        .map_err(|e| format!("Failed to pull: {}", e))?;

    // 4. Checkout feature branch
    Command::new("git")
        .args(["checkout", &current])
        .output()
        .map_err(|e| format!("Failed to checkout: {}", e))?;

    // 5. Rebase onto updated base
    let rebase_result = Command::new("git")
        .args(["rebase", base_branch])
        .output()
        .map_err(|e| format!("Rebase failed: {}", e))?;

    if !rebase_result.status.success() {
        // Rebase conflict - abort and report
        Command::new("git")
            .args(["rebase", "--abort"])
            .output()
            .ok();
        return Err(format!(
            "Rebase conflict! Resolve manually:\n1. git rebase --abort\n2. git rebase {} and resolve conflicts\n3. git rebase --continue",
            base_branch
        ));
    }

    // 6. Force push (after rebase)
    push(Some(&current), true)?;
    println!("✅ Rebased and pushed");

    // 7. Check CI status
    let _repo = get_current_repo()?;
    let status = get_pr_status(0).unwrap_or(CIStatus {
        status: "unknown".to_string(),
        conclusion: Some("unknown".to_string()),
        checks: vec![],
    });

    Ok(format!(
        "🔄 Rebase complete\n📊 CI Status: {}\n🌿 Branch: {}\n✅ Ready for PR review",
        status.status, current
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let url = "https://github.com/owner/repo.git";
        let repo = parse_github_url(url).unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let url = "git@github.com:owner/repo.git";
        let repo = parse_github_url(url).unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }
}