use std::path::Path;

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

pub async fn transcribe(
    config: &AudioCppConfig,
    audio_path: &Path,
) -> Result<TranscriptionResponse, String> {
    let url = format!("http://{}/v1/audio/transcriptions", config.host);

    let audio_bytes = tokio::fs::read(audio_path)
        .await
        .map_err(|e| format!("failed to read audio file {}: {e}", audio_path.display()))?;

    let filename = audio_path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|stem| format!("{stem}.wav"))
        .unwrap_or_else(|| "audio.wav".to_string());

    let file_part = reqwest::multipart::Part::bytes(audio_bytes)
        .file_name(filename)
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
        audio_path.display()
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

    Ok(body)
}
