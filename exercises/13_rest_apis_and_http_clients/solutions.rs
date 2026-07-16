//! Reference solution for module 13.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateLabelRequest {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Label {
    name: String,
}

impl TryFrom<CreateLabelRequest> for Label {
    type Error = String;

    fn try_from(request: CreateLabelRequest) -> Result<Self, Self::Error> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(String::from("name must not be empty"));
        }
        Ok(Self {
            name: name.to_owned(),
        })
    }
}

fn decode_request(json: &str) -> Result<Label, String> {
    let request: CreateLabelRequest =
        serde_json::from_str(json).map_err(|error| format!("invalid JSON: {error}"))?;
    Label::try_from(request)
}

trait LabelOperation: Send + Sync {
    fn create(&self, label: Label) -> Result<Label, String>;
}

struct Prefix {
    prefix: String,
}

impl LabelOperation for Prefix {
    fn create(&self, label: Label) -> Result<Label, String> {
        Ok(Label {
            name: format!("{}{}", self.prefix, label.name),
        })
    }
}

#[derive(Debug)]
struct ApiError(String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": self.0 })),
        )
            .into_response()
    }
}

async fn create_label(
    State(operation): State<Arc<dyn LabelOperation>>,
    Json(request): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<Label>), ApiError> {
    let label = Label::try_from(request).map_err(ApiError)?;
    let created = operation.create(label).map_err(ApiError)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn malformed() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        "{not valid JSON}",
    )
}

async fn call_label(
    client: &reqwest::Client,
    base_url: &str,
    name: &str,
) -> Result<Label, reqwest::Error> {
    client
        .post(format!("{base_url}/labels"))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    assert!(decode_request(r#"{"name":"Rust","extra":true}"#).is_err());

    let operation = Arc::new(Prefix {
        prefix: String::from("topic:"),
    }) as Arc<dyn LabelOperation>;
    let app = Router::new()
        .route("/labels", post(create_label))
        .route("/malformed", get(malformed))
        .with_state(operation);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let base_url = format!("http://{address}");
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
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let label = call_label(&client, &base_url, " Rust ").await?;
    assert_eq!(label.name, "topic:Rust");

    let malformed = client
        .get(format!("{base_url}/malformed"))
        .send()
        .await?
        .error_for_status()?
        .json::<Label>()
        .await;
    assert!(malformed.is_err());

    let _ = shutdown_sender.send(());
    server.await??;
    println!("Module 13 REST/HTTP solution passed.");
    Ok(())
}
