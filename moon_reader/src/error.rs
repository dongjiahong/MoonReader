// Unified error handling for the application
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("File upload error: {0}")]
    FileUpload(String),
    
    #[error("Document parsing error: {0}")]
    DocumentParse(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database operation failed".to_string(),
                    Some(e.to_string()),
                )
            }
            AppError::Validation(ref msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    "Validation failed".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::NotFound(ref msg) => {
                (
                    StatusCode::NOT_FOUND,
                    "Resource not found".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::BadRequest(ref msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    "Bad request".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::ServiceUnavailable(ref msg) => {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service unavailable".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::FileUpload(ref msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    "File upload failed".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::DocumentParse(ref msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    "Document parsing failed".to_string(),
                    Some(msg.clone()),
                )
            }
        };
        
        let body = Json(json!({
            "error": error_message,
            "details": details
        }));
        
        (status, body).into_response()
    }
}

// Helper function to convert validation errors to AppError
pub fn validation_error_to_app_error(errors: validator::ValidationErrors) -> AppError {
    let error_messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |error| {
                format!("{}: {}", field, error.message.as_ref().unwrap_or(&"Invalid value".into()))
            })
        })
        .collect();
    
    AppError::Validation(error_messages.join(", "))
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;