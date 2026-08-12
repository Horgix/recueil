use chrono::{DateTime, Utc};
use std::io;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::pebble::PebbleRecording;
use crate::recording::Recording;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H-%M-%SZ";

pub async fn save_metadata(dir: &Path, recording: &PebbleRecording) -> Result<(), io::Error> {
    let metadata = Recording::from(recording);
    let path = dir.join("recordings.ndjson");
    let line = serde_json::to_string(&metadata)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    file.write_all(format!("{line}\n").as_bytes()).await
}

pub async fn save_audio(
    dir: &Path,
    recorded_at: &DateTime<Utc>,
    audio: &[u8],
) -> Result<PathBuf, io::Error> {
    let filename = recorded_at.format(DATETIME_FORMAT).to_string();
    let path = dir.join("audio").join(format!("{filename}.mp4"));
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    tokio::fs::write(&path, audio).await?;
    println!("audio saved to {}", path.display());
    Ok(path)
}

pub async fn save_transcription(
    dir: &Path,
    recorded_at: &DateTime<Utc>,
    transcription: &str,
) -> Result<PathBuf, io::Error> {
    let filename = recorded_at.format(DATETIME_FORMAT).to_string();
    let path = dir.join("transcriptions").join(format!("{filename}.txt"));
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    tokio::fs::write(&path, format!("{transcription}\n")).await?;
    println!("transcription saved to {}", path.display());
    Ok(path)
}
