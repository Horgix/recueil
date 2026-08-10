use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use axum_typed_multipart::TypedMultipart;

mod recording;
use recording::RecordingForm;

#[derive(Clone)]
struct AppState {
    auth_token: String,
}

#[tokio::main]
async fn main() {
    let auth_token = match std::env::var("AUTH_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => {
            eprintln!("AUTH_TOKEN is not set, refusing to start");
            std::process::exit(1);
        }
    };

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid port number");

    let state = AppState { auth_token };

    let app = Router::new()
        .route("/", post(handle_webhook))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");

    println!("Listening on port {port}");
    axum::serve(listener, app).await.expect("server error");
}

async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    TypedMultipart(form): TypedMultipart<RecordingForm>,
) -> Result<&'static str, StatusCode> {
    let expected = format!("Bearer {}", state.auth_token);
    match headers.get("authorization") {
        None => {
            eprintln!("REJECTED: missing Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Some(v) => match v.to_str() {
            Err(_) => {
                eprintln!("REJECTED: Authorization header is invalid");
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(auth) if auth != expected => {
                eprintln!("REJECTED: Invalid Authorization header; got: {auth:?}");
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(_) => {}
        },
    }

    let audio_size_hint = headers
        .get("x-audio-size")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    let recording = form.into_recording(audio_size_hint);
    println!("{recording:#?}");

    Ok("ok")
}
