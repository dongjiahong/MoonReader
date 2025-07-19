// Performance optimizations for the knowledge accumulation system
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use crate::models::{KnowledgeBase, Document};

/// Cache entry with expiration time
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub expires_at: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: i64) -> Self {
        Self {
            data,
            expires_at: Utc::now() + Duration::seconds(ttl_seconds),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// In-memory cache for frequently accessed data
#[derive(Debug)]
pub struct MemoryCache {
    knowledge_bases: Arc<RwLock<HashMap<String, CacheEntry<Vec<KnowledgeBase>>>>>,
    documents: Arc<RwLock<HashMap<String, CacheEntry<Vec<Document>>>>>,
    document_content: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            knowledge_bases: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            document_content: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Cache knowledge bases list
    pub async fn cache_knowledge_bases(&self, key: &str, data: Vec<KnowledgeBase>, ttl_seconds: i64) {
        let mut cache = self.knowledge_bases.write().await;
        cache.insert(key.to_string(), CacheEntry::new(data, ttl_seconds));
    }
    
    /// Get cached knowledge bases
    pub async fn get_knowledge_bases(&self, key: &str) -> Option<Vec<KnowledgeBase>> {
        let mut cache = self.knowledge_bases.write().await;
        
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        None
    }
    
    /// Cache documents for a knowledge base
    pub async fn cache_documents(&self, kb_id: &str, data: Vec<Document>, ttl_seconds: i64) {
        let mut cache = self.documents.write().await;
        cache.insert(kb_id.to_string(), CacheEntry::new(data, ttl_seconds));
    }
    
    /// Get cached documents for a knowledge base
    pub async fn get_documents(&self, kb_id: &str) -> Option<Vec<Document>> {
        let mut cache = self.documents.write().await;
        
        if let Some(entry) = cache.get(kb_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                cache.remove(kb_id);
            }
        }
        None
    }
    
    /// Cache document content
    pub async fn cache_document_content(&self, doc_id: &str, content: String, ttl_seconds: i64) {
        let mut cache = self.document_content.write().await;
        cache.insert(doc_id.to_string(), CacheEntry::new(content, ttl_seconds));
    }
    
    /// Get cached document content
    pub async fn get_document_content(&self, doc_id: &str) -> Option<String> {
        let mut cache = self.document_content.write().await;
        
        if let Some(entry) = cache.get(doc_id) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                cache.remove(doc_id);
            }
        }
        None
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut kb_cache = self.knowledge_bases.write().await;
        let mut doc_cache = self.documents.write().await;
        let mut content_cache = self.document_content.write().await;
        
        kb_cache.clear();
        doc_cache.clear();
        content_cache.clear();
    }
    
    /// Clear expired entries from all caches
    pub async fn cleanup_expired(&self) {
        // Cleanup knowledge bases cache
        {
            let mut cache = self.knowledge_bases.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
        
        // Cleanup documents cache
        {
            let mut cache = self.documents.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
        
        // Cleanup document content cache
        {
            let mut cache = self.document_content.write().await;
            cache.retain(|_, entry| !entry.is_expired());
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Database query optimization utilities
pub struct QueryOptimizer;

impl QueryOptimizer {
    /// Generate optimized SQL for knowledge base listing with pagination
    pub fn optimized_knowledge_bases_query(limit: Option<i32>, offset: Option<i32>) -> String {
        let base_query = "SELECT id, name, description, created_at, updated_at FROM knowledge_bases ORDER BY created_at DESC";
        
        match (limit, offset) {
            (Some(l), Some(o)) => format!("{} LIMIT {} OFFSET {}", base_query, l, o),
            (Some(l), None) => format!("{} LIMIT {}", base_query, l),
            (None, Some(o)) => format!("{} OFFSET {}", base_query, o),
            (None, None) => base_query.to_string(),
        }
    }
    
    /// Generate optimized SQL for document listing with filtering
    pub fn optimized_documents_query(
        kb_id: &str,
        file_type: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>
    ) -> (String, Vec<String>) {
        let mut query = "SELECT id, knowledge_base_id, filename, file_type, file_path, file_size, upload_date FROM documents WHERE knowledge_base_id = ?".to_string();
        let mut params = vec![kb_id.to_string()];
        
        if let Some(ft) = file_type {
            query.push_str(" AND file_type = ?");
            params.push(ft.to_string());
        }
        
        query.push_str(" ORDER BY upload_date DESC");
        
        match (limit, offset) {
            (Some(l), Some(o)) => {
                query.push_str(&format!(" LIMIT {} OFFSET {}", l, o));
            },
            (Some(l), None) => {
                query.push_str(&format!(" LIMIT {}", l));
            },
            (None, Some(o)) => {
                query.push_str(&format!(" OFFSET {}", o));
            },
            (None, None) => {},
        }
        
        (query, params)
    }
    
    /// Generate optimized SQL for content search
    pub fn optimized_content_search_query(
        kb_id: &str,
        search_term: &str,
        limit: Option<i32>
    ) -> (String, Vec<String>) {
        let query = r#"
            SELECT d.id, d.filename, d.file_type, d.upload_date,
                   SUBSTR(d.content_text, 1, 200) as content_preview
            FROM documents d
            WHERE d.knowledge_base_id = ? 
            AND d.content_text LIKE ?
            ORDER BY d.upload_date DESC
        "#;
        
        let mut final_query = query.to_string();
        let search_pattern = format!("%{}%", search_term);
        let params = vec![kb_id.to_string(), search_pattern];
        
        if let Some(l) = limit {
            final_query.push_str(&format!(" LIMIT {}", l));
        }
        
        (final_query, params)
    }
}

/// File processing optimization utilities
pub struct FileProcessor;

impl FileProcessor {
    /// Optimize large file processing by chunking
    pub async fn process_large_file_chunked<F, Fut>(
        file_path: &str,
        chunk_size: usize,
        processor: F,
    ) -> Result<Vec<String>, std::io::Error>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<String, std::io::Error>>,
    {
        use tokio::fs::File;
        use tokio::io::{AsyncReadExt, BufReader};
        
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut buffer = vec![0; chunk_size];
        
        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
            let processed = processor(chunk).await?;
            results.push(processed);
        }
        
        Ok(results)
    }
    
    /// Optimize text content for storage (remove excessive whitespace, etc.)
    pub fn optimize_text_content(content: &str) -> String {
        // Remove excessive whitespace while preserving structure
        let lines: Vec<&str> = content.lines().collect();
        let mut optimized_lines = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                optimized_lines.push(trimmed);
            }
        }
        
        optimized_lines.join("\n")
    }
    
    /// Generate content preview efficiently
    pub fn generate_content_preview(content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            return content.to_string();
        }
        
        // Find a good breaking point (end of sentence or word)
        let truncated = &content[..max_length];
        
        // Try to break at sentence end
        if let Some(pos) = truncated.rfind('.') {
            if pos > max_length / 2 {
                return format!("{}.", &truncated[..pos]);
            }
        }
        
        // Try to break at word boundary
        if let Some(pos) = truncated.rfind(' ') {
            if pos > max_length / 2 {
                return format!("{}...", &truncated[..pos]);
            }
        }
        
        // Fallback to hard truncation
        format!("{}...", truncated)
    }
}

