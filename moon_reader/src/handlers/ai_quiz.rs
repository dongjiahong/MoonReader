use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use validator::Validate;

use crate::services::{AppState, ai::{AIServiceFactory, AIProviderType}};
use crate::models::{Question, Answer, AIProvider};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AnswerRequest {
    #[validate(length(min = 1, max = 5000, message = "Answer must be between 1 and 5000 characters"))]
    pub user_answer: String,
}

#[derive(Debug, Serialize)]
pub struct QuestionResponse {
    pub id: String,
    pub question_text: String,
    pub context_snippet: Option<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Question> for QuestionResponse {
    fn from(question: Question) -> Self {
        Self {
            id: question.id,
            question_text: question.question_text,
            context_snippet: question.context_snippet,
            generated_at: question.generated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AnswerResponse {
    pub id: String,
    pub question_id: String,
    pub user_answer: String,
    pub ai_score: Option<i32>,
    pub ai_feedback: Option<String>,
    pub ai_suggestions: Vec<String>,
    pub answered_at: chrono::DateTime<chrono::Utc>,
}

impl From<Answer> for AnswerResponse {
    fn from(answer: Answer) -> Self {
        let suggestions = answer.ai_suggestions
            .map(|s| serde_json::from_str::<Vec<String>>(&s).unwrap_or_else(|_| vec![s]))
            .unwrap_or_default();
            
        Self {
            id: answer.id,
            question_id: answer.question_id,
            user_answer: answer.user_answer,
            ai_score: answer.ai_score,
            ai_feedback: answer.ai_feedback,
            ai_suggestions: suggestions,
            answered_at: answer.answered_at,
        }
    }
}

/// Generate a question based on knowledge base content
pub async fn generate_question(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify knowledge base exists
    let _knowledge_base = match state.db.get_knowledge_base_by_id(&kb_id).await {
        Ok(Some(kb)) => kb,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Knowledge base not found"})),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get knowledge base: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve knowledge base"})),
            ));
        }
    };

    // Get documents from the knowledge base
    let documents = match state.db.get_documents_by_knowledge_base(&kb_id).await {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("Failed to get documents: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve documents"})),
            ));
        }
    };

    if documents.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "No documents found in knowledge base. Please add some learning materials first."})),
        ));
    }

    // Combine document content for context
    let mut context = String::new();
    let mut context_snippet = String::new();
    
    for doc in &documents {
        if let Some(content) = &doc.content_text {
            context.push_str(content);
            context.push_str("\n\n");
            
            // Use first document's content as context snippet (truncated)
            if context_snippet.is_empty() {
                context_snippet = content.chars().take(500).collect();
                if content.len() > 500 {
                    context_snippet.push_str("...");
                }
            }
        }
    }

    if context.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "No content found in documents. Please ensure documents are properly parsed."})),
        ));
    }

    // Get AI configuration
    let ai_config = match state.db.get_ai_config().await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "AI not configured. Please configure AI settings first."})),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get AI config: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve AI configuration"})),
            ));
        }
    };

    // Create AI provider
    let provider_type = match ai_config.provider {
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
    
    match ai_config.provider {
        AIProvider::DeepSeek => {
            if let Some(api_key) = ai_config.api_key {
                provider_config.insert("api_key".to_string(), api_key);
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API key not configured for DeepSeek"})),
                ));
            }
        }
        AIProvider::Local => {
            if let Some(api_url) = ai_config.api_url {
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

    if let Some(model_name) = ai_config.model_name {
        provider_config.insert("model".to_string(), model_name);
    }
    provider_config.insert("max_tokens".to_string(), ai_config.max_tokens.to_string());
    provider_config.insert("temperature".to_string(), ai_config.temperature.to_string());

    let ai_provider = match AIServiceFactory::create_provider(provider_type, provider_config) {
        Ok(provider) => provider,
        Err(e) => {
            tracing::error!("Failed to create AI provider: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to create AI provider: {}", e)})),
            ));
        }
    };

    // Generate question using AI
    let question_text = match ai_provider.generate_question(&context).await {
        Ok(question) => question,
        Err(e) => {
            tracing::error!("Failed to generate question: {}", e);
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": format!("Failed to generate question: {}", e)})),
            ));
        }
    };

    // Create and save question
    let question = Question::new(
        kb_id,
        question_text,
        Some(context_snippet),
    );

    if let Err(e) = state.db.save_question(&question).await {
        tracing::error!("Failed to save question: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save question"})),
        ));
    }

    let response: QuestionResponse = question.into();
    Ok(Json(json!(response)))
}

