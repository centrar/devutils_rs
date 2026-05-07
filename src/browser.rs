//! Browser Automation - Built-in browser using WebView2/Playwright

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;

static BROWSER: Lazy<Mutex<Option<BrowserSession>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    pub id: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserResult {
    pub success: bool,
    pub html: Option<String>,
    pub text: Option<String>,
    pub screenshot: Option<String>,
    pub error: Option<String>,
}

pub fn launch_browser(url: &str) -> Result<String, String> {
    let _browser = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| e.to_string())?;
        "launched"
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        "launched"
    } else {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        "launched"
    };

    Ok(format!("Browser launched with URL: {}", url))
}

pub fn take_screenshot(path: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
$bitmap.Save('{}')
$graphics.Dispose()
$bitmap.Dispose()
"#,
                path
            ),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        return Ok(format!("Screenshot saved to: {}", path));
    }

    Err(String::from_utf8_lossy(&output.stderr).to_string())
}

pub fn get_page_source_url(url: &str) -> Result<BrowserResult, String> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                r#"
$response = Invoke-WebRequest -Uri '{}' -UseBasicParsing
$response.StatusCode
$response.Content
"#,
                url
            ),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let html = String::from_utf8_lossy(&output.stdout).to_string();
        let text = strip_tags(&html);
        return Ok(BrowserResult {
            success: true,
            html: Some(html),
            text: Some(text),
            screenshot: None,
            error: None,
        });
    }

    Ok(BrowserResult {
        success: false,
        html: None,
        text: None,
        screenshot: None,
        error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
    })
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let _in_tag = false;
    let mut in_script = false;

    for chunk in html.split('<') {
        if chunk.starts_with("script") {
            in_script = true;
        }
        if chunk.contains("</script>") {
            in_script = false;
            continue;
        }

        if in_script {
            continue;
        }

        let parts: Vec<&str> = chunk.splitn(2, '>').collect();
        if parts.len() > 1 {
            result.push_str(parts[1]);
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn search_web(query: &str) -> Result<String, String> {
    let url = format!(
        "https://www.google.com/search?q={}",
        query.replace(' ', "+")
    );
    let result = get_page_source_url(&url)?;

    if let Some(text) = result.text {
        let lines: Vec<&str> = text.lines().take(10).collect();
        return Ok(lines.join("\n"));
    }

    Err("No results found".to_string())
}

pub fn browser_commands() {
    println!("\n\x1b[36m🌐 Browser Automation\x1b[0m");
    println!("\nUsage:");
    println!("  devutils browser open <url>");
    println!("  devutils browser screenshot <path>");
    println!("  devutils browser fetch <url>");
    println!("  devutils browser search <query>");
    println!("\nNote: Uses system browser + PowerShell for automation");
}
