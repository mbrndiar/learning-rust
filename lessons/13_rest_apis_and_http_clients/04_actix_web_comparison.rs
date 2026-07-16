//! Lesson 13.4: the same thin boundary in Actix Web.

use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, HttpServer, ResponseError, web};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UppercaseRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct UppercaseResponse {
    text: String,
}

trait UppercaseOperation: Send + Sync {
    fn uppercase(&self, text: &str) -> Result<String, ApiError>;
}

struct Uppercaser;

impl UppercaseOperation for Uppercaser {
    fn uppercase(&self, text: &str) -> Result<String, ApiError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ApiError);
        }
        Ok(text.to_uppercase())
    }
}

#[derive(Debug)]
struct ApiError;

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "text must not be empty")
    }
}

impl Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNPROCESSABLE_ENTITY
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({ "error": self.to_string() }))
    }
}

async fn uppercase(
    operation: web::Data<Arc<dyn UppercaseOperation>>,
    request: web::Json<UppercaseRequest>,
) -> Result<HttpResponse, ApiError> {
    let text = operation.uppercase(&request.text)?;
    Ok(HttpResponse::Created().json(UppercaseResponse { text }))
}

async fn run() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let address = listener.local_addr()?;
    let operation = Arc::new(Uppercaser) as Arc<dyn UppercaseOperation>;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&operation)))
            .route("/uppercase", web::post().to(uppercase))
    })
    .listen(listener)?
    .run();
    let handle = server.handle();
    let server_task = actix_web::rt::spawn(server);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let response = client
        .post(format!("http://{address}/uppercase"))
        .json(&serde_json::json!({ "text": " rust " }))
        .send()
        .await?
        .error_for_status()?;
    println!("{}", response.text().await?);

    // Blocking database or filesystem operations belong in `spawn_blocking`;
    // retain ownership and map its JoinError instead of detaching the work.
    let length = actix_web::rt::task::spawn_blocking(|| "blocking result".len()).await?;
    assert_eq!(length, 15);

    handle.stop(true).await;
    server_task.await??;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    actix_web::rt::System::new().block_on(run())
}