/// Submit and evaluate an answer
pub async fn submit_answer(
    Path(question_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<AnswerRequest>,
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

    // Get the question
    let question = match state.db.get_question_by_id(&question_id).await {
        Ok(Some(q)) => q,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Question not found"})),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get question: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve question"})),
            ));
        }
    };

    // Get knowledge base documents for context
    let documents = match state.db.get_documents_by_knowledge_base(&question.knowledge_base_id).await {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("Failed to get documents: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve documents"})),
            ));
        }
    };

    // Combine document content for context
    let mut context = String::new();
    for doc in &documents {
        if let Some(content) = &doc.content_text {
            context.push_str(content);
            context.push_str("\n\n");
        }
    }

    // Get AI configuration
    let ai_config = match state.db.get_ai_config().await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "AI not configured. Please configure AI settings first."})),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to get AI config: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve AI configuration"})),
            ));
        }
    };

    // Create AI provider
    let provider_type = match ai_config.provider {
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
    
    match ai_config.provider {
        AIProvider::DeepSeek => {
            if let Some(api_key) = ai_config.api_key {
                provider_config.insert("api_key".to_string(), api_key);
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "API key not configured for DeepSeek"})),
                ));
            }
        }
        AIProvider::Local => {
            if let Some(api_url) = ai_config.api_url {
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

    if let Some(model_name) = ai_config.model_name {
        provider_config.insert("model".to_string(), model_name);
    }
    provider_config.insert("max_tokens".to_string(), ai_config.max_tokens.to_string());
    provider_config.insert("temperature".to_string(), ai_config.temperature.to_string());

    let ai_provider = match AIServiceFactory::create_provider(provider_type, provider_config) {
        Ok(provider) => provider,
        Err(e) => {
            tracing::error!("Failed to create AI provider: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to create AI provider: {}", e)})),
            ));
        }
    };

    // Evaluate answer using AI
    let evaluation = match ai_provider.evaluate_answer(
        &question.question_text,
        &payload.user_answer,
        &context,
    ).await {
        Ok(eval) => eval,
        Err(e) => {
            tracing::error!("Failed to evaluate answer: {}", e);
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": format!("Failed to evaluate answer: {}", e)})),
            ));
        }
    };

    // Create and save answer
    let mut answer = Answer::new(question_id, payload.user_answer);
    answer.ai_score = Some(evaluation.score as i32);
    answer.ai_feedback = Some(evaluation.feedback);
    answer.ai_suggestions = Some(serde_json::to_string(&evaluation.suggestions).unwrap_or_default());

    if let Err(e) = state.db.save_answer(&answer).await {
        tracing::error!("Failed to save answer: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save answer"})),
        ));
    }

    let response: AnswerResponse = answer.into();
    Ok(Json(json!(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_connection_pool;
    use crate::services::AppState;
    use crate::models::{KnowledgeBase, Document, DocumentType, AIConfig, AIProvider};
    use axum::extract::{Path, State};
    use tempfile::NamedTempFile;

    async fn create_test_app_state() -> AppState {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let pool = create_connection_pool(&database_url).await.unwrap();
        AppState::new(pool)
    }

    async fn setup_test_data(state: &AppState) -> (String, String) {
        // Create a knowledge base
        let kb = state.db.create_knowledge_base("Test KB", Some("Test description")).await.unwrap();
        
        // Create a document with content
        let document = Document::new(
            kb.id.clone(),
            "test.txt".to_string(),
            DocumentType::Txt,
            "/tmp/test.txt".to_string(),
            100,
            Some("This is test content for generating questions. It contains information about AI and machine learning.".to_string()),
        );
        state.db.save_document(&document).await.unwrap();
        
        // Create AI config
        let ai_config = AIConfig::new(
            AIProvider::DeepSeek,
            Some("test-api-key".to_string()),
            None,
            Some("deepseek-chat".to_string()),
            1000,
            0.7,
        );
        state.db.save_ai_config(&ai_config).await.unwrap();
        
        (kb.id, document.id)
    }

    #[tokio::test]
    async fn test_generate_question_no_knowledge_base() {
        let state = create_test_app_state().await;
        
        let result = generate_question(
            Path("non-existent-kb".to_string()),
            State(state),
        ).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_generate_question_no_documents() {
        let state = create_test_app_state().await;
        
        // Create a knowledge base without documents
        let kb = state.db.create_knowledge_base("Empty KB", None).await.unwrap();
        
        let result = generate_question(
            Path(kb.id),
            State(state),
        ).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_generate_question_no_ai_config() {
        let state = create_test_app_state().await;
        
        // Create knowledge base and document but no AI config
        let kb = state.db.create_knowledge_base("Test KB", None).await.unwrap();
        let document = Document::new(
            kb.id.clone(),
            "test.txt".to_string(),
            DocumentType::Txt,
            "/tmp/test.txt".to_string(),
            100,
            Some("Test content".to_string()),
        );
        state.db.save_document(&document).await.unwrap();
        
        let result = generate_question(
            Path(kb.id),
            State(state),
        ).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_submit_answer_question_not_found() {
        let state = create_test_app_state().await;
        
        let request = AnswerRequest {
            user_answer: "Test answer".to_string(),
        };
        
        let result = submit_answer(
            Path("non-existent-question".to_string()),
            State(state),
            Json(request),
        ).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_submit_answer_validation_error() {
        let state = create_test_app_state().await;
        
        let request = AnswerRequest {
            user_answer: "".to_string(), // Empty answer should fail validation
        };
        
        let result = submit_answer(
            Path("some-question-id".to_string()),
            State(state),
            Json(request),
        ).await;
        
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_answer_response_from_answer() {
        let answer = Answer {
            id: "test-id".to_string(),
            question_id: "question-id".to_string(),
            user_answer: "Test answer".to_string(),
            ai_score: Some(85),
            ai_feedback: Some("Good answer".to_string()),
            ai_suggestions: Some(r#"["Suggestion 1", "Suggestion 2"]"#.to_string()),
            answered_at: chrono::Utc::now(),
        };
        
        let response: AnswerResponse = answer.into();
        
        assert_eq!(response.id, "test-id");
        assert_eq!(response.question_id, "question-id");
        assert_eq!(response.user_answer, "Test answer");
        assert_eq!(response.ai_score, Some(85));
        assert_eq!(response.ai_feedback, Some("Good answer".to_string()));
        assert_eq!(response.ai_suggestions, vec!["Suggestion 1", "Suggestion 2"]);
    }

    #[tokio::test]
    async fn test_question_response_from_question() {
        let question = Question {
            id: "test-id".to_string(),
            knowledge_base_id: "kb-id".to_string(),
            question_text: "What is AI?".to_string(),
            context_snippet: Some("AI context".to_string()),
            generated_at: chrono::Utc::now(),
        };
        
        let response: QuestionResponse = question.into();
        
        assert_eq!(response.id, "test-id");
        assert_eq!(response.question_text, "What is AI?");
        assert_eq!(response.context_snippet, Some("AI context".to_string()));
    }
}