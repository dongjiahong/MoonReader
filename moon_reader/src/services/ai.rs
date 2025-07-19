use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AIError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEvaluation {
    pub score: u8,  // 0-100
    pub feedback: String,
    pub suggestions: Vec<String>,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a question based on the provided context
    async fn generate_question(&self, context: &str) -> Result<String, AIError>;
    
    /// Evaluate an answer given the question and context
    async fn evaluate_answer(
        &self,
        question: &str,
        answer: &str,
        context: &str,
    ) -> Result<AIEvaluation, AIError>;
    
    /// Test the connection to the AI service
    async fn test_connection(&self) -> Result<bool, AIError>;
}

#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl DeepSeekProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
        }
    }
    
    pub fn with_config(
        api_key: String,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
            max_tokens: max_tokens.unwrap_or(1000),
            temperature: temperature.unwrap_or(0.7),
        }
    }
    
    async fn make_request(&self, messages: Vec<ChatMessage>) -> Result<String, AIError> {
        let request_body = ChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };
        
        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::ApiError {
                status,
                message: error_text,
            });
        }
        
        let chat_response: ChatResponse = response.json().await?;
        
        chat_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| AIError::InvalidResponse("No content in response".to_string()))
    }
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    async fn generate_question(&self, context: &str) -> Result<String, AIError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some("你是一个专业的教育助手。基于提供的学习材料内容，生成一个有深度的问题来测试学习者对内容的理解。问题应该：1) 测试核心概念的理解 2) 需要综合思考 3) 避免简单的事实性问题。请只返回问题本身，不要包含其他解释。".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some(format!("基于以下学习材料生成一个问题：\n\n{}", context)),
            },
        ];
        
        self.make_request(messages).await
    }
    
    async fn evaluate_answer(
        &self,
        question: &str,
        answer: &str,
        context: &str,
    ) -> Result<AIEvaluation, AIError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some("你是一个专业的教育评估助手。请评估学习者的答案，并提供建设性的反馈。评估标准：准确性、完整性、深度。请以JSON格式返回评估结果，包含：score(0-100的整数)、feedback(详细反馈)、suggestions(改进建议数组)。".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some(format!(
                    "参考材料：\n{}\n\n问题：{}\n\n学习者答案：{}\n\n请评估这个答案并返回JSON格式的评估结果。",
                    context, question, answer
                )),
            },
        ];
        
        let response = self.make_request(messages).await?;
        
        // Try to parse as JSON
        match serde_json::from_str::<AIEvaluation>(&response) {
            Ok(evaluation) => Ok(evaluation),
            Err(_) => {
                // If JSON parsing fails, try to extract information from text response
                Ok(AIEvaluation {
                    score: 70, // Default score
                    feedback: response,
                    suggestions: vec!["请参考参考材料进一步完善答案".to_string()],
                })
            }
        }
    }
    
    async fn test_connection(&self) -> Result<bool, AIError> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: Some("Hello, this is a connection test.".to_string()),
        }];
        
        match self.make_request(messages).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalAIProvider {
    api_url: String,
    client: reqwest::Client,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl LocalAIProvider {
    pub fn new(api_url: String) -> Self {
        Self {
            api_url,
            client: reqwest::Client::new(),
            model: "local-model".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
        }
    }
    
    pub fn with_config(
        api_url: String,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            api_url,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "local-model".to_string()),
            max_tokens: max_tokens.unwrap_or(1000),
            temperature: temperature.unwrap_or(0.7),
        }
    }
    
    async fn make_request(&self, messages: Vec<ChatMessage>) -> Result<String, AIError> {
        let request_body = ChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };
        
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.api_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::ApiError {
                status,
                message: error_text,
            });
        }
        
        let chat_response: ChatResponse = response.json().await?;
        
        chat_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| AIError::InvalidResponse("No content in response".to_string()))
    }
}

