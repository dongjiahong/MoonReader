use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PDF parsing error: {0}")]
    Pdf(String),
    #[error("EPUB parsing error: {0}")]
    Epub(String),
    #[error("Unsupported file format")]
    UnsupportedFormat,
}

#[derive(Debug, Clone)]
pub enum DocumentParser {
    Pdf,
    Epub,
    Txt,
}

impl DocumentParser {
    pub async fn parse(&self, file_path: &Path) -> Result<String, ParseError> {
        match self {
            DocumentParser::Pdf => Self::parse_pdf(file_path).await,
            DocumentParser::Epub => Self::parse_epub(file_path).await,
            DocumentParser::Txt => Self::parse_txt(file_path).await,
        }
    }

    pub fn supported_extensions(&self) -> Vec<&'static str> {
        match self {
            DocumentParser::Pdf => vec!["pdf"],
            DocumentParser::Epub => vec!["epub"],
            DocumentParser::Txt => vec!["txt"],
        }
    }

    async fn parse_pdf(file_path: &Path) -> Result<String, ParseError> {
        let content = tokio::task::spawn_blocking({
            let path = file_path.to_owned();
            move || -> Result<String, ParseError> {
                let bytes = std::fs::read(&path)?;
                pdf_extract::extract_text_from_mem(&bytes)
                    .map_err(|e| ParseError::Pdf(e.to_string()))
            }
        })
        .await
        .map_err(|e| ParseError::Pdf(format!("Task join error: {}", e)))??;
        
        Ok(content)
    }

    async fn parse_epub(file_path: &Path) -> Result<String, ParseError> {
        let content = tokio::task::spawn_blocking({
            let path = file_path.to_owned();
            move || -> Result<String, ParseError> {
                let mut doc = epub::doc::EpubDoc::new(&path)
                    .map_err(|e| ParseError::Epub(e.to_string()))?;
                
                let mut content = String::new();
                let spine = doc.spine.clone();
                
                for spine_item in spine {
                    if let Some((item_content, _)) = doc.get_resource_str(&spine_item.idref) {
                        // Simple HTML tag removal - in production, consider using a proper HTML parser
                        let text = item_content
                            .replace("<br>", "\n")
                            .replace("<br/>", "\n")
                            .replace("<p>", "\n")
                            .replace("</p>", "\n");
                        
                        // Remove HTML tags using a simple regex-like approach
                        let mut clean_text = String::new();
                        let mut in_tag = false;
                        for ch in text.chars() {
                            match ch {
                                '<' => in_tag = true,
                                '>' => in_tag = false,
                                _ if !in_tag => clean_text.push(ch),
                                _ => {}
                            }
                        }
                        
                        content.push_str(&clean_text);
                        content.push('\n');
                    }
                }
                
                Ok(content)
            }
        })
        .await
        .map_err(|e| ParseError::Epub(format!("Task join error: {}", e)))??;
        
        Ok(content)
    }

    async fn parse_txt(file_path: &Path) -> Result<String, ParseError> {
        let content = tokio::fs::read_to_string(file_path).await?;
        Ok(content)
    }
}

pub struct DocumentParserFactory;

impl DocumentParserFactory {
    pub fn get_parser(file_extension: &str) -> Option<DocumentParser> {
        match file_extension.to_lowercase().as_str() {
            "pdf" => Some(DocumentParser::Pdf),
            "epub" => Some(DocumentParser::Epub),
            "txt" => Some(DocumentParser::Txt),
            _ => None,
        }
    }
    
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["pdf", "epub", "txt"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_txt_parser() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "This is a test document.\nWith multiple lines.";
        temp_file.write_all(test_content.as_bytes()).unwrap();
        
        let parser = DocumentParser::Txt;
        let result = parser.parse(temp_file.path()).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[tokio::test]
    async fn test_parser_factory() {
        assert!(DocumentParserFactory::get_parser("pdf").is_some());
        assert!(DocumentParserFactory::get_parser("epub").is_some());
        assert!(DocumentParserFactory::get_parser("txt").is_some());
        assert!(DocumentParserFactory::get_parser("doc").is_none());
    }

    #[test]
    fn test_supported_extensions() {
        let extensions = DocumentParserFactory::supported_extensions();
        assert!(extensions.contains(&"pdf"));
        assert!(extensions.contains(&"epub"));
        assert!(extensions.contains(&"txt"));
    }

    #[tokio::test]
    async fn test_pdf_parser_with_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"This is not a PDF file").unwrap();
        
        let parser = DocumentParser::Pdf;
        let result = parser.parse(temp_file.path()).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Pdf(_) => {}, // Expected
            _ => panic!("Expected PDF parsing error"),
        }
    }

    #[tokio::test]
    async fn test_epub_parser_with_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"This is not an EPUB file").unwrap();
        
        let parser = DocumentParser::Epub;
        let result = parser.parse(temp_file.path()).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Epub(_) => {}, // Expected
            _ => panic!("Expected EPUB parsing error"),
        }
    }

    #[tokio::test]
    async fn test_parser_supported_extensions() {
        let pdf_parser = DocumentParser::Pdf;
        let epub_parser = DocumentParser::Epub;
        let txt_parser = DocumentParser::Txt;
        
        assert_eq!(pdf_parser.supported_extensions(), vec!["pdf"]);
        assert_eq!(epub_parser.supported_extensions(), vec!["epub"]);
        assert_eq!(txt_parser.supported_extensions(), vec!["txt"]);
    }
}