//! Exercises for module 13: one strict request, one operation, one local client.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateLabelRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Label {
    pub name: String,
}

pub trait LabelOperation: Send + Sync {
    fn create(&self, label: Label) -> Result<Label, String>;
}

pub fn decode_request(_json: &str) -> Result<Label, String> {
    todo!("separate JSON decoding from trimmed non-empty domain validation")
}

async fn create_label(
    State(_operation): State<Arc<dyn LabelOperation>>,
    Json(_request): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<Label>), ApiError> {
    todo!("validate the wire value and invoke the injected operation")
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

async fn malformed() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        "{not valid JSON}",
    )
}

pub async fn call_label(
    _client: &reqwest::Client,
    _base_url: &str,
    _name: &str,
) -> Result<Label, reqwest::Error> {
    todo!("POST JSON, validate status, then decode the response")
}

fn app(operation: Arc<dyn LabelOperation>) -> Router {
    Router::new()
        .route("/labels", post(create_label))
        .route("/malformed", get(malformed))
        .with_state(operation)
}

async fn serve(
    operation: Arc<dyn LabelOperation>,
) -> Result<
    (
        String,
        oneshot::Sender<()>,
        tokio::task::JoinHandle<std::io::Result<()>>,
    ),
    std::io::Error,
> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app(operation))
            .with_graceful_shutdown(async {
                let _ = shutdown_receiver.await;
            })
            .await
    });
    Ok((format!("http://{address}"), shutdown_sender, server))
}

fn main() {
    let _ = Duration::from_secs(2);
    let _ = serve;
    println!("Run `cargo test --example ex-13-rest-http` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;

    impl LabelOperation for Echo {
        fn create(&self, label: Label) -> Result<Label, String> {
            Ok(label)
        }
    }

    #[tokio::test]
    async fn strict_request_operation_client_and_cleanup() {
        assert!(decode_request(r#"{"name":"Rust","extra":true}"#).is_err());
        let (base_url, shutdown, server) = serve(Arc::new(Echo)).await.expect("server");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        assert_eq!(
            call_label(&client, &base_url, " Rust ")
                .await
                .expect("label"),
            Label {
                name: String::from("Rust")
            }
        );
        let malformed = client
            .get(format!("{base_url}/malformed"))
            .send()
            .await
            .expect("response")
            .error_for_status()
            .expect("success status")
            .json::<Label>()
            .await;
        assert!(malformed.is_err());

        let _ = shutdown.send(());
        server.await.expect("server task").expect("server shutdown");
    }
}
