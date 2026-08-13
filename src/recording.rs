use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;

use crate::pebble::PebbleRecording;

#[derive(Debug, Serialize)]
pub struct AudioInfo {
    pub size_bytes: usize,
    pub path: Option<PathBuf>,
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
    pub audio: Option<AudioInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcriptions: Option<Vec<Transcription>>,
}

impl Recording {
    pub fn from(pebblerecording: &PebbleRecording, audio_path: Option<PathBuf>) -> Self {
        Recording {
            recording_source: pebblerecording.client.clone(),
            recorded_at: pebblerecording.recorded_at,
            audio: if let Some(audio_payload) = &pebblerecording.audio {
                Some(AudioInfo {
                    size_bytes: audio_payload.size_bytes,
                    path: audio_path,
                })
            } else {
                None
            },
            transcriptions: if let Some(pebbletranscription) = &pebblerecording.transcription {
                Some(vec![Transcription {
                    done_by: TranscriptionSource::PebbleApp,
                    content: pebbletranscription.clone(),
                }])
            } else {
                None
            },
        }
    }
}
