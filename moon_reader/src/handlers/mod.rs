// HTTP request handlers module
pub mod knowledge_base;
pub mod document;
pub mod ai_quiz;
pub mod review;
pub mod ai_config;

// Re-export handler functions for easy access
pub use knowledge_base::*;
pub use document::*;
pub use ai_quiz::*;
pub use review::*;
pub use ai_config::*;