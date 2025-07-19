#[cfg(test)]
mod tests {
    use crate::database::{create_connection_pool, DatabaseManager};
    use crate::models::{Document, DocumentType, Question, Answer, ReviewSession, AIConfig, AIProvider};
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        // Use in-memory database for tests
        let database_url = "sqlite::memory:";
        
        let pool = create_connection_pool(database_url).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_knowledge_base_crud() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Test create
        let kb = db.create_knowledge_base("Test KB", Some("Test description")).await.unwrap();
        assert_eq!(kb.name, "Test KB");
        assert_eq!(kb.description, Some("Test description".to_string()));

        // Test get by id
        let retrieved_kb = db.get_knowledge_base_by_id(&kb.id).await.unwrap();
        assert!(retrieved_kb.is_some());
        assert_eq!(retrieved_kb.unwrap().name, "Test KB");

        // Test get all
        let all_kbs = db.get_knowledge_bases().await.unwrap();
        assert_eq!(all_kbs.len(), 1);

        // Test update
        let updated = db.update_knowledge_base(&kb.id, "Updated KB", Some("Updated description")).await.unwrap();
        assert!(updated);

        // Test delete
        let deleted = db.delete_knowledge_base(&kb.id).await.unwrap();
        assert!(deleted);

        // Verify deletion
        let retrieved_kb = db.get_knowledge_base_by_id(&kb.id).await.unwrap();
        assert!(retrieved_kb.is_none());
    }

    #[tokio::test]
    async fn test_document_crud() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base first
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create a document
        let document = Document::new(
            kb.id.clone(),
            "test.pdf".to_string(),
            DocumentType::Pdf,
            "/path/to/test.pdf".to_string(),
            1024,
            Some("Test content".to_string()),
        );

        // Test save document
        db.save_document(&document).await.unwrap();

        // Test get documents by knowledge base
        let documents = db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].filename, "test.pdf");

        // Test delete document
        let deleted = db.delete_document(&document.id).await.unwrap();
        assert!(deleted);

        // Verify deletion
        let documents = db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
        assert_eq!(documents.len(), 0);
    }

    #[tokio::test]
    async fn test_question_answer_crud() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base first
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create a question
        let question = Question::new(
            kb.id.clone(),
            "What is the capital of France?".to_string(),
            Some("Geography context".to_string()),
        );

        // Test save question
        db.save_question(&question).await.unwrap();

        // Test get questions by knowledge base
        let questions = db.get_questions_by_knowledge_base(&kb.id).await.unwrap();
        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].question_text, "What is the capital of France?");

        // Create an answer
        let answer = Answer::new(question.id.clone(), "Paris".to_string());

        // Test save answer
        db.save_answer(&answer).await.unwrap();

        // Test get answers by question
        let answers = db.get_answers_by_question(&question.id).await.unwrap();
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].user_answer, "Paris");
    }

    #[tokio::test]
    async fn test_review_session_crud() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base first
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create a review session
        let session = ReviewSession::new(kb.id.clone(), 5);

        // Test save review session
        db.save_review_session(&session).await.unwrap();

        // Test get review sessions by knowledge base
        let sessions = db.get_review_sessions_by_knowledge_base(&kb.id).await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].questions_count, 5);
    }

    #[tokio::test]
    async fn test_ai_config_crud() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create AI config
        let config = AIConfig::new(
            AIProvider::DeepSeek,
            Some("test-api-key".to_string()),
            Some("https://api.deepseek.com".to_string()),
            Some("deepseek-chat".to_string()),
            1000,
            0.7,
        );

        // Test save AI config
        db.save_ai_config(&config).await.unwrap();

        // Test get AI config
        let retrieved_config = db.get_ai_config().await.unwrap();
        assert!(retrieved_config.is_some());
        let retrieved_config = retrieved_config.unwrap();
        assert_eq!(retrieved_config.api_key, Some("test-api-key".to_string()));
        assert_eq!(retrieved_config.max_tokens, 1000);

        // Test update AI config (save again should replace)
        let new_config = AIConfig::new(
            AIProvider::OpenAI,
            Some("new-api-key".to_string()),
            Some("https://api.openai.com".to_string()),
            Some("gpt-4".to_string()),
            2000,
            0.5,
        );

        db.save_ai_config(&new_config).await.unwrap();

        // Verify update
        let updated_config = db.get_ai_config().await.unwrap();
        assert!(updated_config.is_some());
        let updated_config = updated_config.unwrap();
        assert_eq!(updated_config.api_key, Some("new-api-key".to_string()));
        assert_eq!(updated_config.max_tokens, 2000);
    }

    #[tokio::test]
    async fn test_question_answer_history() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create multiple questions and answers
        for i in 1..=3 {
            let question = Question::new(
                kb.id.clone(),
                format!("Question {}", i),
                Some(format!("Context {}", i)),
            );
            db.save_question(&question).await.unwrap();

            let mut answer = Answer::new(question.id.clone(), format!("Answer {}", i));
            answer.ai_score = Some(80 + i * 5); // Scores: 85, 90, 95
            db.save_answer(&answer).await.unwrap();
        }

        // Test get question-answer history
        let history = db.get_question_answer_history(&kb.id, None, None).await.unwrap();
        assert_eq!(history.len(), 3);

        // Test with limit
        let limited_history = db.get_question_answer_history(&kb.id, Some(2), None).await.unwrap();
        assert_eq!(limited_history.len(), 2);

        // Test with offset
        let offset_history = db.get_question_answer_history(&kb.id, Some(2), Some(1)).await.unwrap();
        assert_eq!(offset_history.len(), 2);
    }

    #[tokio::test]
    async fn test_filtered_history() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create questions and answers with different scores
        for i in 1..=5 {
            let question = Question::new(
                kb.id.clone(),
                format!("Question {}", i),
                None,
            );
            db.save_question(&question).await.unwrap();

            let mut answer = Answer::new(question.id.clone(), format!("Answer {}", i));
            answer.ai_score = Some(i * 20); // Scores: 20, 40, 60, 80, 100
            db.save_answer(&answer).await.unwrap();
        }

        // Test filter by minimum score
        let filtered_history = db.get_filtered_history(&kb.id, Some(60), None, None, None).await.unwrap();
        assert_eq!(filtered_history.len(), 3); // Scores 60, 80, 100

        // Test filter by score range
        let range_history = db.get_filtered_history(&kb.id, Some(40), Some(80), None, None).await.unwrap();
        assert_eq!(range_history.len(), 3); // Scores 40, 60, 80
    }

    #[tokio::test]
    async fn test_review_session_management() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create a review session
        let session = ReviewSession::new(kb.id.clone(), 5);
        let session_id = session.id.clone();
        
        db.save_review_session(&session).await.unwrap();

        // Test get review session by id
        let retrieved_session = db.get_review_session_by_id(&session_id).await.unwrap();
        assert!(retrieved_session.is_some());
        assert_eq!(retrieved_session.unwrap().questions_count, 5);

        // Test update review session score
        let updated = db.update_review_session_score(&session_id, 87.5).await.unwrap();
        assert!(updated);

        // Verify the score was updated
        let updated_session = db.get_review_session_by_id(&session_id).await.unwrap();
        assert!(updated_session.is_some());
        assert_eq!(updated_session.unwrap().average_score, Some(87.5));
    }

    #[tokio::test]
    async fn test_random_review_questions() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create multiple questions and answers
        for i in 1..=5 {
            let question = Question::new(
                kb.id.clone(),
                format!("Question {}", i),
                Some(format!("Context {}", i)),
            );
            db.save_question(&question).await.unwrap();

            let mut answer = Answer::new(question.id.clone(), format!("Answer {}", i));
            answer.ai_score = Some(80 + i * 2); // Scores: 82, 84, 86, 88, 90
            db.save_answer(&answer).await.unwrap();
        }

        // Test get random review questions
        let random_questions = db.get_random_review_questions(&kb.id, 3).await.unwrap();
        assert_eq!(random_questions.len(), 3);

        // Test with more questions than available
        let all_questions = db.get_random_review_questions(&kb.id, 10).await.unwrap();
        assert_eq!(all_questions.len(), 5); // Should return all available
    }

    #[tokio::test]
    async fn test_learning_progress() {
        let pool = setup_test_db().await;
        let db = DatabaseManager::new(pool);

        // Create a knowledge base
        let kb = db.create_knowledge_base("Test KB", None).await.unwrap();

        // Create questions and answers with varying scores
        let scores = vec![60, 70, 80, 90, 95];
        for (i, score) in scores.iter().enumerate() {
            let question = Question::new(
                kb.id.clone(),
                format!("Question {}", i + 1),
                None,
            );
            db.save_question(&question).await.unwrap();

            let mut answer = Answer::new(question.id.clone(), format!("Answer {}", i + 1));
            answer.ai_score = Some(*score);
            db.save_answer(&answer).await.unwrap();
        }

        // Test get learning progress
        let progress = db.get_learning_progress(&kb.id).await.unwrap();
        
        assert_eq!(progress.total_questions_answered, 5);
        assert!(progress.average_score.is_some());
        assert_eq!(progress.average_score.unwrap(), 79.0); // (60+70+80+90+95)/5 = 79
        assert!(progress.recent_average_score.is_some());
        assert!(progress.improvement_trend.is_some());
        assert_eq!(progress.improvement_trend.unwrap(), "improving"); // Later scores are higher
    }
}