use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;

use crate::pebble::PebbleRecording;

#[derive(Debug, Serialize)]
pub struct AudioInfo {
    pub size_bytes: usize,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum TranscriptionSource {
    PebbleApp,
    AudioCpp,
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
            audio: pebblerecording.audio.as_ref().map(|a| AudioInfo {
                size_bytes: a.size_bytes,
                path: audio_path,
            }),
            transcriptions: pebblerecording.transcription.as_ref().map(|t| {
                vec![Transcription {
                    done_by: TranscriptionSource::PebbleApp,
                    content: t.clone(),
                }]
            }),
        }
    }

    pub fn add_transcription(&mut self, done_by: TranscriptionSource, content: String) {
        self.transcriptions
            .get_or_insert_with(Vec::new)
            .push(Transcription { done_by, content });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pebble_recording_to_recording_without_anything() {
        let creation_time = DateTime::from_timestamp_millis(1_000_000).unwrap();
        let pebble_recording = PebbleRecording {
            client: "test_client".to_string(),
            recorded_at: creation_time,
            audio: None,
            transcription: None,
        };
        let recording = Recording::from(&pebble_recording, None);
        assert_eq!(recording.recording_source, "test_client");
        assert_eq!(recording.recorded_at, creation_time);
        assert!(recording.audio.is_none());
        assert!(recording.transcriptions.is_none());
    }

    #[test]
    fn pebble_recording_to_recording_with_audio() {
        let creation_time = DateTime::from_timestamp_millis(1_000_000).unwrap();
        let pebble_recording = PebbleRecording {
            client: "test_client".to_string(),
            recorded_at: creation_time,
            audio: Some(crate::pebble::AudioPayload {
                size_bytes: 123,
                data: bytes::Bytes::from_static(b"dummy audio data"),
            }),
            transcription: None,
        };
        let recording = Recording::from(&pebble_recording, None);
        assert!(recording.audio.is_some());
        assert_eq!(recording.audio.unwrap().size_bytes, 123);
    }

    #[test]
    fn pebble_recording_to_recording_with_transcription() {
        let creation_time = DateTime::from_timestamp_millis(1_000_000).unwrap();
        let pebble_recording = PebbleRecording {
            client: "test_client".to_string(),
            recorded_at: creation_time,
            audio: None,
            transcription: Some("dummy transcription".to_string()),
        };
        let recording = Recording::from(&pebble_recording, None);
        assert!(recording.transcriptions.is_some());
        let transcriptions = recording.transcriptions.unwrap();
        assert_eq!(transcriptions[0].done_by, TranscriptionSource::PebbleApp);
        assert_eq!(transcriptions[0].content, "dummy transcription");
    }

    #[test]
    fn pebble_recording_to_recording_with_audio_path() {
        let creation_time = DateTime::from_timestamp_millis(1_000_000).unwrap();
        let pebble_recording = PebbleRecording {
            client: "test_client".to_string(),
            recorded_at: creation_time,
            audio: Some(crate::pebble::AudioPayload {
                size_bytes: 123,
                data: bytes::Bytes::from_static(b"dummy audio data"),
            }),
            transcription: None,
        };
        let audio_path = Some(PathBuf::from("/tmp/path/to/audio.mp4"));
        let recording = Recording::from(&pebble_recording, audio_path.clone());
        assert!(recording.audio.is_some());
        let audio = recording.audio.unwrap();
        assert_eq!(audio.size_bytes, 123);
        assert_eq!(audio.path, audio_path);
    }
}