/// Background task for cache cleanup and maintenance
pub struct CacheMaintenanceTask {
    cache: Arc<MemoryCache>,
    cleanup_interval: Duration,
}

impl CacheMaintenanceTask {
    pub fn new(cache: Arc<MemoryCache>, cleanup_interval_minutes: i64) -> Self {
        Self {
            cache,
            cleanup_interval: Duration::minutes(cleanup_interval_minutes),
        }
    }
    
    /// Start the background maintenance task
    pub async fn start(&self) {
        let cache = self.cache.clone();
        let interval = self.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                std::time::Duration::from_secs(interval.num_seconds() as u64)
            );
            
            loop {
                interval_timer.tick().await;
                cache.cleanup_expired().await;
                
                // Log cache statistics (in a real implementation, you'd use proper logging)
                println!("Cache maintenance completed at {}", Utc::now());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};
    
    #[tokio::test]
    async fn test_memory_cache_basic_operations() {
        let cache = MemoryCache::new();
        
        // Test knowledge bases caching
        let kbs = vec![]; // Empty for test
        cache.cache_knowledge_bases("all", kbs.clone(), 60).await;
        
        let cached_kbs = cache.get_knowledge_bases("all").await;
        assert!(cached_kbs.is_some());
        assert_eq!(cached_kbs.unwrap().len(), 0);
        
        // Test non-existent key
        let missing = cache.get_knowledge_bases("missing").await;
        assert!(missing.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MemoryCache::new();
        
        // Cache with very short TTL
        cache.cache_document_content("test-doc", "test content".to_string(), 1).await;
        
        // Should be available immediately
        let content = cache.get_document_content("test-doc").await;
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "test content");
        
        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;
        
        // Should be expired and removed
        let expired_content = cache.get_document_content("test-doc").await;
        assert!(expired_content.is_none());
    }
    
    #[test]
    fn test_query_optimizer() {
        // Test knowledge bases query optimization
        let query = QueryOptimizer::optimized_knowledge_bases_query(Some(10), Some(20));
        assert!(query.contains("LIMIT 10"));
        assert!(query.contains("OFFSET 20"));
        
        // Test documents query optimization
        let (query, params) = QueryOptimizer::optimized_documents_query("kb-1", Some("pdf"), Some(5), None);
        assert!(query.contains("file_type = ?"));
        assert!(query.contains("LIMIT 5"));
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "kb-1");
        assert_eq!(params[1], "pdf");
    }
    
    #[test]
    fn test_file_processor_optimizations() {
        // Test content optimization
        let messy_content = "  Line 1  \n\n\n  Line 2  \n   \n  Line 3  ";
        let optimized = FileProcessor::optimize_text_content(messy_content);
        assert_eq!(optimized, "Line 1\nLine 2\nLine 3");
        
        // Test content preview generation
        let long_content = "This is a very long piece of content that should be truncated. It contains multiple sentences. This should be cut off somewhere.";
        let preview = FileProcessor::generate_content_preview(long_content, 50);
        assert!(preview.len() <= 53); // 50 + "..." = 53
        assert!(preview.ends_with("...") || preview.ends_with("."));
    }
}