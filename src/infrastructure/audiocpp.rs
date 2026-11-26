#[derive(Clone)]
pub struct AudioCppConfig {
    pub host: String,
    pub model: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
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
