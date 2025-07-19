use axum::{
    extract::{Path, State, Query},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;

use crate::services::AppState;
use crate::models::{ReviewSession, Question, Answer, LearningProgress};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReviewSessionRequest {
    pub knowledge_base_id: String,
    pub questions_count: i32,
}

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub question: Question,
    pub answer: Answer,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub items: Vec<HistoryItem>,
    pub total_count: usize,
}

// Get random review question from history
pub async fn get_random_review_question(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // Get all question-answer history for this knowledge base
    let history = state.db.get_question_answer_history(&kb_id, None, None).await
        .map_err(|e| AppError::Database(e))?;
    
    if history.is_empty() {
        return Ok(Json(json!({
            "error": "No history available for review",
            "message": "Please complete some AI quizzes first to build up your review history"
        })));
    }
    
    // Randomly select a question from history
    let mut rng = rand::thread_rng();
    let (question, _) = history.choose(&mut rng).unwrap();
    
    Ok(Json(json!({
        "question": question,
        "message": "Review this question from your history"
    })))
}

// Get question-answer history with filtering
pub async fn get_history(
    Path(kb_id): Path<String>,
    Query(params): Query<HistoryQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<HistoryResponse>, AppError> {
    // Check if knowledge base exists
    let kb = state.db.get_knowledge_base_by_id(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    if kb.is_none() {
        return Err(AppError::Validation("Knowledge base not found".to_string()));
    }
    
    // Get filtered history
    let history = if params.min_score.is_some() || params.max_score.is_some() || 
                     params.start_date.is_some() || params.end_date.is_some() {
        state.db.get_filtered_history(
            &kb_id,
            params.min_score,
            params.max_score,
            params.start_date,
            params.end_date,
        ).await.map_err(|e| AppError::Database(e))?
    } else {
        state.db.get_question_answer_history(
            &kb_id,
            params.limit,
            params.offset,
        ).await.map_err(|e| AppError::Database(e))?
    };
    
    let items: Vec<HistoryItem> = history.into_iter().map(|(question, answer)| {
        HistoryItem { question, answer }
    }).collect();
    
    let total_count = items.len();
    
    Ok(Json(HistoryResponse {
        items,
        total_count,
    }))
}

// Create a new review session
pub async fn create_review_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateReviewSessionRequest>,
) -> Result<Json<Value>, AppError> {
    // Validate the request
    if payload.questions_count <= 0 {
        return Err(AppError::Validation("Questions count must be greater than 0".to_string()));
    }
    
    // Check if knowledge base exists
    let kb = state.db.get_knowledge_base_by_id(&payload.knowledge_base_id).await
        .map_err(|e| AppError::Database(e))?;
    
    if kb.is_none() {
        return Err(AppError::Validation("Knowledge base not found".to_string()));
    }
    
    // Check if there's enough history for the requested number of questions
    let history = state.db.get_question_answer_history(&payload.knowledge_base_id, None, None).await
        .map_err(|e| AppError::Database(e))?;
    
    if history.len() < payload.questions_count as usize {
        return Err(AppError::Validation(format!(
            "Not enough history available. Requested: {}, Available: {}",
            payload.questions_count,
            history.len()
        )));
    }
    
    // Create review session
    let session = ReviewSession::new(payload.knowledge_base_id, payload.questions_count);
    
    state.db.save_review_session(&session).await
        .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(json!({
        "session_id": session.id,
        "knowledge_base_id": session.knowledge_base_id,
        "questions_count": session.questions_count,
        "session_date": session.session_date,
        "message": "Review session created successfully"
    })))
}

