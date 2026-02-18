use crate::text_document::{TextDocument, TextLine, TextStyle};
use crate::markdown_parser::MarkdownParser;

/// Main text processor - handles plain text and <md> tags
pub struct TextProcessor {
    markdown_parser: MarkdownParser,
}

impl TextProcessor {
    pub fn new() -> Self {
        Self {
            markdown_parser: MarkdownParser::new(),
        }
    }

    /// Process text that may contain <md>...</md> tags
    pub fn process(&self, text: &str) -> TextDocument {
        let mut document = TextDocument::new();
        let mut start = 0;

        while start < text.len() {
            // Look for <md> tag
            if let Some(md_start_pos) = text[start..].find("<md>") {
                let actual_md_start = start + md_start_pos;

                // Process text before <md> tag as plain text
                if actual_md_start > start {
                    let plain_text = &text[start..actual_md_start];
                    self.process_plain_text(&mut document, plain_text);
                }

                // Find closing </md> tag
                let md_content_start = actual_md_start + 4;
                if let Some(md_end_pos) = text[md_content_start..].find("</md>") {
                    let actual_md_end = md_content_start + md_end_pos;
                    let markdown_content = &text[md_content_start..actual_md_end];

                    // Process markdown content
                    let md_document = self.markdown_parser.parse(markdown_content);
                    for line in md_document.lines {
                        document.add_line(line);
                    }

                    start = actual_md_end + 5; // After "</md>"
                } else {
                    // No closing tag, treat rest as plain text
                    let remaining = &text[actual_md_start..];
                    self.process_plain_text(&mut document, remaining);
                    break;
                }
            } else {
                // No more <md> tags, process rest as plain text
                let remaining = &text[start..];
                self.process_plain_text(&mut document, remaining);
                break;
            }
        }

        document
    }

    /// Process plain text (no markdown)
    fn process_plain_text(&self, document: &mut TextDocument, text: &str) {
        if text.trim().is_empty() {
            return;
        }

        let default_style = TextStyle::default();

        for line in text.lines() {
            if line.trim().is_empty() {
                // Empty line
                document.add_line(TextLine::new());
            } else {
                let mut text_line = TextLine::new();
                text_line.add_segment(line.to_string(), default_style);
                document.add_line(text_line);
            }
        }
    }
}

