use chrono::{DateTime, Utc};
use std::io;
use std::path::Path;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H-%M-%SZ";

pub async fn save_audio(
    dir: &Path,
    recorded_at: &DateTime<Utc>,
    audio: &[u8],
) -> Result<(), io::Error> {
    let filename = recorded_at.format(DATETIME_FORMAT).to_string();
    let path = dir.join("audio").join(format!("{filename}.mp4"));
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    tokio::fs::write(&path, audio).await?;
    println!("audio saved to {}", path.display());
    Ok(())
}

pub async fn save_transcription(
    dir: &Path,
    recorded_at: &DateTime<Utc>,
    transcription: &str,
) -> Result<(), io::Error> {
    let filename = recorded_at.format(DATETIME_FORMAT).to_string();
    let path = dir.join("transcriptions").join(format!("{filename}.txt"));
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    tokio::fs::write(&path, format!("{transcription}\n")).await?;
    println!("transcription saved to {}", path.display());
    Ok(())
}
