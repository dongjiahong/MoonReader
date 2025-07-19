// Integration tests for the knowledge base API
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use tower::{Service, ServiceExt};

use moon_reader::{
    database::create_connection_pool,
    services::AppState,
};

// Helper function to create a test app with in-memory database
async fn create_test_app() -> (Router, SqlitePool) {
    // Use in-memory database for tests
    let database_url = "sqlite::memory:";
    
    // Create connection pool
    let pool = create_connection_pool(&database_url).await.unwrap();
    
    // Run migrations manually for tests
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Create app state
    let app_state = AppState::new(pool.clone());
    
    // Create the app
    let app = create_app().with_state(app_state);
    
    (app, pool)
}

// Helper function to create the app router (copied from main.rs)
fn create_app() -> Router<AppState> {
    use moon_reader::handlers::*;
    
    Router::new()
        // Knowledge base routes
        .route("/api/knowledge-bases", 
               axum::routing::get(list_knowledge_bases).post(create_knowledge_base))
        .route("/api/knowledge-bases/:id", 
               axum::routing::put(update_knowledge_base).delete(delete_knowledge_base))
}

#[tokio::test]
async fn test_list_empty_knowledge_bases() {
    let (app, _pool) = create_test_app().await;
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["knowledge_bases"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_knowledge_base() {
    let (app, _pool) = create_test_app().await;
    
    let payload = json!({
        "name": "Test Knowledge Base",
        "description": "A test knowledge base"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["name"], "Test Knowledge Base");
    assert_eq!(json["description"], "A test knowledge base");
    assert!(json["id"].is_string());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

#[tokio::test]
async fn test_create_knowledge_base_validation_error() {
    let (app, _pool) = create_test_app().await;
    
    let payload = json!({
        "name": "", // Empty name should fail validation
        "description": "A test knowledge base"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["error"], "Validation failed");
    assert!(json["details"].is_string());
}

#[tokio::test]
async fn test_create_and_list_knowledge_bases() {
    let (mut app, _pool) = create_test_app().await;
    
    // Create a knowledge base
    let payload = json!({
        "name": "Test Knowledge Base",
        "description": "A test knowledge base"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // List knowledge bases
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    let knowledge_bases = json["knowledge_bases"].as_array().unwrap();
    assert_eq!(knowledge_bases.len(), 1);
    assert_eq!(knowledge_bases[0]["name"], "Test Knowledge Base");
}

#[tokio::test]
async fn test_update_knowledge_base() {
    let (mut app, _pool) = create_test_app().await;
    
    // Create a knowledge base first
    let create_payload = json!({
        "name": "Original Name",
        "description": "Original description"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_kb: Value = serde_json::from_slice(&body).unwrap();
    let kb_id = created_kb["id"].as_str().unwrap();
    
    // Update the knowledge base
    let update_payload = json!({
        "name": "Updated Name",
        "description": "Updated description"
    });
    
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}", kb_id))
        .method("PUT")
        .header("content-type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_kb: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(updated_kb["name"], "Updated Name");
    assert_eq!(updated_kb["description"], "Updated description");
    assert_eq!(updated_kb["id"], kb_id);
}

#[tokio::test]
async fn test_update_nonexistent_knowledge_base() {
    let (app, _pool) = create_test_app().await;
    
    let update_payload = json!({
        "name": "Updated Name",
        "description": "Updated description"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases/nonexistent-id")
        .method("PUT")
        .header("content-type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["error"], "Resource not found");
}

#[tokio::test]
async fn test_delete_knowledge_base() {
    let (mut app, _pool) = create_test_app().await;
    
    // Create a knowledge base first
    let create_payload = json!({
        "name": "To Be Deleted",
        "description": "This will be deleted"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_kb: Value = serde_json::from_slice(&body).unwrap();
    let kb_id = created_kb["id"].as_str().unwrap();
    
    // Delete the knowledge base
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}", kb_id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["message"], "Knowledge base deleted successfully");
}

#[tokio::test]
async fn test_delete_nonexistent_knowledge_base() {
    let (app, _pool) = create_test_app().await;
    
    let request = Request::builder()
        .uri("/api/knowledge-bases/nonexistent-id")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["error"], "Resource not found");
}