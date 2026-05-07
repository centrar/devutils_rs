//! Vision Module - Screenshot and image support for AI analysis

use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn capture_screen() -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join("devutils_screenshot.png");
    
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(["-Command", &format!(
                "Add-Type -AssemblyName System.Windows.Forms; $bmp = New-Object System.Drawing.Bitmap([System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Width, [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Height); $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen(0, 0, 0, 0, $bmp.Size); $bmp.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()",
                path.display()
            )])
            .output()
            .map_err(|e| format!("Screenshot failed: {}", e))?;
        
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }
    
    Ok(path)
}

pub fn capture_window(title: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join("devutils_window.png");
    
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $w = Get-Process | Where-Object {{$_.MainWindowTitle -like '*{}*'}} | Select-Object -First 1; if ($w) {{ $hwnd = $w.MainWindowHandle; SetForegroundWindow($hwnd); Start-Sleep -Milliseconds 100; $bmp = New-Object System.Drawing.Bitmap(800, 600); $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen([System.Drawing.Point]::new(0,0), [System.Drawing.Point]::new(0,0), $bmp.Size); $bmp.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose() }}",
            title, path.display()
        );
        
        std::process::Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| format!("Window capture failed: {}", e))?;
    }
    
    Ok(path)
}

pub fn image_to_base64(path: &str) -> Result<String, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read image: {}", e))?;
    Ok(BASE64.encode(&data))
}

pub fn analyze_image(path: &str, prompt: &str) -> Result<String, String> {
    let b64 = image_to_base64(path)?;
    let size = b64.len();
    
    let client = crate::ai::AIClient::new();
    Ok(client.generate_code(&format!(
        "This is a base64-encoded image ({} bytes). Analyze it and answer: {}",
        size, prompt
    )).map(|(s, _)| s).unwrap_or_else(|e| e))
}

pub fn describe_screenshot() -> Result<String, String> {
    let path = capture_screen()?;
    if !path.exists() {
        return Err("Screenshot failed".to_string());
    }
    analyze_image(path.to_str().unwrap_or(""), "Describe what you see in this image")
}

pub fn find_bug_screenshot() -> Result<String, String> {
    let path = capture_screen()?;
    if !path.exists() {
        return Err("Screenshot failed".to_string());
    }
    analyze_image(path.to_str().unwrap_or(""), "What bug or error do you see? Describe it")
}

pub fn add_images(paths: Vec<String>) -> Result<Vec<ImageContext>, String> {
    let mut contexts = Vec::new();
    
    for path in paths {
        let p = std::path::Path::new(&path);
        if !p.exists() {
            return Err(format!("File not found: {}", path));
        }
        
        let ext = p.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if !["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext.as_str()) {
            return Err(format!("Unsupported image format: {}", ext));
        }
        
        let data = std::fs::read(&path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        
        let b64 = BASE64.encode(&data);
        
        contexts.push(ImageContext {
            path: path.clone(),
            base64: b64,
            mime_type: match ext.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "webp" => "image/webp",
                _ => "image/png",
            }.to_string(),
        });
    }
    
    Ok(contexts)
}

#[derive(Clone)]
pub struct ImageContext {
    pub path: String,
    pub base64: String,
    pub mime_type: String,
}

pub fn get_image_description(path: &str) -> Result<String, String> {
    analyze_image(path, "Describe this image in detail")
}