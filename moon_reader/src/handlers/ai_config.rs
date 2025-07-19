use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use validator::Validate;

use crate::services::{AppState, ai::{AIServiceFactory, AIProviderType}};
use crate::models::{AIConfig, AIProvider};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AIConfigRequest {
    pub provider: AIProvider,
    #[validate(length(max = 500, message = "API key too long"))]
    pub api_key: Option<String>,
    #[validate(url(message = "Invalid API URL format"))]
    pub api_url: Option<String>,
    #[validate(length(max = 100, message = "Model name too long"))]
    pub model_name: Option<String>,
    #[validate(range(min = 1, max = 4000, message = "Max tokens must be between 1 and 4000"))]
    pub max_tokens: Option<i32>,
    #[validate(range(min = 0.0, max = 2.0, message = "Temperature must be between 0.0 and 2.0"))]
    pub temperature: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AIConfigResponse {
    pub provider: AIProvider,
    pub api_key_configured: bool,
    pub api_url: Option<String>,
    pub model_name: Option<String>,
    pub max_tokens: i32,
    pub temperature: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<AIConfig> for AIConfigResponse {
    fn from(config: AIConfig) -> Self {
        Self {
            provider: config.provider,
            api_key_configured: config.api_key.is_some(),
            api_url: config.api_url,
            model_name: config.model_name,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            updated_at: config.updated_at,
        }
    }
}

/// Get current AI configuration
pub async fn get_ai_config(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.db.get_ai_config().await {
        Ok(Some(config)) => {
            let response: AIConfigResponse = config.into();
            Ok(Json(json!(response)))
        }
        Ok(None) => {
            // Return default configuration if none exists
            Ok(Json(json!({
                "provider": "deepseek",
                "api_key_configured": false,
                "api_url": null,
                "model_name": null,
                "max_tokens": 1000,
                "temperature": 0.7,
                "updated_at": null
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get AI config: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve AI configuration"})),
            ))
        }
    }
}

/// Save AI configuration
pub async fn save_ai_config(
    State(state): State<AppState>,
    Json(payload): Json<AIConfigRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate the request
    if let Err(validation_errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": validation_errors.to_string()
            })),
        ));
    }

    // Validate provider-specific requirements
    match payload.provider {
        AIProvider::DeepSeek => {
            if payload.api_key.is_none() || payload.api_key.as_ref().unwrap().trim().is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API key is required for DeepSeek provider"})),
                ));
            }
        }
        AIProvider::Local => {
            if payload.api_url.is_none() || payload.api_url.as_ref().unwrap().trim().is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API URL is required for Local AI provider"})),
                ));
            }
        }
        AIProvider::OpenAI => {
            if payload.api_key.is_none() || payload.api_key.as_ref().unwrap().trim().is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API key is required for OpenAI provider"})),
                ));
            }
        }
    }

    let config = AIConfig::new(
        payload.provider,
        payload.api_key,
        payload.api_url,
        payload.model_name,
        payload.max_tokens.unwrap_or(1000),
        payload.temperature.unwrap_or(0.7),
    );

    match state.db.save_ai_config(&config).await {
        Ok(_) => {
            let response: AIConfigResponse = config.into();
            Ok(Json(json!({
                "message": "AI configuration saved successfully",
                "config": response
            })))
        }
        Err(e) => {
            tracing::error!("Failed to save AI config: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to save AI configuration"})),
            ))
        }
    }
}

