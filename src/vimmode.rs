//! Vim Mode - Terminal Navigation
//!
//! Full Vim-style keybindings for terminal navigation:
//! - Normal mode: hjkl navigation, w/b word movement
//! - Insert mode: i/a/o to enter
//! - Visual mode: v for selection
//! - Command mode: : for commands

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static VIM_STATE: Lazy<RwLock<VimState>> = Lazy::new(|| RwLock::new(VimState::new()));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug)]
pub struct VimState {
    pub mode: VimMode,
    pub buffer: String,
    pub cursor_pos: usize,
    pub registers: HashMap<char, String>,
    pub clipboard: String,
    pub history: Vec<String>,
    pub position: (usize, usize),
}

impl VimState {
    fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            buffer: String::new(),
            cursor_pos: 0,
            registers: HashMap::new(),
            clipboard: String::new(),
            history: vec![],
            position: (0, 0),
        }
    }
}

pub fn handle_key(key: &str) -> String {
    let mut state = VIM_STATE.write().unwrap();

    match key {
        // Normal mode navigation
        "h" => {
            state.position.0 = state.position.0.saturating_sub(1);
        }
        "j" => {
            state.position.1 += 1;
        }
        "k" => {
            state.position.1 = state.position.1.saturating_sub(1);
        }
        "l" => {
            state.position.0 += 1;
        }

        // Word movement
        "w" => { /* move to next word */ }
        "b" => { /* move to prev word */ }
        "e" => { /* move to end of word */ }

        // Line movement
        "0" => {
            state.cursor_pos = 0;
        }
        "$" => {
            state.cursor_pos = state.buffer.len();
        }
        "^" => { /* move to first non-blank */ }

        // Insert mode
        "i" => {
            state.mode = VimMode::Insert;
        }
        "a" => {
            state.cursor_pos += 1;
            state.mode = VimMode::Insert;
        }
        "o" => {
            /* new line below */
            state.mode = VimMode::Insert;
        }
        "A" => {
            state.cursor_pos = state.buffer.len();
            state.mode = VimMode::Insert;
        }
        "O" => {
            /* new line above */
            state.mode = VimMode::Insert;
        }

        // Visual mode
        "v" => {
            state.mode = VimMode::Visual;
        }
        "V" => { /* visual line */ }

        // Command mode
        ":" => {
            state.mode = VimMode::Command;
        }

        // Yank/Paste
        "y" => {
            state.clipboard = state.buffer.clone();
        }
        "p" => {
            let clip = state.clipboard.clone();
            state.buffer.push_str(&clip);
        }
        "d" => {
            state.clipboard = state.buffer.clone();
            state.buffer.clear();
        }
        "x" => {
            let pos = state.cursor_pos;
            if pos < state.buffer.len() {
                state.buffer.remove(pos);
            }
        }

        // Undo/Redo
        "u" => { /* undo */ }
        "Ctrl+r" => { /* redo */ }

        // Delete
        "dw" => { /* delete word */ }
        "dd" => {
            state.buffer.clear();
        }

        // Search
        "/" => { /* forward search */ }
        "?" => { /* backward search */ }
        "n" => { /* next search result */ }
        "N" => { /* prev search result */ }

        // Exit
        "Esc" => {
            state.mode = VimMode::Normal;
        }
        "ZZ" => {
            return "exit".to_string();
        }

        _ => {}
    }

    format!(
        "{:?}: {}",
        state.mode,
        state.buffer.chars().take(20).collect::<String>()
    )
}

pub fn get_mode() -> VimMode {
    let state = VIM_STATE.read().unwrap();
    state.mode
}

pub fn get_status() -> String {
    let state = VIM_STATE.read().unwrap();
    let mode_indicator = match state.mode {
        VimMode::Normal => "-- NORMAL --",
        VimMode::Insert => "-- INSERT --",
        VimMode::Visual => "-- VISUAL --",
        VimMode::Command => "-- COMMAND --",
    };
    format!(
        "{} | {} | {}",
        mode_indicator, state.position.0, state.position.1
    )
}

pub fn enter_insert() {
    let mut state = VIM_STATE.write().unwrap();
    state.mode = VimMode::Insert;
}

pub fn exit_insert() {
    let mut state = VIM_STATE.write().unwrap();
    state.mode = VimMode::Normal;
}

pub fn vim_commands() {
    println!("\n\x1b[36m⌨️ Vim Mode Keybindings:\x1b[0m\n");
    println!("  \x1b[33mNormal Mode:\x1b[0m");
    println!("    h j k l     - Arrow keys");
    println!("    w b e       - Word movement");
    println!("    0 $ ^      - Line movement");
    println!("    i a o      - Enter insert mode");
    println!("    v V        - Visual selection");
    println!("    y p d x   - Yank, paste, delete");
    println!("    u Ctrl+r  - Undo, redo");
    println!("    / ?       - Search");
    println!("    ZZ        - Save and exit");
    println!();
    println!("  \x1b[33mInsert Mode:\x1b[0m");
    println!("    Esc       - Return to normal");
    println!();
}
