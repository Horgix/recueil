use reqwest::Client;
use url::Url;

#[derive(Clone)]
pub struct MatrixClient {
    http: Client,
    homeserver: String,
    access_token: String,
    room_id: String,
}

impl MatrixClient {
    pub fn new(homeserver: String, access_token: String, room_id: String) -> Self {
        MatrixClient {
            http: Client::new(),
            homeserver,
            access_token,
            room_id,
        }
    }

    pub async fn send_message(&self, text: &str) -> Result<(), String> {
        let txn_id = uuid::Uuid::new_v4();
        let mut url =
            Url::parse(&self.homeserver).map_err(|e| format!("invalid homeserver URL: {e}"))?;
        url.path_segments_mut()
            .map_err(|_| "homeserver URL cannot be a base".to_string())?
            .extend(&[
                "_matrix",
                "client",
                "v3",
                "rooms",
                &self.room_id,
                "send",
                "m.room.message",
                &txn_id.to_string(),
            ]);
        self.http
            .put(url)
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({ "msgtype": "m.text", "body": text }))
            .send()
            .await
            .map_err(|e| format!("Matrix request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Matrix returned an error: {e}"))?;
        Ok(())
    }
}
