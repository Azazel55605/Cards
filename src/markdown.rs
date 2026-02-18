use iced::widget::canvas::Frame;
use iced::{Color, Point};
use crate::text_renderer::TextRenderer;
use crate::text_processor::TextProcessor;

/// MarkdownRenderer - now a simple wrapper around the new text processing system
pub struct MarkdownRenderer {
    text_processor: TextProcessor,
    text_renderer: TextRenderer,
}

impl MarkdownRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self {
            text_processor: TextProcessor::new(),
            text_renderer: TextRenderer::new(text_color, max_width),
        }
    }

    /// Render text with markdown support via <md> tags
    pub fn render(&self, frame: &mut Frame, text: &str, position: Point) -> f32 {
        // Process text (handles both plain text and <md> tags)
        let document = self.text_processor.process(text);

        // Render the document
        self.text_renderer.render(frame, &document, position)
    }
}