// Get review sessions for a knowledge base
pub async fn get_review_sessions(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // Check if knowledge base exists
    let kb = state.db.get_knowledge_base_by_id(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    if kb.is_none() {
        return Err(AppError::Validation("Knowledge base not found".to_string()));
    }
    
    let sessions = state.db.get_review_sessions_by_knowledge_base(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(json!({
        "sessions": sessions,
        "total_count": sessions.len()
    })))
}

// Update review session with average score
pub async fn update_review_session_score(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let average_score = payload.get("average_score")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::Validation("Missing or invalid average_score".to_string()))?;
    
    if average_score < 0.0 || average_score > 100.0 {
        return Err(AppError::Validation("Average score must be between 0 and 100".to_string()));
    }
    
    let updated = state.db.update_review_session_score(&session_id, average_score).await
        .map_err(|e| AppError::Database(e))?;
    
    if !updated {
        return Err(AppError::Validation("Review session not found".to_string()));
    }
    
    Ok(Json(json!({
        "message": "Review session score updated successfully",
        "session_id": session_id,
        "average_score": average_score
    })))
}

#[derive(Debug, Deserialize)]
pub struct ReviewQuestionsRequest {
    pub count: Option<i32>,
}

// Get random questions for review session
pub async fn get_review_questions(
    Path(kb_id): Path<String>,
    Query(params): Query<ReviewQuestionsRequest>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let count = params.count.unwrap_or(5);
    
    if count <= 0 || count > 20 {
        return Err(AppError::Validation("Count must be between 1 and 20".to_string()));
    }
    
    // Check if knowledge base exists
    let kb = state.db.get_knowledge_base_by_id(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    if kb.is_none() {
        return Err(AppError::Validation("Knowledge base not found".to_string()));
    }
    
    // Get random questions from history
    let questions = state.db.get_random_review_questions(&kb_id, count).await
        .map_err(|e| AppError::Database(e))?;
    
    if questions.is_empty() {
        return Ok(Json(json!({
            "error": "No questions available for review",
            "message": "Please complete some AI quizzes first to build up your review history"
        })));
    }
    
    // Return only the questions (without previous answers for review)
    let review_questions: Vec<_> = questions.into_iter().map(|(question, _)| question).collect();
    
    Ok(Json(json!({
        "questions": review_questions,
        "count": review_questions.len(),
        "message": "Review questions retrieved successfully"
    })))
}

// Get learning progress for a knowledge base
pub async fn get_learning_progress(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<LearningProgress>, AppError> {
    // Check if knowledge base exists
    let kb = state.db.get_knowledge_base_by_id(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    if kb.is_none() {
        return Err(AppError::Validation("Knowledge base not found".to_string()));
    }
    
    let progress = state.db.get_learning_progress(&kb_id).await
        .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(progress))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewAnswerRequest {
    pub question_id: String,
    pub user_answer: String,
    pub session_id: Option<String>,
}

// Submit answer for review question
pub async fn submit_review_answer(
    State(state): State<AppState>,
    Json(payload): Json<ReviewAnswerRequest>,
) -> Result<Json<Value>, AppError> {
    // Validate input
    if payload.user_answer.trim().is_empty() {
        return Err(AppError::Validation("Answer cannot be empty".to_string()));
    }
    
    // Get the original question
    let question = state.db.get_question_by_id(&payload.question_id).await
        .map_err(|e| AppError::Database(e))?;
    
    let _question = question.ok_or_else(|| AppError::Validation("Question not found".to_string()))?;
    
    // Create new answer for the review
    let answer = Answer::new(payload.question_id.clone(), payload.user_answer.clone());
    
    // Save the review answer
    state.db.save_answer(&answer).await
        .map_err(|e| AppError::Database(e))?;
    
    // Get the knowledge base content for AI evaluation (if AI service is available)
    // For now, we'll return a simple response without AI evaluation
    // This can be enhanced later to integrate with the AI service
    
    Ok(Json(json!({
        "answer_id": answer.id,
        "question_id": payload.question_id,
        "user_answer": payload.user_answer,
        "submitted_at": answer.answered_at,
        "message": "Review answer submitted successfully",
        "note": "AI evaluation will be added in future updates"
    })))
}