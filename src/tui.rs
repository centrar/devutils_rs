//! Rich Git TUI - LazyGit-style interface
//!
//! Features:
//! - File panel with staged/unstaged changes
//! - Branch panel with checkout
//! - Commit history panel
//! - Commit creation with staging
//! - Stash management
//! - Keybindings similar to lazygit

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use once_cell::sync::Lazy;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

static GIT_TUI_STATE: Lazy<Mutex<GitTuiState>> = Lazy::new(|| Mutex::new(GitTuiState::new()));

#[derive(Debug, Clone)]
pub struct GitTuiState {
    pub files: Vec<GitFile>,
    pub branches: Vec<String>,
    pub commits: Vec<GitCommit>,
    pub stashed: Vec<String>,
    pub selected_file: usize,
    pub selected_branch: usize,
    pub selected_commit: usize,
    pub active_panel: Panel,
    pub view: View,
    pub input_buffer: String,
    pub showing_commit_dialog: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Files,
    Branches,
    Commits,
    Stash,
    Dialog,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    WorkingTree,
    Branches,
    Commits,
    Stash,
    Diff,
}

#[derive(Debug, Clone)]
pub struct GitFile {
    pub path: String,
    pub status: FileStatus,
    pub staged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub time: String,
}

impl GitTuiState {
    fn new() -> Self {
        let mut state = Self {
            files: Vec::new(),
            branches: Vec::new(),
            commits: Vec::new(),
            stashed: Vec::new(),
            selected_file: 0,
            selected_branch: 0,
            selected_commit: 0,
            active_panel: Panel::Files,
            view: View::WorkingTree,
            input_buffer: String::new(),
            showing_commit_dialog: false,
        };
        state.refresh();
        state
    }

    fn refresh(&mut self) {
        self.files = get_modified_files();
        self.branches = get_branches();
        self.commits = get_commits();
        self.stashed = get_stash();
    }
}

