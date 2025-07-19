mod handlers;
mod models;
mod services;
mod database;
mod parsers;
mod error;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use std::env;

use crate::handlers::*;
use crate::services::AppState;
use crate::database::create_connection_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:knowledge_system.db".to_string());
    
    // Create database connection pool
    let pool = create_connection_pool(&database_url).await?;
    
    // Create application state
    let app_state = AppState::new(pool);
    
    // Build our application with routes
    let app = create_app().with_state(app_state);
    
    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_app() -> Router<AppState> {
    Router::new()
        // Knowledge base routes
        .route("/api/knowledge-bases", 
               get(list_knowledge_bases).post(create_knowledge_base))
        .route("/api/knowledge-bases/:id", 
               put(update_knowledge_base).delete(delete_knowledge_base))
        
        // Document routes
        .route("/api/knowledge-bases/:id/documents", 
               get(list_documents).post(upload_document))
        .route("/api/documents/:id", 
               delete(delete_document))
        .route("/api/documents/:id/content", 
               get(get_document_content))
        
        // AI quiz routes
        .route("/api/knowledge-bases/:id/generate-question", 
               post(generate_question))
        .route("/api/questions/:id/answer", 
               post(submit_answer))
        
        // Review routes
        .route("/api/knowledge-bases/:id/review/random", 
               get(get_random_review_question))
        .route("/api/knowledge-bases/:id/review/questions", 
               get(get_review_questions))
        .route("/api/knowledge-bases/:id/history", 
               get(get_history))
        .route("/api/knowledge-bases/:id/progress", 
               get(get_learning_progress))
        .route("/api/knowledge-bases/:id/review-sessions", 
               get(get_review_sessions))
        .route("/api/review-sessions", 
               post(create_review_session))
        .route("/api/review-sessions/:id/score", 
               put(update_review_session_score))
        .route("/api/review/answer", 
               post(submit_review_answer))
        
        // AI config routes
        .route("/api/ai-config", 
               get(get_ai_config).post(save_ai_config))
        .route("/api/ai-config/test", 
               post(test_ai_connection))
        
        // Add CORS layer
        .layer(CorsLayer::permissive())
}
