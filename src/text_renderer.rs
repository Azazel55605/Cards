use iced::widget::canvas::{Frame, Text, Path, Stroke};
use iced::{Color, Point, alignment};
use crate::text_document::{TextDocument, TextLine, TextSegment, TextStyle};

/// General text renderer - renders styled text documents
pub struct TextRenderer {
    pub text_color: Color,
    pub max_width: f32,
}

impl TextRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self {
            text_color,
            max_width,
        }
    }

    /// Render a text document
    pub fn render(&self, frame: &mut Frame, document: &TextDocument, position: Point) -> f32 {
        let mut current_y = position.y;

        for line in &document.lines {
            if line.is_empty() {
                // Empty line - just add spacing
                current_y += 8.0;
                continue;
            }

            // Add spacing before line
            current_y += line.spacing_before;

            // Render the line
            let line_height = self.calculate_line_height(line);
            current_y = self.render_line(frame, line, position.x + line.indent, current_y, line_height);

            // Add spacing after line
            current_y += line.spacing_after;
        }

        current_y - position.y
    }

    fn calculate_line_height(&self, line: &TextLine) -> f32 {
        let max_size = line.segments.iter()
            .map(|seg| seg.style.size)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(12.0);
        max_size + 4.0
    }

    fn render_line(&self, frame: &mut Frame, line: &TextLine, x: f32, y: f32, line_height: f32) -> f32 {
        let mut current_x = x;
        let mut current_y = y;
        let mut line_segments: Vec<(String, TextStyle, f32)> = Vec::new(); // text, style, x_position
        let avg_char_width_multiplier = 0.55;

        // Build segments for this visual line, wrapping as needed
        for segment in &line.segments {
            let char_width = segment.style.size * avg_char_width_multiplier;
            
            // For code blocks, preserve all spaces and don't word-wrap
            if segment.style.is_code {
                // Render code as-is without word wrapping
                line_segments.push((segment.text.clone(), segment.style, current_x));
                current_x += segment.text.len() as f32 * char_width;
            } else {
                // Normal text - word wrap
                let words: Vec<&str> = segment.text.split_whitespace().collect();

                for (i, word) in words.iter().enumerate() {
                    let word_width = word.len() as f32 * char_width;
                    let space_width = char_width;
                    let needs_space = i > 0 || !line_segments.is_empty();
                    let total_width = word_width + if needs_space { space_width } else { 0.0 };

                    // Check if word fits on current line
                    if current_x + total_width > x + self.max_width && !line_segments.is_empty() {
                        // Render current line
                        self.render_segments(frame, &line_segments, current_y);
                        line_segments.clear();
                        current_x = x;
                        current_y += line_height;
                    }

                    // Add space if needed
                    if needs_space {
                        if !line_segments.is_empty() {
                            if let Some(last) = line_segments.last_mut() {
                                last.0.push(' ');
                            }
                        }
                        current_x += space_width;
                    }

                    // Add word
                    line_segments.push((word.to_string(), segment.style, current_x));
                    current_x += word_width;
                }
            }
        }

        // Render remaining segments
        if !line_segments.is_empty() {
            self.render_segments(frame, &line_segments, current_y);
            current_y += line_height;
        }

        current_y
    }

    fn render_segments(&self, frame: &mut Frame, segments: &[(String, TextStyle, f32)], y: f32) {
        for (text, style, x) in segments {
            self.render_text_segment(frame, text, *x, y, *style);
        }
    }

    fn render_text_segment(&self, frame: &mut Frame, text: &str, x: f32, y: f32, style: TextStyle) {
        let font = if style.is_code {
            iced::Font {
                family: iced::font::Family::Monospace,
                ..Default::default()
            }
        } else if style.bold && style.italic {
            iced::Font {
                weight: iced::font::Weight::Bold,
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else if style.bold {
            iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }
        } else if style.italic {
            iced::Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else {
            iced::Font::default()
        };

        let text_color = style.color.unwrap_or_else(|| {
            if style.is_code {
                Color {
                    r: self.text_color.r.min(1.0),
                    g: (self.text_color.g * 1.1).min(1.0),
                    b: (self.text_color.b * 1.2).min(1.0),
                    a: self.text_color.a,
                }
            } else if style.strikethrough {
                Color {
                    a: self.text_color.a * 0.5,
                    ..self.text_color
                }
            } else {
                self.text_color
            }
        });

        // Render text
        frame.fill_text(Text {
            content: text.to_string(),
            position: Point::new(x, y),
            color: text_color,
            size: style.size.into(),
            font,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: iced::widget::text::Shaping::Advanced,
            ..Default::default()
        });

        let char_width = style.size * 0.55;
        let text_width = text.len() as f32 * char_width;

        // Render strikethrough
        if style.strikethrough {
            let line_y = y + style.size * 0.45;
            let strike_line = Path::line(
                Point::new(x, line_y),
                Point::new(x + text_width, line_y)
            );
            frame.stroke(
                &strike_line,
                Stroke::default()
                    .with_color(text_color)
                    .with_width(1.0)
            );
        }

        // Render underline
        if style.underline {
            let line_y = y + style.size * 0.9;
            let underline = Path::line(
                Point::new(x, line_y),
                Point::new(x + text_width, line_y)
            );
            frame.stroke(
                &underline,
                Stroke::default()
                    .with_color(text_color)
                    .with_width(1.0)
            );
        }
    }
}

