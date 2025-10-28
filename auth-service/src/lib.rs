use std::error::Error;
use std::sync::Arc;
use axum::{
    http::{StatusCode, Method},
    response::{IntoResponse, Response},
    routing::post, 
    serve::Serve, 
    Router,
    Json
};
use tokio::net::TcpListener;
use app_state::AppState;
use domain::AuthAPIError;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, services::ServeDir};

pub mod routes;
pub mod domain;
pub mod services;
pub mod app_state;
pub mod utils;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Incorrect credentials"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token")
        };

        let body = serde_json::to_string(&ErrorResponse {
            error: error_message.to_string(),
        })
        .unwrap_or_else(|_| "{\"error\": \"Failed to serialize error message\"}".to_string());

        (status, [("Content-Type", "application/json")], body).into_response()
    }
}

// this struct encapsulates our application-related logic
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so that tests can access it
    pub address: String,
}

impl Application {
    pub async fn build(app_state: Arc<AppState>, address: &str) -> Result<Self, Box<dyn Error>> {
        let allow_origins = [
            "http:://localhost:8000".parse()?,
            // "http://[YOUR_DROPLET_IP]:8000".parse()?,
        ];
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allow_origins);

        let router = Router::new()
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token))
            .fallback_service(ServeDir::new("assets"))
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", self.address);
        self.server.await
    }
}