use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::recording::Recording;

#[derive(Debug, Serialize)]
pub struct Transcription {
    pub done_by: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct RecordingMetadata {
    pub recording_source: String,
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcriptions: Option<Vec<Transcription>>,
}

impl From<&Recording> for RecordingMetadata {
    fn from(recording: &Recording) -> Self {
        let mut transcriptions = Vec::new();

        if let Some(content) = &recording.transcription {
            transcriptions.push(Transcription {
                done_by: "pebble_app".to_string(),
                content: content.clone(),
            });
        }

        let transcriptions = if transcriptions.is_empty() {
            None
        } else {
            Some(transcriptions)
        };

        RecordingMetadata {
            recording_source: recording.client.clone(),
            recorded_at: recording.recorded_at,
            transcriptions,
        }
    }
}
