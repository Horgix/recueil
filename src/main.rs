use axum::extract::FromRequest;
use axum::{
    Router,
    extract::{Multipart, Request, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};

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
        .route("/", post(handle_request))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");

    println!("Listening on port {port}");
    axum::serve(listener, app).await.expect("server error");
}

async fn handle_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
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

    println!("============================================================");
    println!("> {} {}\n", request.method(), request.uri());

    println!("Headers:");
    for (name, value) in &headers {
        println!("  {}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
    println!();

    let content_type = match headers.get("content-type").and_then(|v| v.to_str().ok()) {
        Some(ct) => ct.to_string(),
        None => {
            eprintln!("REJECTED: missing Content-Type header");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if content_type.contains("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &())
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        println!("Multipart fields:");
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            eprintln!("multipart error: {e}");
            StatusCode::BAD_REQUEST
        })? {
            let name = field.name().unwrap_or("<unnamed>").to_string();
            let content_type = field
                .content_type()
                .map(|ct| format!(", content-type={ct}"))
                .unwrap_or_default();
            let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            match std::str::from_utf8(&bytes) {
                Ok(text) => println!("  [text] {name}{content_type} = {text:?}"),
                Err(_) => println!("  [bin]  {name}{content_type}, {} bytes", bytes.len()),
            }
        }
    } else {
        let body = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        if body.is_empty() {
            println!("(empty body)");
        } else {
            match std::str::from_utf8(&body) {
                Ok(text) => println!("Body ({} bytes):\n{text}", body.len()),
                Err(_) => println!("Body ({} bytes, binary)", body.len()),
            }
        }
    }

    println!();
    Ok("ok")
}
