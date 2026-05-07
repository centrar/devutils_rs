//! Viral Messages - Funny, shareable error messages like thefuck

static FUNNY_MESSAGES: &[(&str, &str)] = &[
    ("Permission denied", "🚫 sudo? sudo? I don't know her."),
    (
        "command not found",
        "❓ Maybe you meant... literally anything else?",
    ),
    ("File not found", "📁 It uploads, I swear! Check the path?"),
    (
        "cannot import",
        "📦 Did you `pip install` it? Or just dream about it?",
    ),
    (
        "undefined reference",
        "💭 You didn't define it. But your hopes and dreams are still real!",
    ),
    (
        "error: thefuck",
        "🤔 The fuck? Exactly. Just run `thefuck` (it's a real tool).",
    ),
    (
        "connection refused",
        "🚪 They're not home. Check if the server is actually running?",
    ),
    (
        "not authorized",
        "🕴️ You're not the boss of me. Actually, you're not the boss of anything.",
    ),
    (
        "timeout",
        "⏰ Table's been flipped. Your code took too long.",
    ),
    (
        "stack overflow",
        "📚 Literally. Stack Overflow has your back. Or your stack.",
    ),
    ("undefined", "🤷 It's not my code. It's not my problem."),
    (
        "NULL",
        "💩 You found the end of the universe. Congratulations.",
    ),
    ("NaN", "🧙 Not a Number? Must be magic then."),
    (
        "404",
        "🔍 Lost? The internet remembers everything. Try again.",
    ),
    (
        "500",
        "💥 My bad. Blame the devs. They won't see this anyway.",
    ),
    (
        "error: expected",
        "📝 Expecting is the first step to disappointment.",
    ),
    (
        "error: expected ;",
        "📝 Missing semicolon. It's not you, it's syntax.",
    ),
    (
        "error: expected {",
        "📝 Open braces are free. Close braces: $0.99",
    ),
    ("error: failed", "💣 And I oop- Failed again!"),
    (
        "panic",
        "😱 AAHH! PANIC! Just kidding. Unless you're on production.",
    ),
    (
        "assertion failed",
        "🤷 We made an assumption. You know what that means.",
    ),
    (
        "segfault",
        "💀 Accessing memory you don't own? Bold strategy.",
    ),
    (
        "bus error",
        "🚌 Wrong bus. Try the one going to Valid Memory Avenue.",
    ),
    ("fatal", "💀 It's over. Nothing personnel, kid."),
    ("error", "🤷 And that's the error. Have a nice day!"),
    (
        "failed to",
        "❌ We did not make it. But the internet is proud of your effort.",
    ),
    (
        "not defined",
        "🤔 You didn't define it. But your potential is limitless!",
    ),
    (
        "TypeError",
        "🔤 That's not the right type. But neither am I!",
    ),
    (
        "ReferenceError",
        "📚 Checked out from the library of valid references.",
    ),
    (
        "SyntaxError",
        "📝 The syntax is what? Syntax is my least favorite word.",
    ),
    (
        "RangeError",
        "🎯 Out of range. Just like my expectations for this code.",
    ),
    ("ECONNREFUSED", "🚪 They didn't answer. Classic ghosting."),
    (
        "ENOENT",
        "📁 Has this path ever existed? In another universe perhaps?",
    ),
    (
        "syntax error near",
        "📍 You found it! The syntax error was hiding all along.",
    ),
    (
        "unexpected token",
        "🎲 I was expecting something else. Surprise me again.",
    ),
    (
        "expected expression",
        "📝 Expressions are like good jokes. They need timing.",
    ),
    (
        "cannot find symbol",
        "🔍 Looked everywhere. Even the debugger is confused.",
    ),
    (
        "undefined variable",
        "💭 Variables are suggestions until you define them.",
    ),
    (
        "Uncaught exception",
        "🎣 You threw something and no one caught it. Embarrassing.",
    ),
    (
        "no such file",
        "📁 Maybe it exists in an alternate reality?",
    ),
    (
        "does not exist",
        "📁 It's not you. Or the file. It really doesn't exist.",
    ),
    (
        "module not found",
        "📦 Ordered online but it never arrived. Classic.",
    ),
    (
        "no module named",
        "📦 He's not here. Who? The module you asked about.",
    ),
    (
        "ImportError",
        "📦 Tried to import a friendship. Got this instead.",
    ),
];

