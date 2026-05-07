//! Context Manager - Token tracking and context compaction

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub static CONTEXT: Lazy<Mutex<ContextManager>> = Lazy::new(|| Mutex::new(ContextManager::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    pub messages: VecDeque<ContextMessage>,
    pub token_count: usize,
    pub total_tokens_used: u64,
    pub total_cost_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub tokens: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            token_count: 0,
            total_tokens_used: 0,
            total_cost_cents: 0,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        let tokens = estimate_tokens(content);
        self.messages.push_back(ContextMessage {
            role: role.to_string(),
            content: content.to_string(),
            tokens,
        });
        self.token_count += tokens;
        self.total_tokens_used += tokens as u64;
    }

    pub fn compact(&mut self) -> usize {
        let initial = self.messages.len();
        if self.messages.len() > 5 {
            let keep: Vec<_> = self.messages.iter().rev().take(5).cloned().collect();
            let mut new_deque = VecDeque::new();
            for msg in keep.into_iter().rev() {
                new_deque.push_front(msg);
            }
            self.messages = new_deque;
        }
        initial.saturating_sub(self.messages.len())
    }

    pub fn get_usage(&self) -> (usize, u64, u64) {
        (
            self.token_count,
            self.total_tokens_used,
            self.total_cost_cents,
        )
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.token_count = 0;
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn get_context_status() -> String {
    let ctx = CONTEXT.lock().unwrap();
    format!(
        "Status: {} messages, {} tokens used, {} total tokens",
        ctx.messages.len(),
        ctx.token_count,
        ctx.total_tokens_used
    )
}

pub fn get_context_summary() -> String {
    let ctx = CONTEXT.lock().unwrap();
    format!(
        "Messages: {} | Tokens: {}",
        ctx.messages.len(),
        ctx.token_count
    )
}

pub fn compact_context() -> String {
    let removed = CONTEXT.lock().unwrap().compact();
    format!("Compacted {} messages", removed)
}

pub fn show_usage() -> String {
    let (tokens, total, _cost) = CONTEXT.lock().unwrap().get_usage();
    let mut result = String::new();
    result.push_str("\n\x1b[36m📊 Token Usage\x1b[0m\n\n");
    result.push_str(&format!("Current: {} tokens\n", tokens));
    result.push_str(&format!("Total: {} tokens\n", total));
    result
}

pub fn context_commands() {
    println!("\n\x1b[36m📊 Context Manager\x1b[0m");
    println!("\nUsage:");
    println!("  devutils context summary");
    println!("  devutils context compact");
    println!("  devutils context usage");
    println!("  devutils context clear");
}
