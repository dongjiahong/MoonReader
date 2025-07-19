-- Initial database schema for knowledge accumulation system

-- 知识库表
CREATE TABLE knowledge_bases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 文档表
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    knowledge_base_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    content_text TEXT,
    upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE
);

-- 问题表
CREATE TABLE questions (
    id TEXT PRIMARY KEY,
    knowledge_base_id TEXT NOT NULL,
    question_text TEXT NOT NULL,
    context_snippet TEXT,
    generated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE
);

-- 答案表
CREATE TABLE answers (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL,
    user_answer TEXT NOT NULL,
    ai_score INTEGER,
    ai_feedback TEXT,
    ai_suggestions TEXT,
    answered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
);

-- 复习会话表
CREATE TABLE review_sessions (
    id TEXT PRIMARY KEY,
    knowledge_base_id TEXT NOT NULL,
    questions_count INTEGER NOT NULL,
    average_score REAL,
    session_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE
);

-- AI配置表
CREATE TABLE ai_config (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    api_key TEXT,
    api_url TEXT,
    model_name TEXT,
    max_tokens INTEGER DEFAULT 1000,
    temperature REAL DEFAULT 0.7,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);