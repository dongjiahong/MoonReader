// Data models module
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// Validation helper functions
fn validate_name_length(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new("name_empty"));
    }
    if name.len() > 255 {
        return Err(ValidationError::new("name_too_long"));
    }
    Ok(())
}

fn validate_file_size(size: i64) -> Result<(), ValidationError> {
    if size <= 0 {
        return Err(ValidationError::new("invalid_file_size"));
    }
    // Max file size: 100MB
    if size > 100 * 1024 * 1024 {
        return Err(ValidationError::new("file_too_large"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct KnowledgeBase {
    pub id: String,
    #[validate(custom = "validate_name_length")]
    pub name: String,
    #[validate(length(max = 1000, message = "Description too long"))]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KnowledgeBase {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Document {
    pub id: String,
    pub knowledge_base_id: String,
    #[validate(custom = "validate_name_length")]
    pub filename: String,
    pub file_type: DocumentType,
    pub file_path: String,
    #[validate(custom = "validate_file_size")]
    pub file_size: i64,
    pub content_text: Option<String>,
    pub upload_date: DateTime<Utc>,
}

impl Document {
    pub fn new(
        knowledge_base_id: String,
        filename: String,
        file_type: DocumentType,
        file_path: String,
        file_size: i64,
        content_text: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            knowledge_base_id,
            filename,
            file_type,
            file_path,
            file_size,
            content_text,
            upload_date: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
pub enum DocumentType {
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "epub")]
    Epub,
    #[serde(rename = "txt")]
    Txt,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Pdf => write!(f, "pdf"),
            DocumentType::Epub => write!(f, "epub"),
            DocumentType::Txt => write!(f, "txt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Question {
    pub id: String,
    pub knowledge_base_id: String,
    #[validate(length(min = 1, max = 2000, message = "Question text must be between 1 and 2000 characters"))]
    pub question_text: String,
    #[validate(length(max = 1000, message = "Context snippet too long"))]
    pub context_snippet: Option<String>,
    pub generated_at: DateTime<Utc>,
}

impl Question {
    pub fn new(
        knowledge_base_id: String,
        question_text: String,
        context_snippet: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            knowledge_base_id,
            question_text,
            context_snippet,
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Answer {
    pub id: String,
    pub question_id: String,
    #[validate(length(min = 1, max = 5000, message = "Answer must be between 1 and 5000 characters"))]
    pub user_answer: String,
    #[validate(range(min = 0, max = 100, message = "Score must be between 0 and 100"))]
    pub ai_score: Option<i32>,
    #[validate(length(max = 2000, message = "Feedback too long"))]
    pub ai_feedback: Option<String>,
    #[validate(length(max = 2000, message = "Suggestions too long"))]
    pub ai_suggestions: Option<String>,
    pub answered_at: DateTime<Utc>,
}

impl Answer {
    pub fn new(question_id: String, user_answer: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            question_id,
            user_answer,
            ai_score: None,
            ai_feedback: None,
            ai_suggestions: None,
            answered_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct ReviewSession {
    pub id: String,
    pub knowledge_base_id: String,
    #[validate(range(min = 1, message = "Questions count must be at least 1"))]
    pub questions_count: i32,
    #[validate(range(min = 0.0, max = 100.0, message = "Average score must be between 0 and 100"))]
    pub average_score: Option<f64>,
    pub session_date: DateTime<Utc>,
}

impl ReviewSession {
    pub fn new(knowledge_base_id: String, questions_count: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            knowledge_base_id,
            questions_count,
            average_score: None,
            session_date: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct AIConfig {
    pub id: Option<i32>,
    pub provider: AIProvider,
    #[validate(length(max = 500, message = "API key too long"))]
    pub api_key: Option<String>,
    #[validate(url(message = "Invalid API URL format"))]
    pub api_url: Option<String>,
    #[validate(length(max = 100, message = "Model name too long"))]
    pub model_name: Option<String>,
    #[validate(range(min = 1, max = 4000, message = "Max tokens must be between 1 and 4000"))]
    pub max_tokens: i32,
    #[validate(range(min = 0.0, max = 2.0, message = "Temperature must be between 0.0 and 2.0"))]
    pub temperature: f64,
    pub updated_at: DateTime<Utc>,
}

impl AIConfig {
    pub fn new(
        provider: AIProvider,
        api_key: Option<String>,
        api_url: Option<String>,
        model_name: Option<String>,
        max_tokens: i32,
        temperature: f64,
    ) -> Self {
        Self {
            id: None,
            provider,
            api_key,
            api_url,
            model_name,
            max_tokens,
            temperature,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
pub enum AIProvider {
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "openai")]
    OpenAI,
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIProvider::DeepSeek => write!(f, "deepseek"),
            AIProvider::Local => write!(f, "local"),
            AIProvider::OpenAI => write!(f, "openai"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgress {
    pub total_questions_answered: i32,
    pub average_score: Option<f64>,
    pub recent_average_score: Option<f64>,
    pub improvement_trend: Option<String>,
    pub total_review_sessions: i32,
}