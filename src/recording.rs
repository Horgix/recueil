use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::pebble::PebbleRecording;

#[derive(Serialize)]
pub struct AudioPayload {
    pub size_bytes: usize,
    #[serde(skip)]
    pub data: Bytes,
}

impl std::fmt::Debug for AudioPayload {
    // Avoid printint the entire audio raw bytes
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPayload")
            .field("size_bytes", &self.size_bytes)
            .finish()
    }
}

#[derive(Debug, Serialize)]
pub enum TranscriptionSource {
    PebbleApp,
}

#[derive(Debug, Serialize)]
pub struct Transcription {
    pub done_by: TranscriptionSource,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Recording {
    pub recording_source: String,
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcriptions: Option<Vec<Transcription>>,
}

impl From<&PebbleRecording> for Recording {
    fn from(recording: &PebbleRecording) -> Self {
        let mut transcriptions = Vec::new();

        if let Some(content) = &recording.transcription {
            transcriptions.push(Transcription {
                done_by: TranscriptionSource::PebbleApp,
                content: content.clone(),
            });
        }

        let transcriptions = if transcriptions.is_empty() {
            None
        } else {
            Some(transcriptions)
        };

        Recording {
            recording_source: recording.client.clone(),
            recorded_at: recording.recorded_at,
            transcriptions,
        }
    }
}
