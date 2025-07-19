use axum::{
    extract::{Path, State, Multipart},
    response::Json,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::services::AppState;
use crate::models::{Document, DocumentType};
use crate::parsers::{DocumentParserFactory, ParseError};
use crate::error::AppError;

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB
const UPLOAD_DIR: &str = "uploads";

pub async fn list_documents(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let documents = state.db.get_documents_by_knowledge_base(&kb_id).await
        .map_err(AppError::Database)?;
    
    let documents_json: Vec<Value> = documents.into_iter().map(|doc| {
        json!({
            "id": doc.id,
            "knowledge_base_id": doc.knowledge_base_id,
            "filename": doc.filename,
            "file_type": doc.file_type.to_string(),
            "file_size": doc.file_size,
            "upload_date": doc.upload_date,
            "content_preview": doc.content_text.as_ref().map(|text| {
                if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                }
            })
        })
    }).collect();
    
    Ok(Json(json!({"documents": documents_json})))
}

pub async fn upload_document(
    Path(kb_id): Path<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // Verify knowledge base exists
    let _kb = state.db.get_knowledge_base_by_id(&kb_id).await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Validation("Knowledge base not found".to_string()))?;

    // Create upload directory if it doesn't exist
    let upload_path = PathBuf::from(UPLOAD_DIR);
    if !upload_path.exists() {
        fs::create_dir_all(&upload_path).await
            .map_err(|e| AppError::FileUpload(format!("Failed to create upload directory: {}", e)))?;
    }

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::FileUpload(format!("Failed to read multipart field: {}", e)))? {
        
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let filename = field.file_name()
            .ok_or_else(|| AppError::FileUpload("No filename provided".to_string()))?
            .to_string();

        // Validate file extension
        let extension = filename.split('.').last()
            .ok_or_else(|| AppError::FileUpload("No file extension found".to_string()))?
            .to_lowercase();

        let parser = DocumentParserFactory::get_parser(&extension)
            .ok_or_else(|| AppError::FileUpload(format!("Unsupported file type: {}", extension)))?;

        // Read file data
        let data = field.bytes().await
            .map_err(|e| AppError::FileUpload(format!("Failed to read file data: {}", e)))?;

        // Validate file size
        if data.len() > MAX_FILE_SIZE {
            return Err(AppError::FileUpload("File size exceeds maximum limit (100MB)".to_string()));
        }

        // Generate unique file path
        let file_id = uuid::Uuid::new_v4().to_string();
        let file_path = upload_path.join(format!("{}_{}", file_id, filename));

        // Save file to disk
        let mut file = fs::File::create(&file_path).await
            .map_err(|e| AppError::FileUpload(format!("Failed to create file: {}", e)))?;
        
        file.write_all(&data).await
            .map_err(|e| AppError::FileUpload(format!("Failed to write file: {}", e)))?;

        // Parse document content
        let content_text = match parser.parse(&file_path).await {
            Ok(content) => Some(content),
            Err(ParseError::Pdf(e)) => {
                // Clean up file on parse error
                let _ = fs::remove_file(&file_path).await;
                return Err(AppError::DocumentParse(format!("PDF parsing failed: {}", e)));
            },
            Err(ParseError::Epub(e)) => {
                // Clean up file on parse error
                let _ = fs::remove_file(&file_path).await;
                return Err(AppError::DocumentParse(format!("EPUB parsing failed: {}", e)));
            },
            Err(ParseError::Io(e)) => {
                // Clean up file on parse error
                let _ = fs::remove_file(&file_path).await;
                return Err(AppError::DocumentParse(format!("IO error during parsing: {}", e)));
            },
            Err(ParseError::UnsupportedFormat) => {
                // Clean up file on parse error
                let _ = fs::remove_file(&file_path).await;
                return Err(AppError::DocumentParse("Unsupported file format".to_string()));
            },
        };

        // Determine document type
        let doc_type = match extension.as_str() {
            "pdf" => DocumentType::Pdf,
            "epub" => DocumentType::Epub,
            "txt" => DocumentType::Txt,
            _ => return Err(AppError::FileUpload("Unsupported file type".to_string())),
        };

        // Create document record
        let document = Document::new(
            kb_id.clone(),
            filename,
            doc_type,
            file_path.to_string_lossy().to_string(),
            data.len() as i64,
            content_text,
        );

        // Save to database
        state.db.save_document(&document).await
            .map_err(AppError::Database)?;

        return Ok(Json(json!({
            "message": "Document uploaded successfully",
            "document": {
                "id": document.id,
                "filename": document.filename,
                "file_type": document.file_type.to_string(),
                "file_size": document.file_size,
                "upload_date": document.upload_date
            }
        })));
    }

    Err(AppError::FileUpload("No file found in request".to_string()))
}

pub async fn delete_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // Get document info first to delete the file
    let document = state.db.get_document_by_id(&id).await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // Delete file from disk
    let file_path = PathBuf::from(&document.file_path);
    if file_path.exists() {
        fs::remove_file(&file_path).await
            .map_err(|e| AppError::FileUpload(format!("Failed to delete file: {}", e)))?;
    }

    // Delete from database
    let deleted = state.db.delete_document(&id).await
        .map_err(AppError::Database)?;

    if deleted {
        Ok(Json(json!({"message": "Document deleted successfully"})))
    } else {
        Err(AppError::NotFound("Document not found".to_string()))
    }
}

pub async fn get_document_content(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // This is a helper function to get document content for preview
    let document = state.db.get_document_by_id(&id).await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    Ok(Json(json!({
        "id": document.id,
        "filename": document.filename,
        "content": document.content_text
    })))
}