static SUGGESTIONS: &[(&str, &str)] = &[
    (
        "not found",
        "Maybe:\n  devutils find <whatever>\n  devutils search <whatever>",
    ),
    (
        "permission denied",
        "Maybe:\n  chmod +x that_file\n  sudo ./that_file.sh",
    ),
    (
        "command not found",
        "Maybe:\n  npm install -g that-command\n  brew install that-tool",
    ),
    (
        "cannot import",
        "Maybe:\n  pip install that-library\n  npm install that-package",
    ),
    (
        "timeout",
        "Maybe:\n  Increase timeout:\n  export TIMEOUT=999999\n  Or actually fix your code",
    ),
    (
        "undefined",
        "Maybe:\n  Define it first:\n  const x = 42;\n  let y = x + 1;",
    ),
];

pub fn format_error(error: &str) -> String {
    let lower = error.to_lowercase();

    for (key, msg) in FUNNY_MESSAGES {
        if lower.contains(&key.to_lowercase()) {
            return format!("{}\n\nDid you mean: {:?}", msg, key);
        }
    }

    let mut response = format!("❌ Error: {}\n\n", error);

    for (key, help) in SUGGESTIONS {
        if lower.contains(key) {
            response.push_str(&format!("{}\n", help));
            break;
        }
    }

    response.push_str("\n💡 For smart help, try: devutils ai explain '<error message>'");

    response
}

pub fn format_suggestion(input: &str) -> String {
    let lower = input.to_lowercase();

    if lower.contains("fix") || lower.contains("bug") {
        return format!(
            "🛠️  To fix that, you'd typically:\n\
            1. Read the error message\n\
            2. Identify the root cause\n\
            3. Fix the root cause\n\
            4. Test the fix\n\
            \n\
            Or just run:\n  devutils ai fix '{}'",
            input
        );
    }

    if lower.contains("run") || lower.contains("start") || lower.contains("serve") {
        return format!(
            "🚀 To run your project:\n\
            devutils run\n\
            \nOr more specifically:\n\
            devutils run '{}'",
            input
        );
    }

    if lower.contains("test") {
        return "🧪 Testing - the good kind of stress!\n\nRun: devutils test".to_string();
    }

    if lower.contains("build") {
        return "🔨 Building - turning code into... more code?\n\nRun: devutils build".to_string();
    }

    if lower.contains("install") || lower.contains("deps") {
        return "📦 Dependencies - the only thing we all agree we need.\n\nCheck: devutils project"
            .to_string();
    }

    format!(
        "💡 For '{}', try:\n  devutils help\n  devutils project\n  devutils ai explain '{}'",
        input, input
    )
}

pub fn did_you_mean(cmd: &str) -> String {
    let known = vec![
        "search",
        "find",
        "grep",
        "ai",
        "generate",
        "explain",
        "debug",
        "test",
        "refactor",
        "complete",
        "plugin",
        "run",
        "build",
        "docker",
        "k8s",
        "system",
        "version",
        "interactive",
        "benchmark",
        "project",
        "test",
        "build",
    ];

    let cmd_lower = cmd.to_lowercase();
    let mut best_match = "";
    let mut best_score = 0;

    for known_cmd in known {
        let score = levenshtein_distance(&cmd_lower, known_cmd);
        if score > best_score && score < 4 {
            best_score = score;
            best_match = known_cmd;
        }
    }

    if best_match.is_empty() {
        return String::new();
    }

    format!("\n🤔 Did you mean: `{}`?", best_match)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for i in 0..=a_chars.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        matrix[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_chars.len()][b_chars.len()]
}

pub fn random_encouragement() -> &'static str {
    let messages = [
        "💪 You got this!",
        "🌟 Every expert was once a beginner.",
        "🔧 Code is just instructions. You're the boss.",
        "🐛 Bug hunting builds character.",
        "📚 Read the docs. Then read them again.",
        "🤔 Still here? The solution is closer than you think.",
        "☕ Coffee helps. Possibly.",
        "🧘 Breathe. Then debug.",
        "🎯 Close. Try again.",
        "🚀 To infinity and beyond your error!",
        "💡 The fix is always obvious in hindsight.",
    ];

    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize;

    messages[idx % messages.len()]
}
