//! UI Module - Interactive TUI using ratatui

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

pub fn banner() {
    show_banner()
}

pub fn show_banner() {
    println!(
        r#"
   ██████╗ ██████╗ ███████╗██╗    ██╗ ██████╗  ██████╗  █████╗ ██████╗ ██████╗ 
  ██╔════╝ ██╔══██╗██╔════╝██║    ██║██╔════╝ ██╔═══██╗██╔══██╗██╔══██╗██╔══██╗
  ██║  ███╗██████╔╝█████╗  ██║ █╗ ██║      ██║   ██║███████║██████╔╝██████╔╝
  ██║   ██║██╔══██╗██╔══╝  ██║███╗██║      ██║   ██║██╔══██║██╔══██╗██╔══██╗
  ╚██████╔╝██║  ██║███████╗╚██╔╝ ██║╚██████╗ ╚██████╔╝██║  ██║██║  ██║██║  ██║
   ╚═════╝ ╚═╝  ╚═╝╚══════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝

    The FASTEST AI-Powered Developer Toolkit
"#
    );
}

pub fn menu() {
    show_help()
}

pub fn show_help() {
    println!(
        r#"
Usage: devutils <command> [options]

Commands:
  search <query>      AI semantic code search
    --semantic     Use AI to understand intent
    -t <type>    File type (py, js, rs)
  
  find <pattern>    Find files by name
  grep <pattern>   Fast grep replacement
  
  ai explain <code>   Explain code
  ai generate <prompt> Generate code
  ai debug <code>     Fix bugs
  ai tests <code>    Generate tests
  
  git status        Quick git status
  git commits [n]   Recent commits
  git branches      List branches
  
  system           System info
  local-ip         Get local IP
  port <n>        Check port availability
  
  interactive      Interactive TUI mode

Options:
  --no-color      Disable colors
  --verbose      Show timing
  --benchmark    Run performance benchmarks

Examples:
  devutils search "authentication" --semantic
  devutils ai explain "def fib(n): return n"
  devutils find "*.py"
  devutils grep "TODO"
  devutils git status
  devutils interactive
"#
    );
}

pub fn print_system_info() -> Result<()> {
    let os_type = sys_info::os_type().unwrap_or_else(|_| "unknown".to_string());
    let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());
    let cpu_num = sys_info::cpu_num().unwrap_or(0);
    let mem = sys_info::mem_info().unwrap_or_else(|_| sys_info::MemInfo {
        total: 0,
        free: 0,
        avail: 0,
        buffers: 0,
        cached: 0,
        swap_total: 0,
        swap_free: 0,
    });

    println!(
        "\n\x1b[36m🖥️  System Info\x1b[0m\n\
        \x1b[90mOS:\x1b[0m {}\n\
        \x1b[90mHostname:\x1b[0m {}\n\
        \x1b[90mCPU Cores:\x1b[0m {}\n\
        \x1b[90mMemory:\x1b[0m {} MB",
        os_type,
        hostname,
        cpu_num,
        mem.total / 1024 / 1024
    );

    Ok(())
}

pub fn interactive_mode() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_result = run_interactive_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    app_result
}

fn run_interactive_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut selected = 0_usize;
    let commands = vec![
        "🔍 Search",
        "📁 Find",
        "🔬 Grep",
        "🤖 AI Chat",
        "📊 Git",
        "🐳 Docker",
        "☸ Kubernetes",
        "📈 Benchmark",
        "❓ Help",
    ];

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);

            let title = Paragraph::new("DevUtils - The Fastest AI-Powered Developer Toolkit")
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            let menu_items: Vec<ListItem> = commands
                .iter()
                .enumerate()
                .map(|(i, cmd)| {
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(*cmd).style(style)
                })
                .collect();

            let menu = List::new(menu_items)
                .block(Block::default().borders(Borders::ALL).title("Commands"))
                .highlight_style(Style::default().fg(Color::Cyan))
                .highlight_symbol(">> ");
            f.render_widget(menu, chunks[1]);

            let footer = Paragraph::new("↑↓ Navigate | Enter Select | Esc Quit")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(footer, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    selected = (selected + 1).min(commands.len() - 1);
                }
                KeyCode::Enter => {
                    println!("\n\x1b[32mSelected: {}\x1b[0m", commands[selected]);
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
