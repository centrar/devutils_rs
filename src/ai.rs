//! AI Module - Full-featured with OpenAI, NVIDIA, and multi-provider support
//! Includes streaming support for real-time responses

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub struct AIClient {
    api_key: String,
    model: String,
    base_url: String,
    #[allow(dead_code)]
    client: reqwest::blocking::Client,
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub system_instruction: Option<String>,
    pub grounding: bool,
    pub files: Vec<String>,
    pub cache_id: Option<String>,
    /// Temperature for generation (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Max tokens to generate
    pub max_tokens: usize,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            system_instruction: None,
            grounding: false,
            files: Vec::new(),
            cache_id: None,
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub content: String,
    pub score: f32,
}

#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: usize,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Option<TokenUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

impl AIClient {
    pub fn new() -> Self {
        let (api_key, model, base_url) = Self::get_active_provider_config();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            api_key,
            model,
            base_url,
            client,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub fn provider_name(&self) -> String {
        if self.base_url.contains("nvidia") {
            "NVIDIA".to_string()
        } else if self.base_url.contains("deepseek") {
            "DeepSeek".to_string()
        } else if self.base_url.contains("openai") {
            "OpenAI".to_string()
        } else if self.base_url.contains("anthropic") {
            "Anthropic".to_string()
        } else if self.base_url.contains("generativelanguage") {
            "Gemini".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn get_active_provider_config() -> (String, String, String) {
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.is_empty() {
                let model = std::env::var("GEMINI_MODEL")
                    .unwrap_or_else(|_| "gemini-2.5-flash".to_string());
                return (
                    key,
                    model,
                    "https://generativelanguage.googleapis.com/v1beta".to_string(),
                );
            }
        }

        if let Ok(nvidia_key) = std::env::var("NVIDIA_API_KEY") {
            if !nvidia_key.is_empty() {
                let model = std::env::var("NVIDIA_MODEL")
                    .unwrap_or_else(|_| "nvidia/deepseek-coder-6.7b-instruct".to_string());
                let base_url = std::env::var("NVIDIA_BASE_URL")
                    .unwrap_or_else(|_| "https://integrate.api.nvidia.com/v1".to_string());
                return (nvidia_key, model, base_url);
            }
        }

        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                return (
                    key,
                    "gpt-4".to_string(),
                    "https://api.openai.com/v1".to_string(),
                );
            }
        }

if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
    if !key.is_empty() {
        return (
            key,
            "deepseek-chat".to_string(),
            "https://api.deepseek.com/v1".to_string(),
        );
    }
}

(
    String::new(),
    "gpt-4".to_string(),
    "https://api.openai.com/v1".to_string(),
)
    }

    pub fn with_model(model: &str) -> Self {
        let (api_key, _, base_url) = Self::get_active_provider_config();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            api_key,
            model: model.to_string(),
            base_url,
            client,
        }
    }

    pub fn generate(&self, prompt: &str) -> Result<(String, TokenUsage), String> {
        self.generate_with_options(prompt, &GenerateOptions::default())
    }

    /// True SSE streaming generation — parses `text/event-stream` chunks as they arrive.
    /// Falls back to Gemini streaming endpoint for Gemini models.
    /// For OpenAI-compatible endpoints, uses `stream: true` and parses SSE.
    pub fn generate_stream<F>(&self, prompt: &str, mut callback: F) -> Result<String, String>
    where
        F: FnMut(&str),
    {
        if self.api_key.is_empty() {
            return Err("ERROR: API key is not configured.".to_string());
        }

        // Gemini streaming path
        if self.base_url.contains("generativelanguage") {
            return self.generate_stream_gemini(prompt, callback);
        }

        // OpenAI-compatible SSE path (works for OpenAI, NVIDIA, DeepSeek, Ollama)
        #[derive(Serialize)]
        struct StreamRequest {
            model: String,
            messages: Vec<serde_json::Value>,
            stream: bool,
            temperature: f32,
        }

        let request = StreamRequest {
            model: self.model.clone(),
            messages: vec![serde_json::json!({ "role": "user", "content": prompt })],
            stream: true,
            temperature: 0.7,
        };

        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|e| format!("Stream request failed: {}", e))?;

        let mut full_text = String::new();
        // Read SSE line by line
        let reader = BufReader::new(resp);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Stream read error: {}", e))?;
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                        callback(delta);
                        full_text.push_str(delta);
                    }
                }
            }
        }
        Ok(full_text)
    }

    /// Gemini native streaming via `streamGenerateContent`
    fn generate_stream_gemini<F>(&self, prompt: &str, mut callback: F) -> Result<String, String>
    where
        F: FnMut(&str),
    {
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, self.model, self.api_key
        );
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });
        let resp = self.client.post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Gemini stream request failed: {}", e))?;

        let mut full_text = String::new();
        let reader = BufReader::new(resp);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Gemini stream read error: {}", e))?;
            if line.starts_with("data: ") {
                let data = &line[6..];
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(text) = v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                        callback(text);
                        full_text.push_str(text);
                    }
                }
            }
        }
        Ok(full_text)
    }

    pub fn generate_with_options(&self, prompt: &str, opts: &GenerateOptions) -> Result<(String, TokenUsage), String> {
        if self.api_key.is_empty() {
            return Err("ERROR: OPENAI_API_KEY / GEMINI_API_KEY is not set.".to_string());
        }

        // Check if using Gemini Native
        if self.base_url.contains("generativelanguage") {
            return self.generate_gemini(prompt, opts);
        }

        // Check if using Ollama local
        if self.base_url.contains("localhost:11434") || self.base_url.contains("ollama") {
            return self.generate_ollama(prompt);
        }

        // Fallback search grounding for OpenAI/DeepSeek
        let mut final_prompt = prompt.to_string();
        if opts.grounding {
            // Minimal web search fallback (using DuckDuckGo HTML)
            if let Ok(search_results) = Self::duckduckgo_search(prompt) {
                final_prompt = format!("Background Context from Web:\n{}\n\nUser Request: {}", search_results, prompt);
            }
        }

        // Handle system instructions
        let mut messages = Vec::new();
        if let Some(sys) = &opts.system_instruction {
            messages.push(Message {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }
        messages.push(Message {
            role: "user".to_string(),
            content: final_prompt,
        });

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send();

        match resp {
            Ok(res) => {
                let body = res.text().unwrap_or_default();
                match serde_json::from_str::<OpenAIResponse>(&body) {
                    Ok(data) => {
                        let text = data.choices
                            .first()
                            .map(|c| c.message.content.clone())
                            .ok_or_else(|| "ERROR: Empty response from AI".to_string())?;
                        let usage = data.usage.unwrap_or_default();
                        Ok((text, usage))
                    },
                    Err(e) => {
                        eprintln!("[ERROR] Failed to parse response: {}", e);
                        Err("ERROR: Failed to parse response".to_string())
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] API request failed: {}", e);
                Err("ERROR: API request failed".to_string())
            }
        }
    }

    fn generate_gemini(&self, prompt: &str, opts: &GenerateOptions) -> Result<(String, TokenUsage), String> {
        #[derive(Serialize)]
        struct GeminiFileData {
            #[serde(rename = "fileUri")]
            file_uri: String,
        }
        #[derive(Serialize)]
        struct GeminiPart {
            #[serde(skip_serializing_if = "Option::is_none")]
            text: Option<String>,
            #[serde(rename = "fileData", skip_serializing_if = "Option::is_none")]
            file_data: Option<GeminiFileData>,
        }
        #[derive(Serialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }
        #[derive(Serialize)]
        struct GeminiSysInstruction {
            parts: Vec<GeminiPart>,
        }
        #[derive(Serialize)]
        struct GeminiTool {
            #[serde(rename = "googleSearchRetrieval")]
            google_search_retrieval: Option<serde_json::Value>,
        }
        #[derive(Serialize)]
        struct GeminiRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            contents: Option<Vec<GeminiContent>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            system_instruction: Option<GeminiSysInstruction>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tools: Option<Vec<GeminiTool>>,
            #[serde(rename = "cachedContent", skip_serializing_if = "Option::is_none")]
            cached_content: Option<String>,
        }

        let mut parts = Vec::new();
        for file_uri in &opts.files {
            parts.push(GeminiPart {
                text: None,
                file_data: Some(GeminiFileData { file_uri: file_uri.to_string() }),
            });
        }
        parts.push(GeminiPart {
            text: Some(prompt.to_string()),
            file_data: None,
        });

        let mut req = GeminiRequest {
            contents: Some(vec![GeminiContent { parts }]),
            system_instruction: None,
            tools: None,
            cached_content: opts.cache_id.clone(),
        };

        if let Some(sys) = &opts.system_instruction {
            req.system_instruction = Some(GeminiSysInstruction {
                parts: vec![GeminiPart { text: Some(sys.to_string()), file_data: None }],
            });
        }

        if opts.grounding {
            req.tools = Some(vec![GeminiTool {
                google_search_retrieval: Some(serde_json::json!({})),
            }]);
        }

        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, self.model, self.api_key);
        let resp = self.client.post(url).json(&req).send();

        match resp {
            Ok(res) => {
                let body = res.text().unwrap_or_default();
                let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                if let Some(text) = v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    let usage = if let Some(meta) = v.get("usageMetadata") {
                        TokenUsage {
                            prompt_tokens: meta["promptTokenCount"].as_u64().unwrap_or(0) as usize,
                            completion_tokens: meta["candidatesTokenCount"].as_u64().unwrap_or(0) as usize,
                            total_tokens: meta["totalTokenCount"].as_u64().unwrap_or(0) as usize,
                        }
                    } else {
                        TokenUsage::default()
                    };
                    Ok((text.to_string(), usage))
                } else {
                    Err(format!("ERROR: Failed to parse Gemini response. Raw: {}", body))
                }
            }
            Err(e) => Err(format!("ERROR: Gemini request failed: {}", e)),
        }
    }

    fn duckduckgo_search(query: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        match client.get(format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query)))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send() {
            Ok(res) => {
                let text = res.text().unwrap_or_default();
                // Simple fast extraction logic (avoiding heavy HTML parser crates for performance)
                let snippets: Vec<&str> = text.split("class=\"result__snippet").take(4).collect();
                let mut results = String::new();
                for (i, snippet) in snippets.iter().skip(1).enumerate() {
                    if let Some(idx) = snippet.find("</a>") {
                        if let Some(start) = snippet.rfind('>') {
                            let clean = &snippet[start+1..idx].replace("<b>", "").replace("</b>", "");
                            results.push_str(&format!("{}. {}\n", i+1, clean));
                        }
                    }
                }
                Ok(results)
            }
            Err(_) => Err("Search failed".to_string())
        }
    }

    pub fn upload_file(&self, path: &str, mime_type: &str) -> Result<String, String> {
        if !self.base_url.contains("generativelanguage") {
            // For non-Gemini, fallback to reading file directly as text
            return std::fs::read_to_string(path)
                .map_err(|e| format!("Fallback file read error: {}", e));
        }

        let file_bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let num_bytes = file_bytes.len();

        let upload_url = format!("https://generativelanguage.googleapis.com/upload/v1beta/files?key={}", self.api_key);
        
        // Initial Resumable Request
        #[derive(Serialize)]
        struct FileUploadMeta {
            file: FileInner
        }
        #[derive(Serialize)]
        struct FileInner {
            display_name: String
        }

        let meta = FileUploadMeta {
            file: FileInner { display_name: std::path::Path::new(path).file_name().unwrap_or_default().to_string_lossy().to_string() }
        };

        let init_resp = self.client.post(&upload_url)
            .header("X-Goog-Upload-Protocol", "resumable")
            .header("X-Goog-Upload-Command", "start")
            .header("X-Goog-Upload-Header-Content-Length", num_bytes.to_string())
            .header("X-Goog-Upload-Header-Content-Type", mime_type)
            .header("Content-Type", "application/json")
            .json(&meta)
            .send().map_err(|e| e.to_string())?;

        let upload_url_active = init_resp.headers()
            .get("X-Goog-Upload-URL")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| "Failed to get upload URL".to_string())?;

        // Upload the bytes
        let final_resp = self.client.post(upload_url_active)
            .header("X-Goog-Upload-Offset", "0")
            .header("X-Goog-Upload-Command", "upload, finalize")
            .header("Content-Length", num_bytes.to_string())
            .body(file_bytes)
            .send().map_err(|e| e.to_string())?;

        let body = final_resp.text().unwrap_or_default();
        let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        
        if let Some(uri) = v["file"]["uri"].as_str() {
            Ok(uri.to_string())
        } else {
            Err(format!("Upload failed. Raw: {}", body))
        }
    }

    pub fn create_cache(&self, file_uris: Vec<String>, ttl_minutes: u32, system_instruction: Option<String>) -> Result<String, String> {
        if !self.base_url.contains("generativelanguage") {
            return Err("ERROR: Context Caching is currently only supported natively via Gemini API.".to_string());
        }

        #[derive(Serialize)]
        struct CacheRequest {
            model: String,
            contents: Vec<serde_json::Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            system_instruction: Option<serde_json::Value>,
            ttl: String,
        }

        let mut contents = Vec::new();
        for uri in file_uris {
            contents.push(serde_json::json!({
                "role": "user",
                "parts": [{ "fileData": { "fileUri": uri } }]
            }));
        }

        let sys_instr = system_instruction.map(|s| {
            serde_json::json!({
                "parts": [{ "text": s }]
            })
        });

        let req = CacheRequest {
            model: format!("models/{}", self.model), // Must prefix with models/
            contents,
            system_instruction: sys_instr,
            ttl: format!("{}s", ttl_minutes * 60),
        };

        let url = format!("{}/cachedContents?key={}", self.base_url, self.api_key);
        let resp = self.client.post(url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send().map_err(|e| e.to_string())?;

        let body = resp.text().unwrap_or_default();
        let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();

        if let Some(name) = v["name"].as_str() {
            Ok(name.to_string())
        } else {
            Err(format!("Cache creation failed. Raw: {}", body))
        }
    }

    fn generate_ollama(&self, prompt: &str) -> Result<(String, TokenUsage), String> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let url = if self.base_url.contains("/v1") {
            format!("{}/generate", self.base_url)
        } else {
            format!("{}/api/generate", self.base_url)
        };

        let resp = self.client.post(url).json(&request).send();

        match resp {
            Ok(res) => {
                let body = res.text().unwrap_or_default();
                match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(v) => {
                        let text = v["response"].as_str().unwrap_or_default().to_string();
                        let usage = TokenUsage {
                            prompt_tokens: v["prompt_eval_count"].as_u64().unwrap_or(0) as usize,
                            completion_tokens: v["eval_count"].as_u64().unwrap_or(0) as usize,
                            total_tokens: (v["prompt_eval_count"].as_u64().unwrap_or(0) + v["eval_count"].as_u64().unwrap_or(0)) as usize,
                        };
                        Ok((text, usage))
                    },
                    Err(_) => Err("ERROR: Failed to parse Ollama response".to_string()),
                }
            }
            Err(e) => {
                eprintln!("[OLLAMA] Connection failed: {}", e);
                Err("ERROR: Ollama connection failed".to_string())
            }
        }
    }



    pub fn explain_code(&self, code: &str) -> Result<(String, TokenUsage), String> {
        let prompt = format!("Explain this code:\n```\n{}\n```", code);

        if self.api_key.is_empty() {
            return Err("ERROR: OPENAI_API_KEY is not set.".to_string());
        }

        self.generate(&prompt)
    }

    pub fn generate_code(&self, prompt: &str) -> Result<(String, TokenUsage), String> {
        if self.api_key.is_empty() {
            return Err("ERROR: OPENAI_API_KEY is not set.".to_string());
        }

        let full_prompt = format!("Generate code: {}", prompt);
        self.generate(&full_prompt)
    }

    /// Stream AI response in real-time using SSE via curl
    /// This is the most reliable cross-platform streaming method
    pub fn stream_generate(&self, prompt: &str) {
        if self.api_key.is_empty() {
            println!("ERROR: OPENAI_API_KEY is not set. Please set it via `devutils config set openai_key <KEY>` or environment variable.");
            return;
        }

        // Use curl for reliable SSE streaming
        self.stream_via_curl(prompt);
    }

    /// SSE streaming via curl - works reliably across platforms
    fn stream_via_curl(&self, prompt: &str) {
        let url = format!("{}/chat/completions", self.base_url);

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "stream": true
        });

        // Hide API key from process list by using a temporary header file
        let auth_header = format!("Authorization: Bearer {}", self.api_key);
        let auth_file = format!(".devutils/.auth_header_{}", std::process::id());
        let _ = std::fs::create_dir_all(".devutils");
        let _ = std::fs::write(&auth_file, auth_header);

        let result = Command::new("curl")
            .args([
                "-s",
                "-N",
                "--no-buffer",
                "-X",
                "POST",
                &url,
                "-H",
                &format!("@{}", auth_file),
                "-H",
                "Content-Type: application/json",
                "-d",
                &request_body.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let _ = std::fs::remove_file(&auth_file);

        match result {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().flatten() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                break;
                            }
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(content) =
                                    json["choices"][0]["delta"]["content"].as_str()
                                {
                                    print!("{}", content);
                                    std::io::Write::flush(&mut std::io::stdout()).ok();
                                }
                            }
                        }
                    }
                }
                let _ = child.wait();
            }
            Err(_e) => {
                // Fallback to non-streaming
                let (response, _) = self.generate(prompt).unwrap_or_else(|e| (e, TokenUsage::default()));
                print!("{}", response);
            }
        }
        println!();
    }

    #[allow(dead_code)]
    fn stream_openai(&self, prompt: &str) {
        #[derive(Serialize)]
        struct StreamRequest {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
            stream: bool,
        }

        let request = StreamRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            stream: true,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let mut child = Command::new("curl")
            .args([
                "-s",
                "-N",
                "-X",
                "POST",
                &url,
                "-H",
                &format!("Authorization: Bearer {}", self.api_key),
                "-H",
                "Content-Type: application/json",
                "-d",
                &serde_json::to_string(&request).unwrap_or_default(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        if let Ok(ref mut c) = child {
            let reader = BufReader::new(c.stdout.take().unwrap());
            for line in reader.lines().flatten() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str() {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }
                    }
                }
            }
        }
        println!();
    }

    #[allow(dead_code)]
    fn stream_ollama(&self, prompt: &str) {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: true,
        };

        let url = if self.base_url.contains("/v1") {
            format!("{}/generate", self.base_url)
        } else {
            format!("{}/api/generate", self.base_url)
        };

        let mut child = Command::new("curl")
            .args([
                "-s",
                "-N",
                "-X",
                "POST",
                &url,
                "-H",
                "Content-Type: application/json",
                "-d",
                &serde_json::to_string(&request).unwrap_or_default(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        if let Ok(ref mut c) = child {
            let reader = BufReader::new(c.stdout.take().unwrap());
            for line in reader.lines().flatten() {
                if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(response) = chunk["response"].as_str() {
                        print!("{}", response);
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                }
            }
        }
        println!();
    }

    pub fn debug_code(&self, code: &str) -> (String, TokenUsage) {
        if self.api_key.is_empty() {
            return (self.mock_debug(code), TokenUsage::default());
        }

        let prompt = format!("Find bugs:\n```\n{}\n```", code);
        self.generate(&prompt).unwrap_or_else(|e| (e, TokenUsage::default()))
    }

    fn mock_debug(&self, code: &str) -> String {
        let mut issues = Vec::new();

        for (i, line) in code.lines().enumerate() {
            let t = line.trim();
            if t.contains("TODO") || t.contains("FIXME") {
                issues.push(format!("Line {}: TODO/FIXME", i + 1));
            }
            if t.contains("password")
                && t.contains('=')
                && !t.contains("env")
                && !t.contains("getenv")
            {
                issues.push(format!("Line {}: Possible hardcoded secret!", i + 1));
            }
        }

        if issues.is_empty() {
            "No obvious issues found.".to_string()
        } else {
            issues.join("\n")
        }
    }

    pub fn chat_with_history(
        &self,
        messages: &[crate::chat::ChatMessage],
    ) -> Result<String, anyhow::Error> {
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(self.generate(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e))
    }

    pub fn refactor_code(&self, code: &str, _style: &str) -> String {
        if self.api_key.is_empty() {
            return self.mock_refactor(code);
        }

        let prompt = format!("Refactor:\n```\n{}\n```", code);
        self.generate(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e)
    }

    fn mock_refactor(&self, code: &str) -> String {
        code.lines()
            .filter(|l| !l.trim().starts_with("//") && !l.trim().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn generate_tests(&self, code: &str) -> String {
        if self.api_key.is_empty() {
            return self.mock_tests(code);
        }

        let prompt = format!("Generate tests for:\n```\n{}\n```", code);
        self.generate(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e)
    }

    fn mock_tests(&self, code: &str) -> String {
        let lang = detect_language(code);
        match lang.as_str() {
            "python" => {
                let funcs: Vec<String> = code.lines()
                    .filter_map(|l| {
                        let t = l.trim();
                        if t.starts_with("def ") {
                            t.split_whitespace().nth(1).map(|s| s.trim_matches('(').to_string())
                        } else { None }
                    })
                    .collect();
                let tests: Vec<String> = funcs.iter()
                    .map(|f| format!("def test_{}():\n    pass", f))
                    .collect();
                format!("import pytest\n\n{}", tests.join("\n\n"))
            }
            "javascript" => "describe('test', () => {\n  test('works', () => {\n    expect(true).toBe(true);\n  });\n});".to_string(),
            "rust" => {
                code.lines()
                    .filter_map(|l| {
                        let t = l.trim();
                        if t.starts_with("fn ") || t.starts_with("pub fn ") {
                            t.split_whitespace().nth(1).map(|f| format!("#[test]\nfn {}() {{}}", f.trim_matches('(')))
                        } else { None }
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            }
            _ => "// No tests generated\n// Set OPENAI_API_KEY for AI-powered test generation".to_string()
        }
    }

    pub fn get_embeddings(&self, text: &str) -> Vec<f32> {
        if self.api_key.is_empty() {
            return self.mock_embeddings(text);
        }

        let request = serde_json::json!({
            "input": text,
            "model": "text-embedding-ada-002"
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send();

        match resp {
            Ok(res) => {
                if let Ok(data) = res.json::<serde_json::Value>() {
                    if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                        if let Some(item) = arr.first() {
                            if let Some(emb) = item.get("embedding").and_then(|e| e.as_array()) {
                                return emb
                                    .iter()
                                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                                    .collect();
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        self.mock_embeddings(text)
    }

    fn mock_embeddings(&self, text: &str) -> Vec<f32> {
        let mut emb = vec![0.0f32; 384];
        for (i, c) in text.chars().enumerate() {
            emb[i % 384] += c as u32 as f32;
        }
        let mag = (emb.iter().map(|x| x * x).sum::<f32>()).sqrt();
        if mag > 0.0 {
            for x in &mut emb {
                *x /= mag;
            }
        }
        emb
    }
}

impl Default for AIClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SemanticSearch {
    client: AIClient,
}

impl SemanticSearch {
    pub fn new() -> Self {
        Self {
            client: AIClient::new(),
        }
    }

    pub fn search(&self, query: &str, dir: &str, limit: usize) -> Vec<SearchResult> {
        let _emb = self.client.get_embeddings(query);

        let q = query.to_lowercase();
        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(dir)
            .max_depth(10)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name.contains(&q) || path.to_string_lossy().to_lowercase().contains(&q) {
                results.push(SearchResult {
                    file: path.to_string_lossy().to_string(),
                    line: 1,
                    content: format!("Match in: {}", name),
                    score: 0.95,
                });
            }

            if results.len() >= limit {
                break;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }
}

impl Default for SemanticSearch {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_language(code: &str) -> String {
    let c = code.to_lowercase();
    if c.contains("fn ") && c.contains("->") {
        "rust".to_string()
    } else if c.contains("def ") && c.contains(":") {
        "python".to_string()
    } else if c.contains("function ") || c.contains("=>") {
        "javascript".to_string()
    } else if c.contains("func ") && c.contains("package ") {
        "go".to_string()
    } else if c.contains("public class") {
        "java".to_string()
    } else {
        "unknown".to_string()
    }
}

fn extract_definitions(code: &str) -> String {
    let defs: Vec<&str> = code
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("fn ")
                || t.starts_with("def ")
                || t.starts_with("function ")
                || t.starts_with("pub fn")
                || t.starts_with("class ")
                || t.starts_with("struct ")
        })
        .take(5)
        .collect();

    if defs.is_empty() {
        return "(none)".to_string();
    }
    defs.iter().map(|s| s.trim()).collect::<Vec<_>>().join(", ")
}

pub struct CodeCompleter {
    #[allow(dead_code)]
    client: AIClient,
}

impl CodeCompleter {
    pub fn new() -> Self {
        Self {
            client: AIClient::new(),
        }
    }

    pub fn complete(&self, code: &str, cursor: usize) -> Vec<Completion> {
        let ctx = extract_context(code, cursor);
        let lang = detect_language(code);

        let mut completions = Vec::new();

        for (keyword, template) in COMPLETION_TEMPLATES.iter() {
            if ctx.to_lowercase().contains(&keyword.to_lowercase()) {
                completions.push(Completion {
                    label: keyword.to_string(),
                    insert_text: template.to_string(),
                    detail: format!("{} completion", lang),
                    score: 0.95,
                });
            }
        }

        if completions.is_empty() {
            for (kw, tmpl) in DEFAULT_COMPLETIONS.iter() {
                completions.push(Completion {
                    label: kw.to_string(),
                    insert_text: tmpl.to_string(),
                    detail: "default".to_string(),
                    score: 0.8,
                });
            }
        }

        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        completions.truncate(10);
        completions
    }
}

impl Default for CodeCompleter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub label: String,
    pub insert_text: String,
    pub detail: String,
    pub score: f32,
}

const COMPLETION_TEMPLATES: &[(&str, &str)] = &[
    ("pub fn", " pub fn name() {\n    \n}"),
    ("pub struct", " pub struct Name {\n    \n}"),
    ("impl", " impl Type {\n    \n}"),
    ("async fn", " async fn name() {\n    \n}"),
    ("match", " match expr {\n    _ => { }\n}"),
    ("if let", " if let Some(x) = expr {\n    \n}"),
    ("for", " for item in items {\n    \n}"),
    (
        "test",
        " #[test]\n fn test_name() {\n    assert_eq!(, );\n}",
    ),
];

const DEFAULT_COMPLETIONS: &[(&str, &str)] = &[
    ("true", "true"),
    ("false", "false"),
    ("None", "None"),
    ("Some", "Some()"),
    ("Ok", "Ok()"),
    ("Err", "Err()"),
];

fn extract_context(code: &str, cursor: usize) -> String {
    let start = cursor.saturating_sub(200);
    let end = cursor.saturating_add(50);
    code.get(start..end).unwrap_or("").to_string()
}

pub struct CodeRefactorer {
    client: AIClient,
}

impl CodeRefactorer {
    pub fn new() -> Self {
        Self {
            client: AIClient::new(),
        }
    }

    pub fn apply_pattern(&self, code: &str, pattern: &str) -> String {
        match pattern {
            "extract-method" => self.extract_method(code),
            "inline" => self.inline_variable(code),
            "rename" => self.rename_variables(code),
            "split-loop" => self.split_loop(code),
            "use-match" => self.use_match_pattern(code),
            _ => self.ai_refactor(code, pattern),
        }
    }

    fn extract_method(&self, code: &str) -> String {
        let lines: Vec<&str> = code.lines().collect();
        if lines.len() < 3 {
            return code.to_string();
        }

        let defs = extract_definitions(code);
        let func_name = defs
            .split(',')
            .next()
            .unwrap_or("extracted")
            .trim()
            .to_string();
        format!(
            "fn {}(x: Type) -> Type {{\n    {}\n}}",
            func_name,
            lines
                .iter()
                .take(5)
                .map(|l| format!("    {}", l))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn inline_variable(&self, code: &str) -> String {
        code.lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.starts_with("let ") && t.contains(" = ") {
                    let parts: Vec<&str> = t.splitn(2, " = ").collect();
                    if parts.len() == 2 {
                        return Some(line.replace(&format!("let {} = ", parts[0].trim()), ""));
                    }
                }
                Some(line.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn rename_variables(&self, code: &str) -> String {
        code.lines()
            .map(|line| {
                let t = line.trim();
                if t.starts_with("let ") {
                    if let Some(var) = t.split_whitespace().nth(1) {
                        if var.len() == 1 || var == var.to_uppercase() {
                            return line.replace(var, &var.to_lowercase());
                        }
                    }
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn split_loop(&self, code: &str) -> String {
        code.lines()
            .enumerate()
            .map(|(i, line)| {
                if line.contains("for ") && i > 0 {
                    format!("    {}", line.trim())
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn use_match_pattern(&self, code: &str) -> String {
        let mut result = Vec::new();

        for line in code.lines() {
            let t = line.trim();
            if t.contains("if let Some(") || t.contains("if let Ok(") {
                let var = t
                    .split('(')
                    .nth(1)
                    .and_then(|s| s.split(')').next())
                    .unwrap_or("x");
                result.push(format!("match {} {{", var));
                result.push("    Some(x) => {{ }},".to_string());
                result.push("    None => {{ }},".to_string());
                result.push("}".to_string());
            } else {
                result.push(line.to_string());
            }
        }

        result.join("\n")
    }

    fn ai_refactor(&self, code: &str, pattern: &str) -> String {
        if self.client.api_key.is_empty() {
            return self.mock_refactor(code, pattern);
        }

        let prompt = format!("Refactor using '{}':\n```\n{}\n```", pattern, code);
        self.client.generate(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e)
    }

    fn mock_refactor(&self, code: &str, _pattern: &str) -> String {
        code.lines()
            .filter(|l| !l.trim().starts_with("//") && !l.trim().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for CodeRefactorer {
    fn default() -> Self {
        Self::new()
    }
}
