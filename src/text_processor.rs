use crate::text_document::TextDocument;
use crate::markdown_parser::MarkdownParser;

/// Text processor — thin wrapper over MarkdownParser
pub struct TextProcessor {
    markdown_parser: MarkdownParser,
}

impl TextProcessor {
    pub fn new() -> Self {
        Self::with_font_size(14.0)
    }

    pub fn with_font_size(font_size: f32) -> Self {
        Self { markdown_parser: MarkdownParser::with_font_size(font_size) }
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.markdown_parser.set_base_font_size(font_size);
    }

    /// Parse the entire string as markdown.
    pub fn parse_full_markdown(&self, text: &str) -> TextDocument {
        self.markdown_parser.parse(text)
    }
}

