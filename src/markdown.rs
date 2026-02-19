use iced::widget::canvas::Frame;
use iced::{Color, Point};
use crate::text_renderer::{TextRenderer, CheckboxPosition};
use crate::text_processor::TextProcessor;

/// MarkdownRenderer - now a simple wrapper around the new text processing system
pub struct MarkdownRenderer {
    text_processor: TextProcessor,
    text_renderer: TextRenderer,
}

impl MarkdownRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self::with_fonts(text_color, max_width, iced::Font::MONOSPACE)
    }

    pub fn with_fonts(text_color: Color, max_width: f32, font: iced::Font) -> Self {
        Self::with_fonts_and_size(text_color, max_width, font, 14.0)
    }

    pub fn with_fonts_and_size(text_color: Color, max_width: f32, font: iced::Font, font_size: f32) -> Self {
        Self {
            text_processor: TextProcessor::with_font_size(font_size),
            text_renderer: TextRenderer::with_fonts(text_color, max_width, font, font),
        }
    }

    pub fn with_fonts_size_and_height(
        text_color: Color, 
        max_width: f32, 
        max_height: f32,
        font: iced::Font, 
        font_size: f32
    ) -> Self {
        Self {
            text_processor: TextProcessor::with_font_size(font_size),
            text_renderer: TextRenderer::with_fonts_and_height(text_color, max_width, max_height, font, font),
        }
    }

    pub fn set_fonts(&mut self, font: iced::Font) {
        self.text_renderer.font_regular = font;
        self.text_renderer.font_code = font;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.text_processor.set_font_size(font_size);
    }

    /// Render text with markdown support via <md> tags
    pub fn render(&self, frame: &mut Frame, text: &str, position: Point) -> (f32, Vec<CheckboxPosition>) {
        // Process text (handles both plain text and <md> tags)
        let document = self.text_processor.process(text);

        // Render the document
        self.text_renderer.render(frame, &document, position)
    }
}

