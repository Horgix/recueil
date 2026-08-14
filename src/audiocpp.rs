use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Clone)]
pub struct AudioCppConfig {
    pub host: String,
    pub model: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

/// Converts an MP4 file to a temporary WAV file using the `ffmpeg` CLI and
/// returns the path to that temp file. Fails if the input is not an MP4 file.
/// The caller is responsible for deleting the temp file when done.
async fn convert_mp4_to_wav(input: &Path) -> Result<PathBuf, String> {
    let input_extension = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if input_extension != "mp4" {
        return Err(format!("expected an MP4 file, got: {}", input.display()));
    }

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("failed to extract file stem from: {}", input.display()))?;
    let output_path = std::env::temp_dir().join(format!("audiocpp_{stem}.wav"));

    eprintln!(
        "[audiocpp] converting {} -> {} via ffmpeg",
        input.display(),
        output_path.display()
    );

    // Builds equivalent to: ffmpeg -i <input>.mp4 -ar 16000 -ac 1 <output>.wav
    let status = tokio::process::Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(input)
        // 16 kHz mono PCM, standard for speech-to-text APIs
        .args(["-ar", "16000", "-ac", "1", "-f", "wav"])
        .arg(&output_path)
        .status()
        .await
        .map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;

    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }

    Ok(output_path)
}

pub async fn transcribe(
    config: &AudioCppConfig,
    audio_path: &Path,
) -> Result<TranscriptionResponse, String> {
    let url = format!("http://{}/v1/audio/transcriptions", config.host);

    // Convert MP4 to WAV; keep track of the temp file to clean up later
    let tmp_wav_pathbuf = convert_mp4_to_wav(audio_path).await?;
    let tmp_wav_file = tmp_wav_pathbuf.as_path();

    let audio_bytes = tokio::fs::read(tmp_wav_file)
        .await
        .map_err(|e| format!("failed to read audio file {}: {e}", tmp_wav_file.display()))?;

    let wav_filename = tmp_wav_file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("failed to get filename from: {}", tmp_wav_file.display()))?
        .to_owned();

    let file_part = reqwest::multipart::Part::bytes(audio_bytes)
        .file_name(wav_filename)
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = reqwest::multipart::Form::new()
        .text("model", config.model.clone())
        .part("file", file_part);

    if let Some(lang) = &config.language {
        form = form.text("language", lang.clone());
    }

    eprintln!(
        "[audiocpp] POST {url} model={}{} file={}",
        config.model,
        config
            .language
            .as_deref()
            .map(|l| format!(" language={l}"))
            .unwrap_or_default(),
        tmp_wav_file.display()
    );

    let response = reqwest::Client::new()
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("request to audiocpp failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        return Err(format!("audiocpp returned HTTP {status}: {body}"));
    }

    let body: TranscriptionResponse = response
        .json()
        .await
        .map_err(|e| format!("failed to parse audiocpp response: {e}"))?;

    // Clean up the temp WAV file.
    let _ = tokio::fs::remove_file(&tmp_wav_file).await;

    Ok(body)
}
