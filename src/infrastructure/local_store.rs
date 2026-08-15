use crate::domain::recording::Recording;
use crate::domain::recording_store::RecordingStore;

use chrono::{DateTime, Utc};
use std::{
    fmt::Debug,
    io::{self, BufRead},
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H-%M-%SZ";

#[derive(Clone)]
pub struct LocalRecordingStore {
    path: PathBuf,
    audio_directory: PathBuf,
    transcriptions_directory: PathBuf,
    ndjson_path: PathBuf,
}

impl Debug for LocalRecordingStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LocalRecordingStore at {{ path: {} }}",
            self.path.display()
        )
    }
}

impl LocalRecordingStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        LocalRecordingStore {
            path: path.as_ref().to_path_buf(),
            audio_directory: path.as_ref().join("audio"),
            transcriptions_directory: path.as_ref().join("transcriptions"),
            ndjson_path: path.as_ref().join("recordings.ndjson"),
        }
    }
}

impl RecordingStore for LocalRecordingStore {
    async fn save_recording_info(&self, recording: &Recording) -> Result<(), io::Error> {
        let line = serde_json::to_string(&recording)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ndjson_path)
            .await?;
        file.write_all(format!("{line}\n").as_bytes()).await
    }

    async fn save_audio(
        &self,
        recorded_at: &DateTime<Utc>,
        audio: &[u8],
    ) -> Result<PathBuf, io::Error> {
        let filename = recorded_at.format(DATETIME_FORMAT).to_string();
        let path = self.audio_directory.join(format!("{filename}.mp4"));
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        tokio::fs::write(&path, audio).await?;
        println!("audio saved to {}", path.display());
        Ok(path)
    }

    async fn save_transcription(
        &self,
        recorded_at: &DateTime<Utc>,
        transcription: &str,
    ) -> Result<PathBuf, io::Error> {
        let filename = recorded_at.format(DATETIME_FORMAT).to_string();
        let path = self
            .transcriptions_directory
            .join(format!("{filename}.txt"));
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        tokio::fs::write(&path, format!("{transcription}\n")).await?;
        println!("transcription saved to {}", path.display());
        Ok(path)
    }

    fn get_all_recordings(&self) -> Result<Vec<Recording>, io::Error> {
        let file = std::fs::File::open(&self.ndjson_path)?;
        let reader = std::io::BufReader::new(file);
        let mut recordings = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let recording: Recording = serde_json::from_str(&line)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            recordings.push(recording);
        }
        Ok(recordings)
    }
}
