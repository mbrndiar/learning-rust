//! Lesson 13.2: an Axum route with extraction, state, and error mapping.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateGreetingRequest {
    name: String,
}

#[derive(Debug, Serialize)]
struct GreetingResponse {
    message: String,
}

trait GreetingOperation: Send + Sync {
    fn greet(&self, name: &str) -> Result<String, ApiError>;
}

struct Greeter;

impl GreetingOperation for Greeter {
    fn greet(&self, name: &str) -> Result<String, ApiError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
        Ok(format!("Hello, {name}!"))
    }
}

#[derive(Debug)]
enum ApiError {
    InvalidName,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidName => (StatusCode::UNPROCESSABLE_ENTITY, "name must not be empty"),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

async fn create_greeting(
    State(operation): State<Arc<dyn GreetingOperation>>,
    Json(request): Json<CreateGreetingRequest>,
) -> Result<(StatusCode, Json<GreetingResponse>), ApiError> {
    let message = operation.greet(&request.name)?;
    Ok((StatusCode::CREATED, Json(GreetingResponse { message })))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/greetings", post(create_greeting))
        .with_state(Arc::new(Greeter) as Arc<dyn GreetingOperation>);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_receiver.await;
            })
            .await
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let response = client
        .post(format!("http://{address}/greetings"))
        .json(&serde_json::json!({ "name": "Ada" }))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    println!("{}", response.text().await?);

    let _ = shutdown_sender.send(());
    server.await??;
    Ok(())
}
