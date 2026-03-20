use iced::widget::canvas::Frame;
use iced::{Color, Point};
use crate::text_renderer::{TextRenderer, CheckboxPosition, LinkPosition, MathPosition};
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

    pub fn with_fonts_size_height_and_link(
        text_color: Color,
        max_width: f32,
        max_height: f32,
        font: iced::Font,
        font_size: f32,
        link_color: Color,
    ) -> Self {
        Self {
            text_processor: TextProcessor::with_font_size(font_size),
            text_renderer: TextRenderer::with_fonts_and_height(text_color, max_width, max_height, font, font)
                .with_link_color(link_color),
        }
    }

    pub fn set_fonts(&mut self, font: iced::Font) {
        self.text_renderer.font_regular = font;
        self.text_renderer.font_code = font;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.text_processor.set_font_size(font_size);
    }

    pub fn set_code_bg(&mut self, color: Color) {
        self.text_renderer.code_bg_color = color;
    }

    /// Render the entire string as pure markdown.
    /// Returns (height, checkboxes, links, math_positions).
    pub fn render_as_markdown(&self, frame: &mut Frame, text: &str, position: Point) -> (f32, Vec<CheckboxPosition>, Vec<LinkPosition>, Vec<MathPosition>) {
        let document = self.text_processor.parse_full_markdown(text);
        self.text_renderer.render(frame, &document, position)
    }
}
