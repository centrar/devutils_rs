use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Read, BufReader, BufRead};
use memmap2::MmapOptions;
use rayon::prelude::*;
use sysinfo::{System, Disks, Networks};
use ignore::WalkBuilder;
use crossbeam::channel;
use serde_json::Value;

/// DevUtils All-in-One Developer Utilities Command
pub fn run_utility(name: &str, args: &[String]) -> String {
    let input = args.join(" ");
    
    match name {
        // Hashing (7)
        "md5" => deep_hash_md5(&input),
        "sha1" => deep_hash_sha1(&input),
        "sha256" => deep_hash_sha256(&input),
        "sha512" => deep_hash_sha512(&input),
        "blake3" => deep_hash_blake3(&input),
        "crc32" => deep_hash_crc32(&input),
        "adler32" => deep_hash_adler32(&input),

        // Encoding (7)
        "base64" | "b64" => deep_encode_base64(&input),
        "unbase64" | "deb64" => deep_decode_base64(&input),
        "b64url" => deep_encode_base64_url(&input),
        "hex" => deep_hex(&input),
        "urlencode" | "urlenc" => urlencoding::encode(&input).to_string(),
        "urldecode" | "urldec" => urlencoding::decode(&input).map(|s| s.to_string()).unwrap_or_default(),
        "htmlencode" => html_escape::encode_text(&input).to_string(),

        // Formatting (6)
        "jsonfmt" | "json" => deep_json_fmt(&input, true),
        "jsoncompact" => deep_json_fmt(&input, false),
        "yaml2json" | "yaml" => deep_yaml_to_json(&input),
        "toml2json" | "toml" => deep_toml_to_json(&input),
        "xml2json" | "xml" => deep_xml_to_json(&input),
        "csv2json" | "csv" => deep_csv_to_json(&input),

        // Filesystem (9)
        "du" => deep_dir_size(&input),
        "tree" => deep_tree(&input),
        "find" => deep_find(&input),
        "grep" => deep_grep(args),
        "replace" => deep_replace(args),
        "stat" => deep_stat(&input),
        "chmod" => deep_chmod(args),
        "chown" => deep_chown(args),
        "touch" => deep_touch(&input),

        // System (9)
        "free" | "mem" => deep_memory_usage(),
        "df" | "disk" => deep_disk_usage(),
        "top" | "sys" => deep_sys_info(),
        "cpuinfo" => deep_cpu_info(),
        "netstats" => deep_net_stats(),
        "uptime" => deep_uptime(),
        "whoami" => whoami::username(),
        "hostname" => whoami::hostname(),
        "env" => std::env::vars().map(|(k,v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("\n"),

        // Text (9)
        "wc" => deep_wc(&input),
        "rev" => input.chars().rev().collect(),
        "sort" => deep_sort(&input),
        "uniq" => deep_uniq(&input),
        "head" => deep_head(args),
        "tail" => deep_tail(args),
        "tr" => deep_tr(args),
        "cut" => deep_cut(args),
        "join" => deep_join(args),

        // Security (3)
        "uuid" => uuid::Uuid::new_v4().to_string(),
        "uuidv7" => uuid::Uuid::now_v7().to_string(),
        "password" | "pass" => {
            let len = args.first().and_then(|a| a.parse().ok()).unwrap_or(16);
            gen_password(len)
        }
        "jwt" => deep_jwt_decode(&input),

        "pipe" | "pipeline" => run_pipeline(&input),

        _ => format!("Utility '{}' not found. Try 'help' for a list of 50 deep utilities.", name),
    }
}

pub fn run_all_utils_cmd(name: &str, args: &[String]) -> String {
    run_utility(name, args)
}

// --- Joint Execution (Pipeline) ---

pub fn run_pipeline(pipeline_str: &str) -> String {
    let parts: Vec<&str> = pipeline_str.split('|').map(|s| s.trim()).collect();
    let mut current_input = String::new();
    
    for part in parts {
        let tokens: Vec<String> = part.split_whitespace().map(|s| s.to_string()).collect();
        if tokens.is_empty() { continue; }
        
        let cmd = &tokens[0];
        let mut args = tokens[1..].to_vec();
        
        // Pass current input as the last argument if not empty
        if !current_input.is_empty() {
            args.push(current_input.clone());
        }
        
        if cmd == "ai" {
            let mut opts = crate::ai::GenerateOptions::default();
            let mut clean_args = Vec::new();
            
            for arg in args {
                if arg == "--grounding" {
                    opts.grounding = true;
                } else if arg.starts_with("--system=") {
                    opts.system_instruction = Some(arg.trim_start_matches("--system=").to_string());
                } else {
                    clean_args.push(arg);
                }
            }

            let prompt = clean_args.join(" ");
            let ai = crate::ai::AIClient::new();
            current_input = match ai.generate_with_options(&prompt, &opts) {
                Ok((response, _)) => response,
                Err(e) => format!("AI Error: {}", e),
            };
        } else {
            current_input = run_utility(cmd, &args);
        }
    }
    
    current_input
}

// --- Deep Hashing ---

fn deep_hash_md5(input: &str) -> String {
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return format!("{:x}", md5::compute(&mmap[..]));
            }
        }
    }
    format!("{:x}", md5::compute(input.as_bytes()))
}

