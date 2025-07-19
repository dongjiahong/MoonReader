use moon_reader::{
    database::create_connection_pool,
    services::AppState,
    models::{KnowledgeBase, Document, DocumentType},
};
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_document_management_workflow() {
    // Create in-memory database for testing
    let pool = create_connection_pool(":memory:").await.unwrap();
    let app_state = AppState::new(pool);
    
    // Create a test knowledge base
    let kb = app_state.db.create_knowledge_base("Test KB", Some("Test description")).await.unwrap();
    
    // Create a test document file
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_content = "This is a test document for integration testing.";
    temp_file.write_all(test_content.as_bytes()).unwrap();
    
    // Create a document record
    let document = Document::new(
        kb.id.clone(),
        "test.txt".to_string(),
        DocumentType::Txt,
        temp_file.path().to_string_lossy().to_string(),
        test_content.len() as i64,
        Some(test_content.to_string()),
    );
    
    // Save document to database
    app_state.db.save_document(&document).await.unwrap();
    
    // Test: List documents by knowledge base
    let documents = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].filename, "test.txt");
    assert_eq!(documents[0].file_type.to_string(), "txt");
    
    // Test: Get document by ID
    let retrieved_doc = app_state.db.get_document_by_id(&document.id).await.unwrap();
    assert!(retrieved_doc.is_some());
    let retrieved_doc = retrieved_doc.unwrap();
    assert_eq!(retrieved_doc.filename, "test.txt");
    assert_eq!(retrieved_doc.content_text.as_ref().unwrap(), test_content);
    
    // Test: Delete document
    let deleted = app_state.db.delete_document(&document.id).await.unwrap();
    assert!(deleted);
    
    // Verify document is deleted
    let documents_after_delete = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents_after_delete.len(), 0);
    
    // Verify document by ID returns None
    let deleted_doc = app_state.db.get_document_by_id(&document.id).await.unwrap();
    assert!(deleted_doc.is_none());
}

#[tokio::test]
async fn test_document_content_preview() {
    // Create in-memory database for testing
    let pool = create_connection_pool(":memory:").await.unwrap();
    let app_state = AppState::new(pool);
    
    // Create a test knowledge base
    let kb = app_state.db.create_knowledge_base("Test KB", Some("Test description")).await.unwrap();
    
    // Create a document with long content
    let long_content = "A".repeat(500); // 500 characters
    let document = Document::new(
        kb.id.clone(),
        "long_test.txt".to_string(),
        DocumentType::Txt,
        "/tmp/long_test.txt".to_string(),
        long_content.len() as i64,
        Some(long_content.clone()),
    );
    
    // Save document to database
    app_state.db.save_document(&document).await.unwrap();
    
    // Test: Get documents with content preview
    let documents = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents.len(), 1);
    
    let doc = &documents[0];
    assert_eq!(doc.filename, "long_test.txt");
    assert_eq!(doc.content_text.as_ref().unwrap().len(), 500);
    
    // The preview logic would be in the handler, but we can test the full content retrieval
    let full_doc = app_state.db.get_document_by_id(&document.id).await.unwrap().unwrap();
    assert_eq!(full_doc.content_text.as_ref().unwrap(), &long_content);
}

#[tokio::test]
async fn test_multiple_document_types() {
    // Create in-memory database for testing
    let pool = create_connection_pool(":memory:").await.unwrap();
    let app_state = AppState::new(pool);
    
    // Create a test knowledge base
    let kb = app_state.db.create_knowledge_base("Multi-format KB", Some("Testing multiple formats")).await.unwrap();
    
    // Create documents of different types
    let txt_doc = Document::new(
        kb.id.clone(),
        "test.txt".to_string(),
        DocumentType::Txt,
        "/tmp/test.txt".to_string(),
        100,
        Some("Text content".to_string()),
    );
    
    let pdf_doc = Document::new(
        kb.id.clone(),
        "test.pdf".to_string(),
        DocumentType::Pdf,
        "/tmp/test.pdf".to_string(),
        1000,
        Some("PDF content".to_string()),
    );
    
    let epub_doc = Document::new(
        kb.id.clone(),
        "test.epub".to_string(),
        DocumentType::Epub,
        "/tmp/test.epub".to_string(),
        2000,
        Some("EPUB content".to_string()),
    );
    
    // Save all documents
    app_state.db.save_document(&txt_doc).await.unwrap();
    app_state.db.save_document(&pdf_doc).await.unwrap();
    app_state.db.save_document(&epub_doc).await.unwrap();
    
    // Test: List all documents
    let documents = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
    assert_eq!(documents.len(), 3);
    
    // Verify document types
    let mut found_types = std::collections::HashSet::new();
    for doc in documents {
        found_types.insert(doc.file_type.to_string());
    }
    
    assert!(found_types.contains("txt"));
    assert!(found_types.contains("pdf"));
    assert!(found_types.contains("epub"));
}