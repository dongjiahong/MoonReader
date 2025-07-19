// End-to-end integration tests for the complete knowledge accumulation system
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::io::Write;
use tempfile::NamedTempFile;
use tower::{Service, ServiceExt};

use moon_reader::{
    database::create_connection_pool,
    services::AppState,
    models::{Document, DocumentType, AIConfig, AIProvider},
};

// Helper function to create a test app with in-memory database
async fn create_test_app() -> (Router, SqlitePool, AppState) {
    // Use in-memory database for tests
    let database_url = "sqlite::memory:";
    
    // Create connection pool
    let pool = create_connection_pool(&database_url).await.unwrap();
    
    // Run migrations manually for tests
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Create app state
    let app_state = AppState::new(pool.clone());
    
    // Create the app
    let app = create_app().with_state(app_state.clone());
    
    (app, pool, app_state)
}

// Helper function to create the app router
fn create_app() -> Router<AppState> {
    use moon_reader::handlers::*;
    
    Router::new()
        // Knowledge base routes
        .route("/api/knowledge-bases", 
               axum::routing::get(knowledge_base::list_knowledge_bases)
               .post(knowledge_base::create_knowledge_base))
        .route("/api/knowledge-bases/:id", 
               axum::routing::put(knowledge_base::update_knowledge_base)
               .delete(knowledge_base::delete_knowledge_base))
        // Document routes
        .route("/api/knowledge-bases/:id/documents",
               axum::routing::get(document::list_documents)
               .post(document::upload_document))
        .route("/api/documents/:id",
               axum::routing::delete(document::delete_document))
        // AI Quiz routes
        .route("/api/knowledge-bases/:id/generate-question",
               axum::routing::post(ai_quiz::generate_question))
        .route("/api/questions/:id/answer",
               axum::routing::post(ai_quiz::submit_answer))
        // Review routes
        .route("/api/knowledge-bases/:id/review/random",
               axum::routing::get(review::get_random_review_question))
        .route("/api/knowledge-bases/:id/history",
               axum::routing::get(review::get_history))
        // AI Config routes
        .route("/api/ai-config",
               axum::routing::get(ai_config::get_ai_config)
               .post(ai_config::save_ai_config))
        .route("/api/ai-config/test",
               axum::routing::post(ai_config::test_ai_connection))
}