/// Test AI connection with current configuration
pub async fn test_ai_connection(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get current AI configuration
    let config = match state.db.get_ai_config().await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "No AI configuration found. Please configure AI settings first."})),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get AI config for testing: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve AI configuration"})),
            ));
        }
    };

    // Create AI provider based on configuration
    let provider_type = match config.provider {
        AIProvider::DeepSeek => AIProviderType::DeepSeek,
        AIProvider::Local => AIProviderType::Local,
        AIProvider::OpenAI => {
            return Err((
                StatusCode::NOT_IMPLEMENTED,
                Json(json!({"error": "OpenAI provider not yet implemented"})),
            ));
        }
    };

    let mut provider_config = HashMap::new();
    
    match config.provider {
        AIProvider::DeepSeek => {
            if let Some(api_key) = config.api_key {
                provider_config.insert("api_key".to_string(), api_key);
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API key not configured for DeepSeek"})),
                ));
            }
        }
        AIProvider::Local => {
            if let Some(api_url) = config.api_url {
                provider_config.insert("api_url".to_string(), api_url);
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API URL not configured for Local AI"})),
                ));
            }
        }
        _ => {}
    }

    if let Some(model_name) = config.model_name {
        provider_config.insert("model".to_string(), model_name);
    }
    provider_config.insert("max_tokens".to_string(), config.max_tokens.to_string());
    provider_config.insert("temperature".to_string(), config.temperature.to_string());

    // Create AI provider and test connection
    match AIServiceFactory::create_provider(provider_type, provider_config) {
        Ok(provider) => {
            match provider.test_connection().await {
                Ok(true) => {
                    Ok(Json(json!({
                        "status": "success",
                        "message": "AI service connection successful",
                        "provider": config.provider.to_string()
                    })))
                }
                Ok(false) => {
                    Err((
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "status": "failed",
                            "message": "AI service connection failed",
                            "provider": config.provider.to_string()
                        })),
                    ))
                }
                Err(e) => {
                    tracing::error!("AI connection test error: {}", e);
                    Err((
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "status": "error",
                            "message": format!("AI service error: {}", e),
                            "provider": config.provider.to_string()
                        })),
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create AI provider: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to create AI provider: {}", e)
                })),
            ))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_connection_pool;
    use crate::services::AppState;
    use axum::extract::State;
    use tempfile::NamedTempFile;

    async fn create_test_app_state() -> AppState {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let pool = create_connection_pool(&database_url).await.unwrap();
        AppState::new(pool)
    }

    #[tokio::test]
    async fn test_get_ai_config_default() {
        let state = create_test_app_state().await;
        let result = get_ai_config(State(state)).await;
        
        match &result {
            Ok(_) => {},
            Err((status, json)) => {
                println!("Error status: {:?}", status);
                println!("Error body: {:?}", json.0);
            }
        }
        
        assert!(result.is_ok());
        let response = result.unwrap();
        let json_value: serde_json::Value = response.0;
        
        assert_eq!(json_value["provider"], "deepseek");
        assert_eq!(json_value["api_key_configured"], false);
    }

    #[tokio::test]
    async fn test_save_ai_config_deepseek() {
        let state = create_test_app_state().await;
        
        let request = AIConfigRequest {
            provider: AIProvider::DeepSeek,
            api_key: Some("test-api-key".to_string()),
            api_url: None,
            model_name: Some("deepseek-chat".to_string()),
            max_tokens: Some(1500),
            temperature: Some(0.8),
        };
        
        let result = save_ai_config(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());
        
        // Verify the config was saved
        let get_result = get_ai_config(State(state)).await;
        assert!(get_result.is_ok());
        
        let response = get_result.unwrap();
        let json_value: serde_json::Value = response.0;
        
        assert_eq!(json_value["provider"], "deepseek");
        assert_eq!(json_value["api_key_configured"], true);
        assert_eq!(json_value["model_name"], "deepseek-chat");
        assert_eq!(json_value["max_tokens"], 1500);
        assert_eq!(json_value["temperature"], 0.8);
    }

    #[tokio::test]
    async fn test_save_ai_config_local() {
        let state = create_test_app_state().await;
        
        let request = AIConfigRequest {
            provider: AIProvider::Local,
            api_key: None,
            api_url: Some("http://localhost:8080".to_string()),
            model_name: Some("local-model".to_string()),
            max_tokens: Some(2000),
            temperature: Some(0.5),
        };
        
        let result = save_ai_config(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());
        
        // Verify the config was saved
        let get_result = get_ai_config(State(state)).await;
        assert!(get_result.is_ok());
        
        let response = get_result.unwrap();
        let json_value: serde_json::Value = response.0;
        
        assert_eq!(json_value["provider"], "local");
        assert_eq!(json_value["api_key_configured"], false);
        assert_eq!(json_value["api_url"], "http://localhost:8080");
        assert_eq!(json_value["model_name"], "local-model");
        assert_eq!(json_value["max_tokens"], 2000);
        assert_eq!(json_value["temperature"], 0.5);
    }

    #[tokio::test]
    async fn test_save_ai_config_validation_error() {
        let state = create_test_app_state().await;
        
        let request = AIConfigRequest {
            provider: AIProvider::DeepSeek,
            api_key: None, // Missing required API key for DeepSeek
            api_url: None,
            model_name: None,
            max_tokens: Some(1000),
            temperature: Some(0.7),
        };
        
        let result = save_ai_config(State(state), Json(request)).await;
        assert!(result.is_err());
        
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_test_ai_connection_no_config() {
        let state = create_test_app_state().await;
        
        let result = test_ai_connection(State(state)).await;
        assert!(result.is_err());
        
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}