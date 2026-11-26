use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TranscriptionSource {
    PebbleApp,
    AudioCpp,
}

pub trait TranscriptionProvider {
    fn transcribe(&self, audio_path: &Path) -> String;
    pub async fn transcribe(
        config: &AudioCppConfig,
        audio_path: &Path,
    ) -> Result<TranscriptionResponse, String> {
}
