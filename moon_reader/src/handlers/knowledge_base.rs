use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::models::KnowledgeBase;
use crate::services::AppState;
use crate::error::{AppError, AppResult, validation_error_to_app_error};

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateKnowledgeBaseRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateKnowledgeBaseRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct KnowledgeBaseResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub document_count: i64,
}

impl From<KnowledgeBase> for KnowledgeBaseResponse {
    fn from(kb: KnowledgeBase) -> Self {
        Self {
            id: kb.id,
            name: kb.name,
            description: kb.description,
            created_at: kb.created_at.to_rfc3339(),
            updated_at: kb.updated_at.to_rfc3339(),
            document_count: 0, // Will be populated separately if needed
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListKnowledgeBasesResponse {
    pub knowledge_bases: Vec<KnowledgeBaseResponse>,
}



// Handler functions
pub async fn list_knowledge_bases(
    State(state): State<AppState>
) -> AppResult<Json<ListKnowledgeBasesResponse>> {
    let knowledge_bases = state.db.get_knowledge_bases().await?;
    let response = ListKnowledgeBasesResponse {
        knowledge_bases: knowledge_bases.into_iter().map(KnowledgeBaseResponse::from).collect(),
    };
    
    Ok(Json(response))
}

pub async fn create_knowledge_base(
    State(state): State<AppState>,
    Json(payload): Json<CreateKnowledgeBaseRequest>,
) -> AppResult<Json<KnowledgeBaseResponse>> {
    // Validate input
    if let Err(validation_errors) = payload.validate() {
        return Err(validation_error_to_app_error(validation_errors));
    }
    
    let knowledge_base = state.db.create_knowledge_base(&payload.name, payload.description.as_deref()).await?;
    
    tracing::info!("Created knowledge base: {}", knowledge_base.id);
    Ok(Json(KnowledgeBaseResponse::from(knowledge_base)))
}

pub async fn update_knowledge_base(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateKnowledgeBaseRequest>,
) -> AppResult<Json<KnowledgeBaseResponse>> {
    // Validate input
    if let Err(validation_errors) = payload.validate() {
        return Err(validation_error_to_app_error(validation_errors));
    }
    
    // Check if knowledge base exists
    let existing_kb = state.db.get_knowledge_base_by_id(&id).await?;
    if existing_kb.is_none() {
        return Err(AppError::NotFound("Knowledge base not found".to_string()));
    }
    
    // Update the knowledge base
    let updated = state.db.update_knowledge_base(&id, &payload.name, payload.description.as_deref()).await?;
    if !updated {
        return Err(AppError::NotFound("Knowledge base not found".to_string()));
    }
    
    // Fetch the updated knowledge base
    let updated_kb = state.db.get_knowledge_base_by_id(&id).await?
        .ok_or_else(|| AppError::Internal("Failed to retrieve updated knowledge base".to_string()))?;
    
    tracing::info!("Updated knowledge base: {}", id);
    Ok(Json(KnowledgeBaseResponse::from(updated_kb)))
}

pub async fn delete_knowledge_base(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<Value>> {
    let deleted = state.db.delete_knowledge_base(&id).await?;
    if !deleted {
        return Err(AppError::NotFound("Knowledge base not found".to_string()));
    }
    
    tracing::info!("Deleted knowledge base: {}", id);
    Ok(Json(json!({"message": "Knowledge base deleted successfully"})))
}