fn get_modified_files() -> Vec<GitFile> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "-uall"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.len() > 2)
            .map(|l| {
                let status = l.chars().next().unwrap_or(' ');
                let staged = l.chars().nth(1).unwrap_or(' ') != ' ';
                let path = l[3..].to_string();
                let file_status = match status {
                    'M' => FileStatus::Modified,
                    'A' => FileStatus::Added,
                    'D' => FileStatus::Deleted,
                    'R' => FileStatus::Renamed,
                    '?' => FileStatus::Untracked,
                    _ => FileStatus::Modified,
                };
                GitFile {
                    path,
                    status: file_status,
                    staged,
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn get_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn get_commits() -> Vec<GitCommit> {
    let output = Command::new("git")
        .args(["log", "--oneline", "-n", "50", "--format=%h|%s|%an|%ar"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|l| {
                let parts: Vec<&str> = l.split('|').collect();
                if parts.len() >= 4 {
                    Some(GitCommit {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        author: parts[2].to_string(),
                        time: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn get_stash() -> Vec<String> {
    let output = Command::new("git")
        .args(["stash", "list", "--format=%s"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

pub fn run_git_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = git_tui_loop(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

pub fn run_tui() -> Result<()> {
    run_git_tui()
}

fn git_tui_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut state = GIT_TUI_STATE.lock().unwrap();

    loop {
        terminal.draw(|f| render_git_ui(f, &state))?;

        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;

            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Tab => {
                        state.active_panel = match state.active_panel {
                            Panel::Files => Panel::Branches,
                            Panel::Branches => Panel::Commits,
                            Panel::Commits => Panel::Stash,
                            Panel::Stash => Panel::Files,
                            Panel::Dialog => Panel::Dialog,
                        };
                    }
                    KeyCode::Char('j') | KeyCode::Down => match state.active_panel {
                        Panel::Files
                            if state.selected_file < state.files.len().saturating_sub(1) =>
                        {
                            state.selected_file += 1;
                        }
                        Panel::Branches
                            if state.selected_branch < state.branches.len().saturating_sub(1) =>
                        {
                            state.selected_branch += 1;
                        }
                        Panel::Commits
                            if state.selected_commit < state.commits.len().saturating_sub(1) =>
                        {
                            state.selected_commit += 1;
                        }
                        _ => {}
                    },
                    KeyCode::Char('k') | KeyCode::Up => match state.active_panel {
                        Panel::Files if state.selected_file > 0 => {
                            state.selected_file -= 1;
                        }
                        Panel::Branches if state.selected_branch > 0 => {
                            state.selected_branch -= 1;
                        }
                        Panel::Commits if state.selected_commit > 0 => {
                            state.selected_commit -= 1;
                        }
                        _ => {}
                    },
                    KeyCode::Char('s') => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            // Stage selected file
                            if let Some(file) = state.files.get(state.selected_file) {
                                let _ = Command::new("git").args(["add", &file.path]).output();
                                state.refresh();
                            }
                        } else {
                            // Open stash
                            state.showing_commit_dialog = true;
                            state.input_buffer.clear();
                        }
                    }
                    KeyCode::Char('c') => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            break;
                        }
                    }
                    KeyCode::Enter => {
                        if state.showing_commit_dialog {
                            // Create commit
                            if !state.input_buffer.is_empty() {
                                let _ = Command::new("git")
                                    .args(["commit", "-m", &state.input_buffer])
                                    .output();
                                state.refresh();
                            }
                            state.showing_commit_dialog = false;
                            state.input_buffer.clear();
                        }
                    }
                    KeyCode::Char('l') => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            // View log
                            state.view = View::Commits;
                        }
                    }
                    KeyCode::Char('o') | KeyCode::Char('b') => {
                        // Open/ checkout branch
                        if let Some(branch) = state.branches.get(state.selected_branch) {
                            let _ = Command::new("git").args(["checkout", branch]).output();
                            state.refresh();
                        }
                    }
                    KeyCode::Char('r') => {
                        // Refresh
                        state.refresh();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn render_git_ui(f: &mut Frame, state: &GitTuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new(" DevUtils Git TUI (like lazygit) ").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    // Main content - 3 columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    // Files panel
    render_files_panel(f, main_chunks[0], state);

    // Diff panel
    render_diff_panel(f, main_chunks[1], state);

    // Branches/Commits panel
    render_info_panel(f, main_chunks[2], state);

    // Status bar
    let status = format!(" [Tab] Switch panel | ↑↓ Navigate | s Stage | c Commit | q Quit ");
    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status_bar, chunks[2]);
}

fn render_files_panel(f: &mut Frame, area: Rect, state: &GitTuiState) {
    let title = match state.active_panel {
        Panel::Files => " Changes ",
        Panel::Branches => " Branches ",
        Panel::Commits => " Commits ",
        _ => " Panel ",
    };

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let icon = match file.status {
                FileStatus::Modified => "M",
                FileStatus::Added => "A",
                FileStatus::Deleted => "D",
                FileStatus::Renamed => "R",
                FileStatus::Untracked => "?",
            };
            let marker = if state.active_panel == Panel::Files && i == state.selected_file {
                "▶"
            } else {
                " "
            };
            let content = format!("{} [{}] {}", marker, icon, file.path);
            ListItem::new(content).style(
                if state.active_panel == Panel::Files && i == state.selected_file {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                },
            )
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

    f.render_widget(list, area);
}

fn render_diff_panel(f: &mut Frame, area: Rect, state: &GitTuiState) {
    let selected = state.files.get(state.selected_file);
    let diff_content = if let Some(file) = selected {
        let output = Command::new("git").args(["diff", &file.path]).output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => "No diff available".to_string(),
        }
    } else {
        "Select a file to see diff".to_string()
    };

    let diff = Paragraph::new(diff_content)
        .block(Block::default().title(" Diff ").borders(Borders::ALL))
        .scroll((state.selected_file as u16, 0));

    f.render_widget(diff, area);
}

fn render_info_panel(f: &mut Frame, area: Rect, state: &GitTuiState) {
    let mut lines = vec![];

    // Branches
    lines.push(Span::raw(""));
    lines.push(Span::styled(
        " BRANCHES ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    for (i, branch) in state.branches.iter().enumerate().take(10) {
        let marker = if state.active_panel == Panel::Branches && i == state.selected_branch {
            "▶ "
        } else {
            "  "
        };
        lines.push(Span::raw(format!("{}{}", marker, branch)));
    }

    // Commits
    lines.push(Span::raw(""));
    lines.push(Span::styled(
        " RECENT COMMITS ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    for (i, commit) in state.commits.iter().enumerate().take(10) {
        let marker = if state.active_panel == Panel::Commits && i == state.selected_commit {
            "▶ "
        } else {
            "  "
        };
        lines.push(Span::raw(format!(
            "{}{} {}",
            marker, commit.hash, commit.message
        )));
    }

    let content = Line::from(lines);
    let info =
        Paragraph::new(content).block(Block::default().title(" Info ").borders(Borders::ALL));

    f.render_widget(info, area);
}
