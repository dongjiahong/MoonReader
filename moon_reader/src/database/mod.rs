// Database module for data access layer
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;
use chrono::Utc;
use crate::models::{KnowledgeBase, Document, Question, Answer, ReviewSession, AIConfig, DocumentType, AIProvider, LearningProgress};

#[cfg(test)]
mod tests;

pub async fn create_connection_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    
    let pool = SqlitePool::connect_with(options).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(pool)
}

// Database manager for handling database operations
#[derive(Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    // Knowledge Base CRUD operations
    pub async fn create_knowledge_base(&self, name: &str, description: Option<&str>) -> Result<KnowledgeBase, sqlx::Error> {
        let kb = KnowledgeBase::new(name.to_string(), description.map(|s| s.to_string()));
        
        sqlx::query(
            "INSERT INTO knowledge_bases (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&kb.id)
        .bind(&kb.name)
        .bind(&kb.description)
        .bind(&kb.created_at)
        .bind(&kb.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(kb)
    }
    
    pub async fn get_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>, sqlx::Error> {
        let rows = sqlx::query_as::<_, KnowledgeBase>(
            "SELECT id, name, description, created_at, updated_at FROM knowledge_bases ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }
    
    pub async fn get_knowledge_base_by_id(&self, id: &str) -> Result<Option<KnowledgeBase>, sqlx::Error> {
        let row = sqlx::query_as::<_, KnowledgeBase>(
            "SELECT id, name, description, created_at, updated_at FROM knowledge_bases WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }
    
    pub async fn update_knowledge_base(&self, id: &str, name: &str, description: Option<&str>) -> Result<bool, sqlx::Error> {
        let updated_at = Utc::now();
        let result = sqlx::query(
            "UPDATE knowledge_bases SET name = ?, description = ?, updated_at = ? WHERE id = ?"
        )
        .bind(name)
        .bind(description)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn delete_knowledge_base(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM knowledge_bases WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    // Document CRUD operations
    pub async fn save_document(&self, document: &Document) -> Result<(), sqlx::Error> {
        let file_type_str = document.file_type.to_string();
        
        sqlx::query(
            "INSERT INTO documents (id, knowledge_base_id, filename, file_type, file_path, file_size, content_text, upload_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&document.id)
        .bind(&document.knowledge_base_id)
        .bind(&document.filename)
        .bind(&file_type_str)
        .bind(&document.file_path)
        .bind(&document.file_size)
        .bind(&document.content_text)
        .bind(&document.upload_date)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_documents_by_knowledge_base(&self, knowledge_base_id: &str) -> Result<Vec<Document>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, knowledge_base_id, filename, file_type, file_path, file_size, content_text, upload_date FROM documents WHERE knowledge_base_id = ? ORDER BY upload_date DESC"
        )
        .bind(knowledge_base_id)
        .fetch_all(&self.pool)
        .await?;
        
        let documents = rows.into_iter().map(|row| {
            let file_type = match row.get::<String, _>("file_type").as_str() {
                "pdf" => DocumentType::Pdf,
                "epub" => DocumentType::Epub,
                "txt" => DocumentType::Txt,
                _ => DocumentType::Txt, // Default fallback
            };
            
            Document {
                id: row.get("id"),
                knowledge_base_id: row.get("knowledge_base_id"),
                filename: row.get("filename"),
                file_type,
                file_path: row.get("file_path"),
                file_size: row.get("file_size"),
                content_text: row.get("content_text"),
                upload_date: row.get("upload_date"),
            }
        }).collect();
        
        Ok(documents)
    }
    
    pub async fn get_document_by_id(&self, id: &str) -> Result<Option<Document>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, knowledge_base_id, filename, file_type, file_path, file_size, content_text, upload_date FROM documents WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let file_type = match row.get::<String, _>("file_type").as_str() {
                "pdf" => DocumentType::Pdf,
                "epub" => DocumentType::Epub,
                "txt" => DocumentType::Txt,
                _ => DocumentType::Txt, // Default fallback
            };
            
            Ok(Some(Document {
                id: row.get("id"),
                knowledge_base_id: row.get("knowledge_base_id"),
                filename: row.get("filename"),
                file_type,
                file_path: row.get("file_path"),
                file_size: row.get("file_size"),
                content_text: row.get("content_text"),
                upload_date: row.get("upload_date"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn delete_document(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM documents WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    // Question and Answer CRUD operations
    pub async fn save_question(&self, question: &Question) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO questions (id, knowledge_base_id, question_text, context_snippet, generated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&question.id)
        .bind(&question.knowledge_base_id)
        .bind(&question.question_text)
        .bind(&question.context_snippet)
        .bind(&question.generated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn save_answer(&self, answer: &Answer) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO answers (id, question_id, user_answer, ai_score, ai_feedback, ai_suggestions, answered_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&answer.id)
        .bind(&answer.question_id)
        .bind(&answer.user_answer)
        .bind(&answer.ai_score)
        .bind(&answer.ai_feedback)
        .bind(&answer.ai_suggestions)
        .bind(&answer.answered_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_questions_by_knowledge_base(&self, knowledge_base_id: &str) -> Result<Vec<Question>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Question>(
            "SELECT id, knowledge_base_id, question_text, context_snippet, generated_at FROM questions WHERE knowledge_base_id = ? ORDER BY generated_at DESC"
        )
        .bind(knowledge_base_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }
    
    pub async fn get_question_by_id(&self, id: &str) -> Result<Option<Question>, sqlx::Error> {
        let row = sqlx::query_as::<_, Question>(
            "SELECT id, knowledge_base_id, question_text, context_snippet, generated_at FROM questions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }
    
    pub async fn get_answers_by_question(&self, question_id: &str) -> Result<Vec<Answer>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Answer>(
            "SELECT id, question_id, user_answer, ai_score, ai_feedback, ai_suggestions, answered_at FROM answers WHERE question_id = ? ORDER BY answered_at DESC"
        )
        .bind(question_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }
    
    // Review Session CRUD operations
    pub async fn save_review_session(&self, session: &ReviewSession) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO review_sessions (id, knowledge_base_id, questions_count, average_score, session_date) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&session.id)
        .bind(&session.knowledge_base_id)
        .bind(&session.questions_count)
        .bind(&session.average_score)
        .bind(&session.session_date)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_review_sessions_by_knowledge_base(&self, knowledge_base_id: &str) -> Result<Vec<ReviewSession>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReviewSession>(
            "SELECT id, knowledge_base_id, questions_count, average_score, session_date FROM review_sessions WHERE knowledge_base_id = ? ORDER BY session_date DESC"
        )
        .bind(knowledge_base_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }
    
    // History management operations
    pub async fn get_question_answer_history(&self, knowledge_base_id: &str, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<(Question, Answer)>, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let rows = sqlx::query(
            "SELECT q.id as question_id, q.knowledge_base_id, q.question_text, q.context_snippet, q.generated_at,
                    a.id as answer_id, a.user_answer, a.ai_score, a.ai_feedback, a.ai_suggestions, a.answered_at
             FROM questions q 
             INNER JOIN answers a ON q.id = a.question_id 
             WHERE q.knowledge_base_id = ? 
             ORDER BY a.answered_at DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(knowledge_base_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let history = rows.into_iter().map(|row| {
            let question = Question {
                id: row.get("question_id"),
                knowledge_base_id: row.get("knowledge_base_id"),
                question_text: row.get("question_text"),
                context_snippet: row.get("context_snippet"),
                generated_at: row.get("generated_at"),
            };
            
            let answer = Answer {
                id: row.get("answer_id"),
                question_id: row.get("question_id"),
                user_answer: row.get("user_answer"),
                ai_score: row.get("ai_score"),
                ai_feedback: row.get("ai_feedback"),
                ai_suggestions: row.get("ai_suggestions"),
                answered_at: row.get("answered_at"),
            };
            
            (question, answer)
        }).collect();
        
        Ok(history)
    }
    
    pub async fn get_filtered_history(&self, knowledge_base_id: &str, min_score: Option<i32>, max_score: Option<i32>, start_date: Option<chrono::DateTime<Utc>>, end_date: Option<chrono::DateTime<Utc>>) -> Result<Vec<(Question, Answer)>, sqlx::Error> {
        // Use a simpler approach with fixed parameters and NULL checks
        let rows = sqlx::query(
            "SELECT q.id as question_id, q.knowledge_base_id, q.question_text, q.context_snippet, q.generated_at,
                    a.id as answer_id, a.user_answer, a.ai_score, a.ai_feedback, a.ai_suggestions, a.answered_at
             FROM questions q 
             INNER JOIN answers a ON q.id = a.question_id 
             WHERE q.knowledge_base_id = ? 
             AND (? IS NULL OR a.ai_score >= ?)
             AND (? IS NULL OR a.ai_score <= ?)
             AND (? IS NULL OR a.answered_at >= ?)
             AND (? IS NULL OR a.answered_at <= ?)
             ORDER BY a.answered_at DESC"
        )
        .bind(knowledge_base_id)
        .bind(min_score)
        .bind(min_score)
        .bind(max_score)
        .bind(max_score)
        .bind(start_date)
        .bind(start_date)
        .bind(end_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;
        
        let history = rows.into_iter().map(|row| {
            let question = Question {
                id: row.get("question_id"),
                knowledge_base_id: row.get("knowledge_base_id"),
                question_text: row.get("question_text"),
                context_snippet: row.get("context_snippet"),
                generated_at: row.get("generated_at"),
            };
            
            let answer = Answer {
                id: row.get("answer_id"),
                question_id: row.get("question_id"),
                user_answer: row.get("user_answer"),
                ai_score: row.get("ai_score"),
                ai_feedback: row.get("ai_feedback"),
                ai_suggestions: row.get("ai_suggestions"),
                answered_at: row.get("answered_at"),
            };
            
            (question, answer)
        }).collect();
        
        Ok(history)
    }
    
    pub async fn update_review_session_score(&self, session_id: &str, average_score: f64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE review_sessions SET average_score = ? WHERE id = ?"
        )
        .bind(average_score)
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn get_review_session_by_id(&self, id: &str) -> Result<Option<ReviewSession>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReviewSession>(
            "SELECT id, knowledge_base_id, questions_count, average_score, session_date FROM review_sessions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }
    
    // Get random questions from history for review
    pub async fn get_random_review_questions(&self, knowledge_base_id: &str, count: i32) -> Result<Vec<(Question, Answer)>, sqlx::Error> {
        let history = sqlx::query(
            "SELECT q.id as question_id, q.knowledge_base_id, q.question_text, q.context_snippet, q.generated_at,
                    a.id as answer_id, a.user_answer, a.ai_score, a.ai_feedback, a.ai_suggestions, a.answered_at
             FROM questions q 
             INNER JOIN answers a ON q.id = a.question_id 
             WHERE q.knowledge_base_id = ? 
             ORDER BY RANDOM() 
             LIMIT ?"
        )
        .bind(knowledge_base_id)
        .bind(count)
        .fetch_all(&self.pool)
        .await?;
        
        let questions = history.into_iter().map(|row| {
            let question = Question {
                id: row.get("question_id"),
                knowledge_base_id: row.get("knowledge_base_id"),
                question_text: row.get("question_text"),
                context_snippet: row.get("context_snippet"),
                generated_at: row.get("generated_at"),
            };
            
            let answer = Answer {
                id: row.get("answer_id"),
                question_id: row.get("question_id"),
                user_answer: row.get("user_answer"),
                ai_score: row.get("ai_score"),
                ai_feedback: row.get("ai_feedback"),
                ai_suggestions: row.get("ai_suggestions"),
                answered_at: row.get("answered_at"),
            };
            
            (question, answer)
        }).collect();
        
        Ok(questions)
    }
    
    // Get learning progress statistics
    pub async fn get_learning_progress(&self, knowledge_base_id: &str) -> Result<LearningProgress, sqlx::Error> {
        // Get all question-answer pairs first
        let history = self.get_question_answer_history(knowledge_base_id, None, None).await?;
        
        let total_answered = history.len() as i32;
        
        // Calculate average score
        let scores: Vec<i32> = history.iter()
            .filter_map(|(_, answer)| answer.ai_score)
            .collect();
        
        let avg_score = if !scores.is_empty() {
            let sum: i32 = scores.iter().sum();
            Some(sum as f64 / scores.len() as f64)
        } else {
            None
        };
        
        // Get recent performance (last 10 answers)
        let recent_scores: Vec<i32> = history.iter()
            .take(10) // Already ordered by answered_at DESC
            .filter_map(|(_, answer)| answer.ai_score)
            .collect();
        
        let recent_average = if !recent_scores.is_empty() {
            let sum: i32 = recent_scores.iter().sum();
            Some(sum as f64 / recent_scores.len() as f64)
        } else {
            None
        };
        
        // Get improvement trend (compare first half vs second half of answers)
        let improvement_trend = if scores.len() >= 4 {
            let half_point = scores.len() / 2;
            
            // Reverse the history to get chronological order
            let mut chronological_scores = scores.clone();
            chronological_scores.reverse();
            
            let first_half: Vec<i32> = chronological_scores.iter().take(half_point).cloned().collect();
            let second_half: Vec<i32> = chronological_scores.iter().skip(half_point).cloned().collect();
            
            let first_avg = first_half.iter().sum::<i32>() as f64 / first_half.len() as f64;
            let second_avg = second_half.iter().sum::<i32>() as f64 / second_half.len() as f64;
            
            if second_avg > first_avg + 5.0 {
                Some("improving".to_string())
            } else if first_avg > second_avg + 5.0 {
                Some("declining".to_string())
            } else {
                Some("stable".to_string())
            }
        } else {
            None
        };
        
        Ok(LearningProgress {
            total_questions_answered: total_answered,
            average_score: avg_score,
            recent_average_score: recent_average,
            improvement_trend,
            total_review_sessions: 0, // Will be calculated separately if needed
        })
    }
    
    // AI Config CRUD operations
    pub async fn save_ai_config(&self, config: &AIConfig) -> Result<(), sqlx::Error> {
        let provider_str = config.provider.to_string();
        
        // Delete existing config first (since we only want one config)
        sqlx::query("DELETE FROM ai_config")
            .execute(&self.pool)
            .await?;
        
        sqlx::query(
            "INSERT INTO ai_config (provider, api_key, api_url, model_name, max_tokens, temperature, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&provider_str)
        .bind(&config.api_key)
        .bind(&config.api_url)
        .bind(&config.model_name)
        .bind(&config.max_tokens)
        .bind(&config.temperature)
        .bind(&config.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_ai_config(&self) -> Result<Option<AIConfig>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, provider, api_key, api_url, model_name, max_tokens, temperature, updated_at FROM ai_config ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let provider = match row.get::<String, _>("provider").as_str() {
                "deepseek" => AIProvider::DeepSeek,
                "local" => AIProvider::Local,
                "openai" => AIProvider::OpenAI,
                _ => AIProvider::DeepSeek, // Default fallback
            };
            
            Ok(Some(AIConfig {
                id: Some(row.get("id")),
                provider,
                api_key: row.get("api_key"),
                api_url: row.get("api_url"),
                model_name: row.get("model_name"),
                max_tokens: row.get("max_tokens"),
                temperature: row.get("temperature"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }
}