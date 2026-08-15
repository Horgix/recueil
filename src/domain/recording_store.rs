use chrono::{DateTime, Utc};

use super::recording::Recording;

use std::{io, path::PathBuf};

pub trait RecordingStore {
    fn get_all_recordings(&self) -> Result<Vec<Recording>, io::Error>;
    async fn save_recording_info(&self, recording: &Recording) -> Result<(), io::Error>;
    // TODO: make it only take a recording as param?
    // TODO: should not return a PathBuf that is specifc to localstore
    async fn save_audio(
        &self,
        recorded_at: &DateTime<Utc>,
        audio: &[u8],
    ) -> Result<PathBuf, io::Error>;
    async fn save_transcription(
        &self,
        recorded_at: &DateTime<Utc>,
        transcription: &str,
    ) -> Result<PathBuf, io::Error>;
}
