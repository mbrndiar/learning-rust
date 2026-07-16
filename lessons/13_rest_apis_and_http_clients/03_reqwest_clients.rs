//! Lesson 13.3: a bounded Reqwest client calling an ephemeral local API.

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Deserialize)]
struct LookupQuery {
    term: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LookupResponse {
    normalized: String,
}

async fn lookup(Query(query): Query<LookupQuery>) -> Json<LookupResponse> {
    Json(LookupResponse {
        normalized: query.term.trim().to_lowercase(),
    })
}

async fn broken() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        "{not valid JSON}",
    )
}

async fn fetch_lookup(
    client: &reqwest::Client,
    base_url: &str,
    term: &str,
) -> Result<LookupResponse, reqwest::Error> {
    client
        .get(format!("{base_url}/lookup"))
        .query(&[("term", term)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/lookup", get(lookup))
        .route("/broken", get(broken));
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
    let result = fetch_lookup(&client, &base_url, " Rust Language ").await?;
    println!("normalized={}", result.normalized);

    let malformed = client
        .get(format!("{base_url}/broken"))
        .send()
        .await?
        .error_for_status()?
        .json::<LookupResponse>()
        .await;
    assert!(malformed.is_err());
    println!("malformed body rejected={}", malformed.is_err());

    let _ = shutdown_sender.send(());
    server.await??;
    Ok(())
}