#[async_trait]
impl AIProvider for LocalAIProvider {
    async fn generate_question(&self, context: &str) -> Result<String, AIError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some("You are a professional educational assistant. Based on the provided learning material content, generate a thoughtful question to test the learner's understanding. The question should: 1) Test understanding of core concepts 2) Require comprehensive thinking 3) Avoid simple factual questions. Please return only the question itself without other explanations.".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some(format!("Generate a question based on the following learning material:\n\n{}", context)),
            },
        ];
        
        self.make_request(messages).await
    }
    
    async fn evaluate_answer(
        &self,
        question: &str,
        answer: &str,
        context: &str,
    ) -> Result<AIEvaluation, AIError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some("You are a professional educational assessment assistant. Please evaluate the learner's answer and provide constructive feedback. Evaluation criteria: accuracy, completeness, depth. Please return the evaluation result in JSON format, including: score (integer 0-100), feedback (detailed feedback), suggestions (array of improvement suggestions).".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: Some(format!(
                    "Reference material:\n{}\n\nQuestion: {}\n\nLearner's answer: {}\n\nPlease evaluate this answer and return a JSON-formatted evaluation result.",
                    context, question, answer
                )),
            },
        ];
        
        let response = self.make_request(messages).await?;
        
        // Try to parse as JSON
        match serde_json::from_str::<AIEvaluation>(&response) {
            Ok(evaluation) => Ok(evaluation),
            Err(_) => {
                // If JSON parsing fails, try to extract information from text response
                Ok(AIEvaluation {
                    score: 70, // Default score
                    feedback: response,
                    suggestions: vec!["Please refer to the reference material to further improve your answer".to_string()],
                })
            }
        }
    }
    
    async fn test_connection(&self) -> Result<bool, AIError> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: Some("Hello, this is a connection test.".to_string()),
        }];
        
        match self.make_request(messages).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

// Data structures for API communication
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

// AI Service Factory
#[derive(Debug, Clone)]
pub enum AIProviderType {
    DeepSeek,
    Local,
}

pub struct AIServiceFactory;

impl AIServiceFactory {
    pub fn create_provider(
        provider_type: AIProviderType,
        config: HashMap<String, String>,
    ) -> Result<Box<dyn AIProvider>, AIError> {
        match provider_type {
            AIProviderType::DeepSeek => {
                let api_key = config
                    .get("api_key")
                    .ok_or_else(|| AIError::ConfigError("Missing API key for DeepSeek".to_string()))?
                    .clone();
                
                let model = config.get("model").cloned();
                let max_tokens = config
                    .get("max_tokens")
                    .and_then(|s| s.parse().ok());
                let temperature = config
                    .get("temperature")
                    .and_then(|s| s.parse().ok());
                
                Ok(Box::new(DeepSeekProvider::with_config(
                    api_key, model, max_tokens, temperature,
                )))
            }
            AIProviderType::Local => {
                let api_url = config
                    .get("api_url")
                    .ok_or_else(|| AIError::ConfigError("Missing API URL for Local AI".to_string()))?
                    .clone();
                
                let model = config.get("model").cloned();
                let max_tokens = config
                    .get("max_tokens")
                    .and_then(|s| s.parse().ok());
                let temperature = config
                    .get("temperature")
                    .and_then(|s| s.parse().ok());
                
                Ok(Box::new(LocalAIProvider::with_config(
                    api_url, model, max_tokens, temperature,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_deepseek_provider_creation() {
        let provider = DeepSeekProvider::new("test-key".to_string());
        assert_eq!(provider.api_key, "test-key");
        assert_eq!(provider.model, "deepseek-chat");
    }
    
    #[tokio::test]
    async fn test_local_provider_creation() {
        let provider = LocalAIProvider::new("http://localhost:8080".to_string());
        assert_eq!(provider.api_url, "http://localhost:8080");
        assert_eq!(provider.model, "local-model");
    }
    
    #[tokio::test]
    async fn test_ai_service_factory() {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "test-key".to_string());
        
        let provider = AIServiceFactory::create_provider(AIProviderType::DeepSeek, config);
        assert!(provider.is_ok());
    }
    
    #[tokio::test]
    async fn test_ai_service_factory_missing_config() {
        let config = HashMap::new();
        
        let provider = AIServiceFactory::create_provider(AIProviderType::DeepSeek, config);
        assert!(provider.is_err());
    }
}