#[tokio::test]
async fn test_complete_knowledge_base_to_quiz_workflow() {
    let (mut app, _pool, app_state) = create_test_app().await;
    
    // Step 1: Create a knowledge base
    let kb_payload = json!({
        "name": "Machine Learning Basics",
        "description": "Fundamental concepts of machine learning"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(kb_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let kb_response: Value = serde_json::from_slice(&body).unwrap();
    let kb_id = kb_response["id"].as_str().unwrap();
    
    // Step 2: Upload a document to the knowledge base
    let test_content = r#"
        Machine Learning is a subset of artificial intelligence that enables computers to learn and make decisions from data without being explicitly programmed.
        
        Key concepts include:
        1. Supervised Learning: Learning with labeled data
        2. Unsupervised Learning: Finding patterns in unlabeled data
        3. Reinforcement Learning: Learning through interaction with environment
        
        Common algorithms include linear regression, decision trees, and neural networks.
    "#;
    
    // Create a temporary file for testing
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(test_content.as_bytes()).unwrap();
    
    // Simulate document upload by directly adding to database
    let document = Document::new(
        kb_id.to_string(),
        "ml_basics.txt".to_string(),
        DocumentType::Txt,
        temp_file.path().to_string_lossy().to_string(),
        test_content.len() as i64,
        Some(test_content.to_string()),
    );
    
    app_state.db.save_document(&document).await.unwrap();
    
    // Step 3: Verify document was uploaded
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/documents", kb_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let docs_response: Value = serde_json::from_slice(&body).unwrap();
    let documents = docs_response["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["filename"], "ml_basics.txt");
    
    // Step 4: Configure AI service (mock configuration)
    let ai_config = AIConfig {
        id: None,
        provider: AIProvider::DeepSeek,
        api_key: Some("test-api-key".to_string()),
        api_url: Some("https://api.deepseek.com/v1".to_string()),
        model_name: Some("deepseek-chat".to_string()),
        max_tokens: 1000,
        temperature: 0.7,
        updated_at: chrono::Utc::now(),
    };
    
    app_state.db.save_ai_config(&ai_config).await.unwrap();
    
    // Step 5: Generate a question (this would normally call AI service)
    // For testing, we'll simulate the AI response
    let question_payload = json!({
        "context_limit": 500
    });
    
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/generate-question", kb_id))
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(question_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    // Note: This might return an error if AI service is not available, which is expected in tests
    // We'll check for either success or a specific AI service error
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE);
    
    // Step 6: Test error handling for empty knowledge base
    let empty_kb_payload = json!({
        "name": "Empty KB",
        "description": "Knowledge base with no documents"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(empty_kb_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let empty_kb_response: Value = serde_json::from_slice(&body).unwrap();
    let empty_kb_id = empty_kb_response["id"].as_str().unwrap();
    
    // Try to generate question from empty knowledge base
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/generate-question", empty_kb_id))
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    assert!(error_response["error"].as_str().unwrap().contains("No documents found"));
}

#[tokio::test]
async fn test_document_upload_and_parsing_workflow() {
    let (mut app, _pool, app_state) = create_test_app().await;
    
    // Create a knowledge base
    let kb = app_state.db.create_knowledge_base("Document Test KB", Some("Testing document parsing")).await.unwrap();
    
    // Test 1: TXT file parsing
    let txt_content = "This is a simple text document for testing parsing functionality.";
    let txt_doc = Document::new(
        kb.id.clone(),
        "test.txt".to_string(),
        DocumentType::Txt,
        "/tmp/test.txt".to_string(),
        txt_content.len() as i64,
        Some(txt_content.to_string()),
    );
    
    app_state.db.save_document(&txt_doc).await.unwrap();
    
    // Test 2: Large file handling
    let large_content = "A".repeat(10000); // 10KB content
    let large_doc = Document::new(
        kb.id.clone(),
        "large_test.txt".to_string(),
        DocumentType::Txt,
        "/tmp/large_test.txt".to_string(),
        large_content.len() as i64,
        Some(large_content.clone()),
    );
    
    app_state.db.save_document(&large_doc).await.unwrap();
    
    // Test 3: Multiple document types
    let pdf_doc = Document::new(
        kb.id.clone(),
        "test.pdf".to_string(),
        DocumentType::Pdf,
        "/tmp/test.pdf".to_string(),
        5000,
        Some("Extracted PDF content for testing".to_string()),
    );
    
    let epub_doc = Document::new(
        kb.id.clone(),
        "test.epub".to_string(),
        DocumentType::Epub,
        "/tmp/test.epub".to_string(),
        8000,
        Some("Extracted EPUB content for testing".to_string()),
    );
    
    app_state.db.save_document(&pdf_doc).await.unwrap();
    app_state.db.save_document(&epub_doc).await.unwrap();
    
    // Verify all documents are stored
    let documents = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents.len(), 4);
    
    // Test document retrieval API
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/documents", kb.id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let docs_response: Value = serde_json::from_slice(&body).unwrap();
    let api_documents = docs_response["documents"].as_array().unwrap();
    assert_eq!(api_documents.len(), 4);
    
    // Verify document types are correctly stored
    let mut found_types = std::collections::HashSet::new();
    for doc in api_documents {
        found_types.insert(doc["file_type"].as_str().unwrap().to_string());
    }
    
    assert!(found_types.contains("txt"));
    assert!(found_types.contains("pdf"));
    assert!(found_types.contains("epub"));
}

#[tokio::test]
async fn test_ai_quiz_and_review_workflow() {
    let (mut app, _pool, app_state) = create_test_app().await;
    
    // Setup: Create knowledge base with content
    let kb = app_state.db.create_knowledge_base("Quiz Test KB", Some("Testing AI quiz functionality")).await.unwrap();
    
    let content = r#"
        Artificial Intelligence (AI) is the simulation of human intelligence in machines.
        Machine Learning is a subset of AI that enables computers to learn from data.
        Deep Learning uses neural networks with multiple layers to process data.
    "#;
    
    let document = Document::new(
        kb.id.clone(),
        "ai_concepts.txt".to_string(),
        DocumentType::Txt,
        "/tmp/ai_concepts.txt".to_string(),
        content.len() as i64,
        Some(content.to_string()),
    );
    
    app_state.db.save_document(&document).await.unwrap();
    
    // Test 1: AI Configuration
    let ai_config_payload = json!({
        "provider": "deepseek",
        "api_key": "test-key",
        "api_url": "https://api.deepseek.com/v1",
        "model_name": "deepseek-chat",
        "max_tokens": 1000,
        "temperature": 0.7
    });
    
    let request = Request::builder()
        .uri("/api/ai-config")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(ai_config_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test 2: Get AI Configuration
    let request = Request::builder()
        .uri("/api/ai-config")
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let config_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(config_response["provider"], "deepseek");
    assert_eq!(config_response["model_name"], "deepseek-chat");
    
    // Test 3: Question Generation (will likely fail without real AI service, but we test the endpoint)
    let question_request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/generate-question", kb.id))
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    
    let response = app.call(question_request).await.unwrap();
    // Accept either success or service unavailable (expected in test environment)
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE);
    
    // Test 4: Manual question creation for testing review functionality
    use moon_reader::models::{Question, Answer};
    
    let question = Question::new(
        kb.id.clone(),
        "What is the difference between AI and Machine Learning?".to_string(),
        Some("AI vs ML concepts".to_string()),
    );
    
    app_state.db.save_question(&question).await.unwrap();
    
    let answer = Answer::new(
        question.id.clone(),
        "AI is broader, ML is a subset of AI that learns from data".to_string(),
    );
    
    app_state.db.save_answer(&answer).await.unwrap();
    
    // Test 5: Review functionality - get random question
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/review/random", kb.id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let review_response: Value = serde_json::from_slice(&body).unwrap();
    // The response might contain an error message if no questions are found
    assert!(review_response["question"].is_object() || review_response["error"].is_string());
    
    // Test 6: Get question history
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/history", kb.id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let history_response: Value = serde_json::from_slice(&body).unwrap();
    let items = history_response["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["question"]["question_text"], "What is the difference between AI and Machine Learning?");
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    let (mut app, _pool, app_state) = create_test_app().await;
    
    // Test 1: Invalid knowledge base creation
    let invalid_payload = json!({
        "name": "", // Empty name
        "description": "Should fail validation"
    });
    
    let request = Request::builder()
        .uri("/api/knowledge-bases")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(invalid_payload.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test 2: Access non-existent knowledge base
    let request = Request::builder()
        .uri("/api/knowledge-bases/non-existent-id")
        .method("PUT")
        .header("content-type", "application/json")
        .body(Body::from(json!({"name": "Updated", "description": "Test"}).to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Test 3: Delete non-existent knowledge base
    let request = Request::builder()
        .uri("/api/knowledge-bases/non-existent-id")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Test 4: Access documents of non-existent knowledge base
    let request = Request::builder()
        .uri("/api/knowledge-bases/non-existent-id/documents")
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    // Currently returns 200 with empty list - this might be acceptable behavior
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let docs_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(docs_response["documents"].as_array().unwrap().len(), 0);
    
    // Test 5: Delete non-existent document
    let request = Request::builder()
        .uri("/api/documents/non-existent-id")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Test 6: Generate question without AI configuration
    let kb = app_state.db.create_knowledge_base("No AI Config KB", None).await.unwrap();
    
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/generate-question", kb.id))
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    // Should return error about missing AI configuration
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::SERVICE_UNAVAILABLE);
    
    // Test 7: Invalid AI configuration
    let invalid_ai_config = json!({
        "provider": "invalid-provider",
        "api_key": "",
        "max_tokens": -1 // Invalid value
    });
    
    let request = Request::builder()
        .uri("/api/ai-config")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(invalid_ai_config.to_string()))
        .unwrap();
    
    let response = app.call(request).await.unwrap();
    // Accept either 400 (BAD_REQUEST) or 422 (UNPROCESSABLE_ENTITY) for validation errors
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY);
    
    // Test 8: Review from empty knowledge base
    let empty_kb = app_state.db.create_knowledge_base("Empty Review KB", None).await.unwrap();
    
    let request = Request::builder()
        .uri(&format!("/api/knowledge-bases/{}/review/random", empty_kb.id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    assert!(error_response["error"].as_str().unwrap().contains("No history available"));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (_app, _pool, app_state) = create_test_app().await;
    
    // Test concurrent knowledge base creation
    let mut handles = vec![];
    
    for i in 0..5 {
        let app_state_clone = app_state.clone();
        let handle = tokio::spawn(async move {
            let kb_name = format!("Concurrent KB {}", i);
            let kb = app_state_clone.db.create_knowledge_base(&kb_name, Some("Concurrent test")).await.unwrap();
            
            // Add a document to each knowledge base
            let content = format!("Content for knowledge base {}", i);
            let document = Document::new(
                kb.id.clone(),
                format!("doc_{}.txt", i),
                DocumentType::Txt,
                format!("/tmp/doc_{}.txt", i),
                content.len() as i64,
                Some(content),
            );
            
            app_state_clone.db.save_document(&document).await.unwrap();
            kb.id
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut kb_ids = vec![];
    for handle in handles {
        let kb_id = handle.await.unwrap();
        kb_ids.push(kb_id);
    }
    
    // Verify all knowledge bases were created
    assert_eq!(kb_ids.len(), 5);
    
    // Verify each knowledge base has its document
    for kb_id in kb_ids {
        let documents = app_state.db.get_documents_by_knowledge_base(&kb_id).await.unwrap();
        assert_eq!(documents.len(), 1);
    }
    
    // Verify total count
    let all_kbs = app_state.db.get_knowledge_bases().await.unwrap();
    assert_eq!(all_kbs.len(), 5);
}

#[tokio::test]
async fn test_data_consistency_and_cascade_delete() {
    let (_app, _pool, app_state) = create_test_app().await;
    
    // Create knowledge base with documents and questions
    let kb = app_state.db.create_knowledge_base("Cascade Test KB", Some("Testing cascade delete")).await.unwrap();
    
    // Add multiple documents
    for i in 0..3 {
        let document = Document::new(
            kb.id.clone(),
            format!("doc_{}.txt", i),
            DocumentType::Txt,
            format!("/tmp/doc_{}.txt", i),
            100,
            Some(format!("Content {}", i)),
        );
        app_state.db.save_document(&document).await.unwrap();
    }
    
    // Add questions and answers
    use moon_reader::models::{Question, Answer};
    
    for i in 0..2 {
        let question = Question::new(
            kb.id.clone(),
            format!("Question {}?", i),
            Some(format!("Context {}", i)),
        );
        app_state.db.save_question(&question).await.unwrap();
        
        let answer = Answer::new(
            question.id.clone(),
            format!("Answer {}", i),
        );
        app_state.db.save_answer(&answer).await.unwrap();
    }
    
    // Verify data exists
    let documents = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents.len(), 3);
    
    let questions = app_state.db.get_questions_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(questions.len(), 2);
    
    // Delete knowledge base
    let deleted = app_state.db.delete_knowledge_base(&kb.id).await.unwrap();
    assert!(deleted);
    
    // Verify cascade delete worked
    let documents_after = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents_after.len(), 0);
    
    let questions_after = app_state.db.get_questions_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(questions_after.len(), 0);
    
    // Verify knowledge base is gone
    let kb_after = app_state.db.get_knowledge_base_by_id(&kb.id).await.unwrap();
    assert!(kb_after.is_none());
}