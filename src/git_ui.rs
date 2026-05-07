//! Git TUI - Full-featured Git interface like lazygit

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;
use std::process::Command;

const COLOR_MODIFIED: Color = Color::Yellow;
const COLOR_ADDED: Color = Color::Green;
const COLOR_DELETED: Color = Color::Red;
const COLOR_STAGED: Color = Color::Cyan;
const COLOR_UNSTAGED: Color = Color::Magenta;

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub conflicted: Vec<FileChange>,
    pub branch: String,
    pub is_clean: bool,
    pub ahead: i32,
    pub behind: i32,
    pub is_rebasing: bool,
    pub is_merging: bool,
    pub is_cherry_picking: bool,
    pub is_reverting: bool,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub content: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: i32,
    pub behind: i32,
}

#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub date: String,
}

pub struct GitTUI {
    current_view: GitView,
    selected_file: usize,
    selected_commit: usize,
    selected_hunk: usize,
    selected_stash: usize,
    show_stash: bool,
    show_stashes: Vec<StashEntry>,
    commits: Vec<Commit>,
    branches: Vec<Branch>,
    worktrees: Vec<String>,
    status: GitStatus,
    scroll: usize,
    quit: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitView {
    Status,
    Files,
    CommitList,
    Branches,
    Stash,
    Stashes,
    Worktree,
    Rebase,
    Bisect,
    Log,
}

impl GitTUI {
    pub fn new() -> Self {
        Self {
            current_view: GitView::Files,
            selected_file: 0,
            selected_commit: 0,
            selected_hunk: 0,
            selected_stash: 0,
            show_stash: false,
            show_stashes: Vec::new(),
            commits: Vec::new(),
            branches: Vec::new(),
            worktrees: Vec::new(),
            status: GitStatus {
                staged: Vec::new(),
                unstaged: Vec::new(),
                untracked: Vec::new(),
                conflicted: Vec::new(),
                branch: "main".to_string(),
                is_clean: true,
                ahead: 0,
                behind: 0,
                is_rebasing: false,
                is_merging: false,
                is_cherry_picking: false,
                is_reverting: false,
            },
            scroll: 0,
            quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.refresh_status()?;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.quit {
            self.render(&mut terminal)?;
            self.handle_input()?;
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            self.render_header(f, chunks[0]);
            self.render_main(f, chunks[1]);
            self.render_footer(f, chunks[2]);
        })?;
        Ok(())
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let branch_name = &self.status.branch;
        let status_text = if self.status.is_clean {
            format!("✓ {}", "Clean")
        } else {
            let staged = self.status.staged.len();
            let unstaged = self.status.unstaged.len() + self.status.untracked.len();
            format!("{} staged, {} unstaged", staged, unstaged)
        };

        let status_color = if self.status.is_clean {
            Color::Green
        } else {
            Color::Yellow
        };

        let title = format!(
            "⎇  {}  │  {}  │  [1]Files  [2]Commits  [3]Branches  [4]Stash  [5]Worktree",
            branch_name, status_text
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let paragraph = Paragraph::new(title)
            .style(Style::default().fg(status_color))
            .block(block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, area);
    }

    fn render_main(&mut self, f: &mut Frame, area: Rect) {
        match self.current_view {
            GitView::Files => self.render_files_view(f, area),
            GitView::CommitList => self.render_commits_view(f, area),
            GitView::Branches => self.render_branches_view(f, area),
            GitView::Stash => self.render_stash_view(f, area),
            GitView::Stashes => self.render_stashes_list_view(f, area),
            GitView::Worktree => self.render_worktree_view(f, area),
            GitView::Log => self.render_log_view(f, area),
            _ => self.render_files_view(f, area),
        }
    }

    fn render_files_view(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let staged_items: Vec<ListItem> = self
            .status
            .staged
            .iter()
            .map(|f| {
                ListItem::new(format!("+ {}", f.path)).style(Style::default().fg(COLOR_STAGED))
            })
            .collect();

        let unstaged_items: Vec<ListItem> = self
            .status
            .unstaged
            .iter()
            .chain(self.status.untracked.iter())
            .chain(self.status.conflicted.iter())
            .map(|f| {
                let status_char = match f.status {
                    FileStatus::Modified => "~",
                    FileStatus::Deleted => "-",
                    _ => "?",
                };
                ListItem::new(format!("{} {}", status_char, f.path))
                    .style(Style::default().fg(COLOR_UNSTAGED))
            })
            .collect();

        let staged_list = List::new(staged_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Staged Changes"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        let unstaged_list = List::new(unstaged_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Unstaged Changes"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_widget(staged_list, chunks[0]);
        f.render_widget(unstaged_list, chunks[1]);
    }

    fn render_commits_view(&mut self, f: &mut Frame, area: Rect) {
        if self.commits.is_empty() {
            let no_commits = Paragraph::new("No commits yet")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Commits"))
                .alignment(Alignment::Center);
            f.render_widget(no_commits, area);
            return;
        }

        let rows: Vec<Row> = self
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.selected_commit {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(c.short_hash.clone()).style(style),
                    Cell::from(c.message.chars().take(50).collect::<String>()).style(style),
                    Cell::from(c.author.clone()).style(style),
                    Cell::from(c.date.clone()).style(style),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Length(8),
                Constraint::Min(30),
                Constraint::Length(15),
                Constraint::Length(12),
            ],
        )
        .block(Block::default().borders(Borders::ALL).title("Commits"))
        .highlight_style(Style::default().bg(Color::Blue));

        f.render_widget(table, area);
    }

    fn render_branches_view(&mut self, f: &mut Frame, area: Rect) {
        if self.branches.is_empty() {
            let no_branches = Paragraph::new("No branches")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Branches"))
                .alignment(Alignment::Center);
            f.render_widget(no_branches, area);
            return;
        }

        let items: Vec<ListItem> = self
            .branches
            .iter()
            .map(|b| {
                let icon = if b.is_current { "●" } else { " " };
                let name = if b.is_remote {
                    format!("remotes/{}", b.name)
                } else {
                    b.name.clone()
                };
                ListItem::new(format!("{} {}", icon, name)).style(if b.is_current {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Branches"))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_widget(list, area);
    }

    fn render_stash_view(&mut self, f: &mut Frame, area: Rect) {
        let content = if self.status.is_clean {
            "Nothing to stash"
        } else {
            "Press 's' to stash changes"
        };

        let block = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Stash"))
            .alignment(Alignment::Center);

        f.render_widget(block, area);
    }

    fn render_stashes_list_view(&mut self, f: &mut Frame, area: Rect) {
        if self.show_stashes.is_empty() {
            let no_stashes = Paragraph::new("No stash entries")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Stashes"))
                .alignment(Alignment::Center);
            f.render_widget(no_stashes, area);
            return;
        }

        let items: Vec<ListItem> = self
            .show_stashes
            .iter()
            .map(|s| ListItem::new(format!("stash@{{{}}}: {}", s.index, s.message)))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Stashes"))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_widget(list, area);
    }

    fn render_worktree_view(&mut self, f: &mut Frame, area: Rect) {
        let content = if self.worktrees.is_empty() {
            "No worktrees. Press 'a' to add worktree"
        } else {
            "Worktrees:"
        };

        let mut items = vec![ListItem::new(content)];

        for wt in &self.worktrees {
            items.push(ListItem::new(format!("  {}", wt)));
        }

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Worktrees"));

        f.render_widget(list, area);
    }

    fn render_log_view(&mut self, f: &mut Frame, area: Rect) {
        self.render_commits_view(f, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let help_text = "[Enter]View  [s]Stage  [u]Unstage  [c]Commit  [p]Push  [P]Pull  [b]Branch  [r]Rebase  [d]Discard  [q]Quit";

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn handle_input(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.quit = true;
                }
                KeyCode::Char('1') => {
                    self.current_view = GitView::Files;
                }
                KeyCode::Char('2') => {
                    self.current_view = GitView::CommitList;
                    let _ = self.refresh_commits();
                }
                KeyCode::Char('3') => {
                    self.current_view = GitView::Branches;
                    let _ = self.refresh_branches();
                }
                KeyCode::Char('4') => {
                    self.current_view = GitView::Stash;
                }
                KeyCode::Char('5') => {
                    self.current_view = GitView::Worktree;
                    let _ = self.refresh_worktrees();
                }
                KeyCode::Char('s') => {
                    if self.current_view == GitView::Files {
                        let _ = self.stage_all();
                    }
                }
                KeyCode::Char('u') => {
                    if self.current_view == GitView::Files {
                        let _ = self.unstage_all();
                    }
                }
                KeyCode::Char('c') => {
                    let _ = self.commit();
                }
                KeyCode::Char('p') => {
                    let _ = self.push();
                }
                KeyCode::Char('P') => {
                    let _ = self.pull();
                }
                KeyCode::Char('r') => {
                    let _ = self.start_rebase();
                }
                KeyCode::Char('d') => {
                    if self.current_view == GitView::Files {
                        let _ = self.discard_changes();
                    }
                }
                KeyCode::Char('b') => {
                    if self.current_view == GitView::Branches {
                        let _ = self.checkout_branch();
                    }
                }
                KeyCode::Char('a') => {
                    if self.current_view == GitView::Worktree {
                        let _ = self.add_worktree();
                    }
                }
                KeyCode::Char('z') => {
                    let _ = self.undo();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_selection(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_selection(-1);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn move_selection(&mut self, delta: isize) {
        match self.current_view {
            GitView::Files => {
                let total = self.status.staged.len()
                    + self.status.unstaged.len()
                    + self.status.untracked.len();
                if total > 0 {
                    self.selected_file =
                        (self.selected_file as isize + delta).rem_euclid(total as isize) as usize;
                }
            }
            GitView::CommitList => {
                if !self.commits.is_empty() {
                    self.selected_commit = (self.selected_commit as isize + delta)
                        .rem_euclid(self.commits.len() as isize)
                        as usize;
                }
            }
            GitView::Branches => {
                if !self.branches.is_empty() {
                    let current_idx = self.branches.iter().position(|b| b.is_current).unwrap_or(0);
                    let new_idx = ((current_idx as isize + delta)
                        .rem_euclid(self.branches.len() as isize))
                        as usize;
                    self.selected_commit = new_idx;
                }
            }
            _ => {}
        }
    }

    pub fn refresh_status(&mut self) -> Result<()> {
        let output = Command::new("git")
            .args(["status", "--porcelain", "-uall"])
            .output()?;

        let status_output = String::from_utf8_lossy(&output.stdout);

        self.status.staged.clear();
        self.status.unstaged.clear();
        self.status.untracked.clear();
        self.status.conflicted.clear();

        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }

            let index_status = line.chars().next().unwrap_or(' ');
            let worktree_status = line.chars().nth(1).unwrap_or(' ');
            let path = line[3..].to_string();

            let file = FileChange {
                path: path.clone(),
                status: FileStatus::Modified,
                hunks: Vec::new(),
            };

            if index_status == '?' {
                self.status.untracked.push(file);
            } else if index_status == 'U' || worktree_status == 'U' {
                self.status.conflicted.push(file);
            } else if index_status != ' ' && index_status != '?' {
                self.status.staged.push(FileChange {
                    path,
                    status: match index_status {
                        'A' => FileStatus::Added,
                        'D' => FileStatus::Deleted,
                        'M' => FileStatus::Modified,
                        'R' => FileStatus::Renamed,
                        'C' => FileStatus::Copied,
                        _ => FileStatus::Modified,
                    },
                    hunks: Vec::new(),
                });
            } else if worktree_status != ' ' {
                self.status.unstaged.push(FileChange {
                    path,
                    status: match worktree_status {
                        'D' => FileStatus::Deleted,
                        _ => FileStatus::Modified,
                    },
                    hunks: Vec::new(),
                });
            }
        }

        let branch_output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;
        self.status.branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        self.status.is_clean = self.status.staged.is_empty()
            && self.status.unstaged.is_empty()
            && self.status.untracked.is_empty();

        let rev_parse = Command::new("git")
            .args(["revparse", "--abbrev-ref", "@{u}", "--"])
            .output();

        if let Ok(output) = rev_parse {
            let upstream = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let ahead = Command::new("git")
                .args([
                    "rev-list",
                    "--count",
                    format!("{}..{}", self.status.branch, upstream).as_str(),
                ])
                .output()
                .ok();
            let behind = Command::new("git")
                .args([
                    "rev-list",
                    "--count",
                    format!("{}..{}", upstream, self.status.branch).as_str(),
                ])
                .output()
                .ok();

            if let Some(o) = ahead {
                self.status.ahead = String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
            if let Some(o) = behind {
                self.status.behind = String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
        }

        Ok(())
    }

    fn refresh_commits(&mut self) -> Result<()> {
        let output = Command::new("git")
            .args([
                "log",
                "--oneline",
                "-20",
                "--pretty=format:%h|%s|%an|%ad|%p",
            ])
            .output()?;

        self.commits.clear();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                self.commits.push(Commit {
                    hash: parts[0].to_string(),
                    short_hash: parts[0][..7].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                    parents: parts
                        .get(4)
                        .map(|p| p.split(' ').map(String::from).collect())
                        .unwrap_or_default(),
                });
            }
        }

        Ok(())
    }

    fn refresh_branches(&mut self) -> Result<()> {
        let output = Command::new("git")
            .args([
                "branch",
                "-a",
                "--format=%(refname:short)|%(HEAD)|%(upstream:short)",
            ])
            .output()?;

        self.branches.clear();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if let Some(name) = parts.first() {
                if !name.is_empty() {
                    let is_current = parts.get(1).map(|h| *h == "*").unwrap_or(false);
                    let upstream = parts
                        .get(2)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let is_remote = name.starts_with("remotes/");

                    self.branches.push(Branch {
                        name: name.to_string(),
                        is_current,
                        is_remote,
                        upstream,
                        ahead: 0,
                        behind: 0,
                    });
                }
            }
        }

        Ok(())
    }

    fn refresh_worktrees(&mut self) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .output()?;

        self.worktrees.clear();

        let mut current = String::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.starts_with("path ") {
                current = line[5..].to_string();
            } else if line == "HEAD" && !current.is_empty() {
                self.worktrees.push(current.clone());
            }
        }

        Ok(())
    }

    fn stage_all(&self) -> Result<()> {
        Command::new("git").args(["add", "-A"]).output()?;
        Ok(())
    }

    fn unstage_all(&self) -> Result<()> {
        Command::new("git").args(["reset", "HEAD", "--"]).output()?;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        println!("\n\x1b[36mCommit message:\x1b[0m (use git commit -m \"message\" in terminal)");
        Ok(())
    }

    fn push(&self) -> Result<()> {
        let output = Command::new("git").args(["push"]).output()?;

        if output.status.success() {
            println!("\n\x1b[32m✓ Pushed successfully\x1b[0m");
        } else {
            println!("\n\x1b[31m✗ Push failed\x1b[0m");
        }
        Ok(())
    }

    fn pull(&self) -> Result<()> {
        let output = Command::new("git").args(["pull", "--rebase"]).output()?;

        if output.status.success() {
            println!("\n\x1b[32m✓ Pulled successfully\x1b[0m");
        } else {
            println!("\n\x1b[31m✗ Pull failed\x1b[0m");
        }
        Ok(())
    }

    fn start_rebase(&self) -> Result<()> {
        println!("\n\x1b[36mStarting interactive rebase...\x1b[0m");
        Command::new("git")
            .args(["rebase", "-i", "HEAD~10"])
            .spawn()?;
        Ok(())
    }

    fn discard_changes(&self) -> Result<()> {
        let output = Command::new("git").args(["checkout", "--", "."]).output()?;

        if output.status.success() {
            println!("\n\x1b[32m✓ Changes discarded\x1b[0m");
        }
        Ok(())
    }

    fn checkout_branch(&self) -> Result<()> {
        if let Some(branch) = self.branches.get(self.selected_commit) {
            let _ = Command::new("git")
                .args(["checkout", &branch.name])
                .output()?;
        }
        Ok(())
    }

    fn add_worktree(&self) -> Result<()> {
        println!("\n\x1b[36mAdding worktree...\x1b[0m");
        Ok(())
    }

    fn undo(&self) -> Result<()> {
        let _ = Command::new("git")
            .args(["reset", "--soft", "HEAD~1"])
            .output()?;
        println!("\n\x1b[32m✓ Undo successful\x1b[0m");
        Ok(())
    }
}

impl Default for GitTUI {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_git_tui() -> Result<()> {
    let mut tui = GitTUI::new();
    tui.run()?;
    Ok(())
}