fn deep_hash_sha1(input: &str) -> String {
    use sha1::{Sha1, Digest};
    let mut h = Sha1::new();
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                h.update(&mmap[..]);
                return format!("{:x}", h.finalize());
            }
        }
    }
    h.update(input.as_bytes());
    format!("{:x}", h.finalize())
}

fn deep_hash_sha256(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut h = Sha256::new();
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                h.update(&mmap[..]);
                return format!("{:x}", h.finalize());
            }
        }
    }
    h.update(input.as_bytes());
    format!("{:x}", h.finalize())
}

fn deep_hash_sha512(input: &str) -> String {
    use sha2::{Sha512, Digest};
    let mut h = Sha512::new();
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                h.update(&mmap[..]);
                return format!("{:x}", h.finalize());
            }
        }
    }
    h.update(input.as_bytes());
    format!("{:x}", h.finalize())
}

fn deep_hash_blake3(input: &str) -> String {
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return blake3::hash(&mmap[..]).to_hex().to_string();
            }
        }
    }
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

fn deep_hash_crc32(input: &str) -> String {
    use crc32fast::Hasher;
    let mut h = Hasher::new();
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                h.update(&mmap[..]);
                return format!("{:08x}", h.finalize());
            }
        }
    }
    h.update(input.as_bytes());
    format!("{:08x}", h.finalize())
}

fn deep_hash_adler32(input: &str) -> String {
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return format!("{:08x}", adler::adler32_slice(&mmap[..]));
            }
        }
    }
    format!("{:08x}", adler::adler32_slice(input.as_bytes()))
}

// --- Deep Encoding ---

fn deep_encode_base64(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return STANDARD.encode(&mmap[..]);
            }
        }
    }
    STANDARD.encode(input)
}

fn deep_encode_base64_url(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return URL_SAFE_NO_PAD.encode(&mmap[..]);
            }
        }
    }
    URL_SAFE_NO_PAD.encode(input)
}

fn deep_decode_base64(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    match STANDARD.decode(input.trim()) {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(_) => "Invalid base64".to_string(),
    }
}

fn deep_hex(input: &str) -> String {
    if Path::new(input).is_file() {
        if let Ok(file) = File::open(input) {
            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                return mmap.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("");
            }
        }
    }
    input.as_bytes().iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
}

// --- Deep Formatting ---

fn deep_json_fmt(input: &str, pretty: bool) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(input) {
        if pretty {
            serde_json::to_string_pretty(&v).unwrap_or_default()
        } else {
            serde_json::to_string(&v).unwrap_or_default()
        }
    } else { "Invalid JSON".to_string() }
}

fn deep_yaml_to_json(input: &str) -> String {
    match serde_yaml::from_str::<Value>(input) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
        Err(e) => format!("Invalid YAML: {}", e),
    }
}

fn deep_toml_to_json(input: &str) -> String {
    match toml::from_str::<Value>(input) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
        Err(e) => format!("Invalid TOML: {}", e),
    }
}

fn deep_xml_to_json(input: &str) -> String {
    match quick_xml::de::from_str::<Value>(input) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
        Err(e) => format!("Invalid XML: {}", e),
    }
}

fn deep_csv_to_json(input: &str) -> String {
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let mut items: Vec<Value> = Vec::new();
    let mut skipped = 0;
    for result in reader.deserialize() {
        if let Ok(item) = result {
            items.push(item);
        } else {
            skipped += 1;
        }
    }
    if skipped > 0 {
        eprintln!("Warning: Skipped {} malformed row(s)", skipped);
    }
    serde_json::to_string_pretty(&Value::from(items)).unwrap_or_default()
}

