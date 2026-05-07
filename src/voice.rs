//! Voice Module - Voice input and speech-to-text for DevUtils

use std::process::Command;

pub fn speak_text(text: &str) -> Result<(), String> {
    let escaped = text.replace("'", "''");
    
    #[cfg(target_os = "windows")]
    {
        Command::new("powershell")
            .args(["-Command", &format!(
                "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.Speak('{}')",
                escaped
            )])
            .output()
            .map_err(|e| format!("Speak failed: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("say")
            .arg(&escaped)
            .output()
            .map_err(|e| format!("Speak failed: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("espeak")
            .arg(&escaped)
            .output()
            .map_err(|e| format!("Speak failed: {}", e))?;
    }
    
    Ok(())
}

pub fn speak_file(text: &str, filename: &str) -> Result<(), String> {
    let escaped = text.replace("'", "''");
    
    #[cfg(target_os = "windows")]
    {
        Command::new("powershell")
            .args(["-Command", &format!(
                "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.SetOutputToWaveFile('{}'); $synth.Speak('{}')",
                filename, escaped
            )])
            .output()
            .map_err(|e| format!("Failed: {}", e))?;
    }
    
    Ok(())
}

pub fn record_voice(duration_secs: u32) -> Result<String, String> {
    if duration_secs == 0 {
        return Ok("No input".to_string());
    }
    
    let path = std::env::temp_dir().join("devutils_voice.wav");
    let path_str = path.to_string_lossy();
    
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $rec = New-Object System.Media.SoundRecorder('{}', 44100, 16, 2); $rec.StartRecording(); Start-Sleep -Seconds {}; $rec.StopRecording()",
            path_str, duration_secs
        );
        
        Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| format!("Record failed: {}", e))?;
    }
    
    Ok(path_str.to_string())
}

pub fn transcribe_whisper(audio_path: &str) -> Result<String, String> {
    if !std::path::Path::new(audio_path).exists() {
        return Err("Audio file not found".to_string());
    }
    
    let output = Command::new("whisper")
        .arg(audio_path)
        .arg("--language")
        .arg("english")
        .arg("--model")
        .arg("base")
        .output()
        .map_err(|e| format!("Whisper failed: {}", e))?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    let txt_path = format!("{}.txt", audio_path.trim_end_matches(".wav").trim_end_matches(".mp3"));
    std::fs::read_to_string(&txt_path)
        .map_err(|e| format!("Failed to read transcript: {}", e))
}

pub fn voice_input(duration_secs: u32) -> Result<String, String> {
    let path = record_voice(duration_secs)?;
    
    let has_whisper = Command::new("whisper")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if has_whisper {
        transcribe_whisper(&path)
    } else {
        Ok(format!("Recorded to {} but whisper not installed", path))
    }
}

pub fn voice_chat() -> Result<String, String> {
    let _providers = crate::local_models::check_providers();
    
    let audio = record_voice(5)?;
    let has_whisper = Command::new("whisper")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if has_whisper {
        let text = transcribe_whisper(&audio)?;
        
        let response = crate::local_models::run_local(&text)?;
        speak_text(&response)?;
        
        Ok(response)
    } else {
        Ok(format!("Voice recorded to {}. Install whisper for transcription.", audio))
    }
}

pub fn list_voices() -> Vec<String> {
    vec![
        "default".to_string(),
        "David".to_string(),
        "Zira".to_string(),
    ]
}