use axum_typed_multipart::{FieldData, TryFromMultipart};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Raw multipart form as sent by the Index webhook.
/// Field names map directly to the multipart field names from the spec.
#[derive(TryFromMultipart)]
pub struct RecordingForm {
    pub client: String,
    #[form_data(field_name = "recordedAt")]
    pub recorded_at_ms: u64,
    pub transcription: Option<String>,
    pub audio: Option<FieldData<Bytes>>,
}

impl RecordingForm {
    /// Convert into the domain type. Fail if the `X-Audio-Size` header
    /// disagrees with the actual buffered audio length or `recordedAt`
    /// timestamp is wrong
    pub fn into_recording(self, audio_size_hint: Option<usize>) -> Result<Recording, String> {
        let recorded_at = DateTime::from_timestamp_millis(self.recorded_at_ms as i64).ok_or(
            format!("invalid recordedAt timestamp: {}", self.recorded_at_ms),
        )?;

        let audio = if let Some(f) = self.audio {
            let size_bytes = f.contents.len();
            let hint = audio_size_hint.ok_or("audio present but X-Audio-Size header is missing")?;
            if hint != size_bytes {
                return Err(format!(
                    "audio size mismatch: X-Audio-Size={hint}, actual={size_bytes}"
                ));
            }
            Some(AudioPayload {
                size_bytes,
                data: f.contents,
            })
        } else {
            None
        };

        Ok(Recording {
            client: self.client,
            recorded_at,
            transcription: self.transcription,
            audio,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AudioPayload {
    pub size_bytes: usize,
    #[serde(skip)]
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct Recording {
    pub client: String,
    pub recorded_at: DateTime<Utc>,
    pub transcription: Option<String>,
    pub audio: Option<AudioPayload>,
}