// --- Deep Filesystem ---

fn deep_dir_size(path: &str) -> String {
    let (tx, rx) = channel::unbounded();
    WalkBuilder::new(if path.is_empty() { "." } else { path }).build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |res| {
            if let Ok(e) = res {
                if let Ok(m) = e.metadata() {
                    if m.is_file() { tx.send(m.len()).ok(); }
                }
            }
            ignore::WalkState::Continue
        })
    });
    drop(tx);
    let total: u64 = rx.into_iter().sum();
    format!("{} bytes ({} MB)", total, total / 1024 / 1024)
}

fn deep_tree(path: &str) -> String {
    let mut out = String::new();
    for result in WalkBuilder::new(if path.is_empty() { "." } else { path }).build() {
        if let Ok(e) = result {
            let depth = e.depth();
            let indent = "  ".repeat(depth);
            out.push_str(&format!("{}{}\n", indent, e.file_name().to_string_lossy()));
        }
    }
    out
}

fn deep_find(query: &str) -> String {
    let mut out = Vec::new();
    for result in WalkBuilder::new(".").build() {
        if let Ok(e) = result {
            if e.file_name().to_string_lossy().contains(query) {
                out.push(e.path().to_string_lossy().to_string());
            }
        }
    }
    out.join("\n")
}

fn deep_grep(args: &[String]) -> String {
    if args.len() < 2 { return "Usage: grep <pattern> <file/dir>".to_string(); }
    let pattern = &args[0];
    let path = &args[1];
    
    let walker = WalkBuilder::new(path).build_parallel();
    let (tx, rx) = channel::unbounded();
    
    walker.run(|| {
        let tx = tx.clone();
        let pattern = pattern.clone();
        Box::new(move |res| {
            if let Ok(e) = res {
                if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(e.path()) {
                        for (i, line) in content.lines().enumerate() {
                            if line.contains(&pattern) {
                                tx.send(format!("{}:{}: {}", e.path().display(), i+1, line)).ok();
                            }
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
    drop(tx);
    rx.into_iter().collect::<Vec<_>>().join("\n")
}

fn deep_replace(args: &[String]) -> String {
    if args.len() < 3 { return "Usage: replace <pattern> <replacement> <file>".to_string(); }
    let pattern = &args[0];
    let replacement = &args[1];
    let file_path = &args[2];
    
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let new_content = content.replace(pattern, replacement);
        if std::fs::write(file_path, new_content).is_ok() {
            return format!("Replaced all occurrences of '{}' with '{}' in {}", pattern, replacement, file_path);
        }
    }
    "Failed to replace".to_string()
}

fn deep_stat(path: &str) -> String {
    if let Ok(m) = std::fs::metadata(path) {
        format!("Size: {}\nType: {:?}\nModified: {:?}\nReadOnly: {}", 
            m.len(), m.file_type(), m.modified().ok(), m.permissions().readonly())
    } else { "File not found".to_string() }
}

fn deep_chmod(_args: &[String]) -> String {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        if args.len() < 2 { return "Usage: chmod <mode> <file>".to_string(); }
        let mode = u32::from_str_radix(&args[0], 8).unwrap_or(0o644);
        if std::fs::set_permissions(&args[1], std::fs::Permissions::from_mode(mode)).is_ok() {
            return format!("Chmod {} set for {}", args[0], args[1]);
        }
    }
    "Chmod not supported on this platform or failed".to_string()
}

fn deep_chown(_args: &[String]) -> String {
    "Chown requires native OS bindings; use 'stat' to view owner.".to_string()
}

fn deep_touch(path: &str) -> String {
    if OpenOptions::new().create(true).append(true).open(path).is_ok() {
        format!("Touched {}", path)
    } else { "Failed to touch".to_string() }
}

// --- Deep System ---

fn deep_memory_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    format!("Used: {} MB / Total: {} MB", sys.used_memory() / 1024 / 1024, sys.total_memory() / 1024 / 1024)
}

fn deep_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    disks.iter().map(|d| format!("{:?}: {}/{} GB free", d.mount_point(), d.available_space()/1024/1024/1024, d.total_space()/1024/1024/1024)).collect::<Vec<_>>().join("\n")
}

fn deep_sys_info() -> String {
    format!("OS: {}\nKernel: {}\nArch: {}\nCores: {}", 
        System::name().unwrap_or_default(), System::kernel_version().unwrap_or_default(), System::cpu_arch().unwrap_or_default(), num_cpus::get())
}

fn deep_cpu_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.cpus().iter().enumerate().map(|(i, c)| format!("CPU {}: {}% @ {} MHz", i, c.cpu_usage(), c.frequency())).collect::<Vec<_>>().join("\n")
}

fn deep_net_stats() -> String {
    let nets = Networks::new_with_refreshed_list();
    nets.iter().map(|(n, d)| format!("{}: RX {} / TX {} bytes", n, d.received(), d.transmitted())).collect::<Vec<_>>().join("\n")
}

fn deep_uptime() -> String {
    format!("{} seconds", System::uptime())
}

// --- Deep Text ---

fn deep_wc(path: &str) -> String {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let mut lines = 0;
        let mut words = 0;
        let mut chars = 0;
        for line in reader.lines().flatten() {
            lines += 1;
            words += line.split_whitespace().count();
            chars += line.len();
        }
        return format!("Lines: {}, Words: {}, Chars: {}", lines, words, chars);
    }
    "File not found".to_string()
}

fn deep_sort(path: &str) -> String {
    if let Ok(content) = std::fs::read_to_string(path) {
        let mut lines: Vec<_> = content.lines().collect();
        lines.par_sort();
        return lines.join("\n");
    }
    "File not found".to_string()
}

fn deep_uniq(path: &str) -> String {
    if let Ok(content) = std::fs::read_to_string(path) {
        let mut lines: Vec<_> = content.lines().collect();
        // UNIX-style uniq only removes adjacent duplicates.
        // For global deduplication, use devutils utils pipe "sort file | uniq"
        lines.dedup();
        return lines.join("\n");
    }
    "File not found".to_string()
}

fn deep_head(args: &[String]) -> String {
    let n = args.first().and_then(|a| a.parse().ok()).unwrap_or(10);
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        return reader.lines().take(n).flatten().collect::<Vec<_>>().join("\n");
    }
    "File not found".to_string()
}

