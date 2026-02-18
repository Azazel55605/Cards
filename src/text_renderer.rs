use iced::widget::canvas::{Frame, Text, Path, Stroke};
use iced::{Color, Point, alignment};
use crate::text_document::{TextDocument, TextLine, TextSegment, TextStyle};

/// General text renderer - renders styled text documents
pub struct TextRenderer {
    pub text_color: Color,
    pub max_width: f32,
    pub max_height: Option<f32>, // None = unlimited, Some(height) = clip at height
    pub font_regular: iced::Font,
    pub font_code: iced::Font,
}

impl TextRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self {
            text_color,
            max_width,
            max_height: None,
            font_regular: iced::Font::MONOSPACE,
            font_code: iced::Font::MONOSPACE,
        }
    }

    pub fn with_fonts(text_color: Color, max_width: f32, font_regular: iced::Font, font_code: iced::Font) -> Self {
        Self {
            text_color,
            max_width,
            max_height: None,
            font_regular,
            font_code,
        }
    }

    pub fn with_fonts_and_height(
        text_color: Color,
        max_width: f32,
        max_height: f32,
        font_regular: iced::Font,
        font_code: iced::Font
    ) -> Self {
        Self {
            text_color,
            max_width,
            max_height: Some(max_height),
            font_regular,
            font_code,
        }
    }

    /// Render a text document
    pub fn render(&self, frame: &mut Frame, document: &TextDocument, position: Point) -> f32 {
        let mut current_y = position.y;
        let start_y = position.y;

        for line in &document.lines {
            // Check if we've exceeded max_height
            if let Some(max_h) = self.max_height {
                if current_y - start_y >= max_h {
                    // Stop rendering - we've reached the bottom boundary
                    break;
                }
            }

            if line.is_empty() {
                // Empty line - just add spacing
                current_y += 8.0;
                continue;
            }

            // Add spacing before line
            current_y += line.spacing_before;

            // Check again after spacing
            if let Some(max_h) = self.max_height {
                if current_y - start_y >= max_h {
                    break;
                }
            }

            // Render the line
            let line_height = self.calculate_line_height(line);
            current_y = self.render_line(frame, line, position.x + line.indent, current_y, line_height, start_y);

            // Add spacing after line
            current_y += line.spacing_after;
        }

        current_y - position.y
    }

    fn calculate_line_height(&self, line: &TextLine) -> f32 {
        let max_size = line.segments.iter()
            .map(|seg| seg.style.size)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(14.0);
        // Use same line height calculation as editor: 21.0 * (size/14.0)
        21.0 * (max_size / 14.0)
    }

    fn render_line(&self, frame: &mut Frame, line: &TextLine, x: f32, y: f32, line_height: f32, start_y: f32) -> f32 {
        let mut current_x = x;
        let mut current_y = y;
        let mut line_segments: Vec<(String, TextStyle, f32)> = Vec::new();

        // Build segments for this visual line, wrapping as needed
        for segment in &line.segments {
            // Use same character width calculation as editor: 8.43 * (size/14.0)
            let char_width = 8.43 * (segment.style.size / 14.0);
            let max_chars = ((self.max_width / char_width).floor() as usize).saturating_sub(1).max(1);

            let mut buffer = String::new();
            let chars: Vec<char> = segment.text.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                // Check if we would exceed max_height before rendering more
                if let Some(max_h) = self.max_height {
                    if current_y + line_height - start_y > max_h {
                        return current_y;
                    }
                }

                let ch = chars[i];

                // Check if adding this character would exceed max width
                // Use buffer.chars().count() like the editor does
                if buffer.chars().count() >= max_chars {
                    // Try to find last space in buffer to break there
                    if let Some(last_space_idx) = buffer.rfind(' ') {
                        // Break at space - only for normal text, not code
                        if !segment.style.is_code {
                            let before_space = buffer[..last_space_idx].to_string();
                            let after_space = buffer[last_space_idx + 1..].to_string();

                            if !before_space.is_empty() {
                                line_segments.push((before_space, segment.style, current_x));
                            }

                            // Render current line and wrap
                            self.render_segments(frame, &line_segments, current_y);
                            line_segments.clear();
                            current_x = x;
                            current_y += line_height;

                            // Check height again after wrapping
                            if let Some(max_h) = self.max_height {
                                if current_y + line_height - start_y > max_h {
                                    return current_y;
                                }
                            }

                            // Continue with remainder after space
                            buffer = after_space;
                            i += 1;
                            continue;
                        }
                    }

                    // No space found or is code - break at character boundary (long word)
                    if !buffer.is_empty() {
                        line_segments.push((buffer.clone(), segment.style, current_x));
                    }

                    // Render current line and wrap
                    self.render_segments(frame, &line_segments, current_y);
                    line_segments.clear();
                    current_x = x;
                    current_y += line_height;

                    // Check height again after wrapping
                    if let Some(max_h) = self.max_height {
                        if current_y + line_height - start_y > max_h {
                            return current_y;
                        }
                    }

                    buffer.clear();
                }

                // Add the character
                buffer.push(ch);
                i += 1;
            }

            // Flush remaining buffer for this segment
            if !buffer.is_empty() {
                line_segments.push((buffer.clone(), segment.style, current_x));
                current_x += buffer.chars().count() as f32 * char_width;
            }
        }

        // Render remaining segments
        if !line_segments.is_empty() {
            // Final check before rendering last line
            if let Some(max_h) = self.max_height {
                if current_y + line_height - start_y > max_h {
                    return current_y;
                }
            }

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
            // Code blocks use code font
            self.font_code
        } else if style.bold && style.italic {
            // Regular text with bold+italic
            iced::Font {
                family: self.font_regular.family,
                weight: iced::font::Weight::Bold,
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else if style.bold {
            // Regular text with bold
            iced::Font {
                family: self.font_regular.family,
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }
        } else if style.italic {
            // Regular text with italic
            iced::Font {
                family: self.font_regular.family,
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else {
            // Regular text
            self.font_regular
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

