//! Fast Rendering - Zed-style smooth terminal output

pub fn init_fast_mode() {
    println!("⚡ Fast render mode enabled");
}

pub fn clear_screen() {
    println!("\x1b[2J\x1b[H");
}

pub fn save_cursor() {
    print!("\x1b[s");
}

pub fn restore_cursor() {
    print!("\x1b[u");
}

pub fn hide_cursor() {
    print!("\x1b[?25l");
}

pub fn show_cursor() {
    print!("\x1b[?25h");
}

pub fn move_cursor(x: u16, y: u16) {
    print!("\x1b[{};{}H", y + 1, x + 1);
}

pub fn animate_loading() -> &'static str {
    "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
}

pub fn spin_char(frame: usize) -> char {
    let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    chars[frame % chars.len()]
}

pub struct ProgressBar {
    current: usize,
    total: usize,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self { current: 0, total, width: 30 }
    }
    
    pub fn set(&mut self, value: usize) {
        self.current = value;
        self.render();
    }
    
    pub fn render(&self) {
        let pct = self.current as f32 / self.total.max(1) as f32;
        let filled = (pct * self.width as f32) as usize;
        
        println!(
            "\r[{}{}] {:.0}%",
            "█".repeat(filled),
            "░".repeat(self.width - filled),
            pct * 100.0
        );
    }
}