fn deep_tail(args: &[String]) -> String {
    let n = args.first().and_then(|a| a.parse().ok()).unwrap_or(10);
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader.lines().flatten().collect();
        let start = lines.len().saturating_sub(n);
        return lines[start..].join("\n");
    }
    "File not found".to_string()
}

fn deep_tr(args: &[String]) -> String {
    if args.len() < 3 { return "Usage: tr <from> <to> <file>".to_string(); }
    let from = &args[0];
    let to = &args[1];
    let path = &args[2];
    if let Ok(content) = std::fs::read_to_string(path) {
        return content.replace(from, to);
    }
    "File not found".to_string()
}

fn deep_cut(args: &[String]) -> String {
    let delimiter = args.first().map(|s| s.as_str()).unwrap_or(",");
    let field = args.get(1).and_then(|a| a.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
    let path = args.get(2).map(|s| s.as_str()).unwrap_or("");
    if let Ok(content) = std::fs::read_to_string(path) {
        return content.lines().map(|l| l.split(delimiter).nth(field).unwrap_or_default()).collect::<Vec<_>>().join("\n");
    }
    "File not found".to_string()
}

fn deep_join(args: &[String]) -> String {
    if args.len() < 2 { return "Usage: join <file1> <file2>".to_string(); }
    let c1 = std::fs::read_to_string(&args[0]).unwrap_or_default();
    let c2 = std::fs::read_to_string(&args[1]).unwrap_or_default();
    let mut out = Vec::new();
    for (l1, l2) in c1.lines().zip(c2.lines()) {
        out.push(format!("{} {}", l1, l2));
    }
    out.join("\n")
}

// --- Deep Security ---

fn gen_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| {
        let idx = rng.gen_range(0..CHARSET.len());
        CHARSET[idx] as char
    }).collect()
}

fn deep_jwt_decode(token: &str) -> String {
    use jsonwebtoken::{decode_header, decode, DecodingKey, Validation};
    if let Ok(header) = decode_header(token) {
        let validation = Validation::new(header.alg);
        if let Ok(data) = decode::<Value>(token, &DecodingKey::from_secret(&[]), &validation) {
            return serde_json::to_string_pretty(&data.claims).unwrap_or_default();
        }
        return format!("Header: {:?}", header);
    }
    "Invalid JWT".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        // Test basic pipeline logic: uuid | rev
        let res = run_pipeline("uuid | rev");
        assert_eq!(res.len(), 36);
    }
}
