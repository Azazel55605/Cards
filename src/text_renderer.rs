use iced::widget::canvas::{Frame, Text, Path, Stroke, Fill};
use iced::{Color, Point, alignment, Rectangle};
use crate::text_document::{TextDocument, TextLine, TextSegment, TextStyle};

/// Position of a rendered checkbox for hit detection
#[derive(Debug, Clone)]
pub struct CheckboxPosition {
    pub rect: Rectangle,
    pub line_index: usize,
    pub checked: bool,
}

/// Position of a rendered link for click detection
#[derive(Debug, Clone)]
pub struct LinkPosition {
    pub rect: Rectangle,
    pub url: String,
}

/// General text renderer - renders styled text documents
pub struct TextRenderer {
    pub text_color: Color,
    pub link_color: Color,
    pub code_bg_color: Color,
    pub max_width: f32,
    pub max_height: Option<f32>, // None = unlimited, Some(height) = clip at height
    pub font_regular: iced::Font,
    pub font_code: iced::Font,
}

impl TextRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self {
            text_color,
            link_color: Color::from_rgb8(88, 166, 255), // default blue
            code_bg_color: Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            max_width,
            max_height: None,
            font_regular: iced::Font::MONOSPACE,
            font_code: iced::Font::MONOSPACE,
        }
    }

    pub fn with_fonts(text_color: Color, max_width: f32, font_regular: iced::Font, font_code: iced::Font) -> Self {
        Self {
            text_color,
            link_color: Color::from_rgb8(88, 166, 255),
            code_bg_color: Color::from_rgba(0.5, 0.5, 0.5, 0.15),
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
            link_color: Color::from_rgb8(88, 166, 255),
            code_bg_color: Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            max_width,
            max_height: Some(max_height),
            font_regular,
            font_code,
        }
    }

    pub fn with_link_color(mut self, color: Color) -> Self {
        self.link_color = color;
        self
    }

    pub fn with_code_bg(mut self, color: Color) -> Self {
        self.code_bg_color = color;
        self
    }

    /// Render a text document. Returns (height, checkboxes, links).
    pub fn render(&self, frame: &mut Frame, document: &TextDocument, position: Point) -> (f32, Vec<CheckboxPosition>, Vec<LinkPosition>) {
        let mut current_y = position.y;
        let start_y = position.y;
        let mut checkbox_positions = Vec::new();
        let mut link_positions: Vec<LinkPosition> = Vec::new();

        for line in &document.lines {
            if let Some(max_h) = self.max_height {
                if current_y - start_y >= max_h { break; }
            }

            // ── Horizontal rule ─────────────────────────────────────────────
            if line.is_rule {
                current_y += line.spacing_before;
                if let Some(max_h) = self.max_height {
                    if current_y - start_y >= max_h { break; }
                }
                let rule_color = Color { a: self.text_color.a * 0.35, ..self.text_color };
                let rule = Path::line(
                    Point::new(position.x, current_y + 4.0),
                    Point::new(position.x + self.max_width, current_y + 4.0),
                );
                frame.stroke(&rule, Stroke::default().with_color(rule_color).with_width(1.0));
                current_y += 8.0 + line.spacing_after;
                continue;
            }

            if line.is_empty() {
                current_y += 8.0;
                continue;
            }

            current_y += line.spacing_before;

            if let Some(max_h) = self.max_height {
                if current_y - start_y >= max_h { break; }
            }

            let line_height = self.calculate_line_height(line);
            let (new_y, checkbox_pos, mut line_links) = self.render_line(frame, line, position.x + line.indent, current_y, line_height, start_y);
            current_y = new_y;

            if let Some(pos) = checkbox_pos {
                checkbox_positions.push(pos);
            }
            link_positions.append(&mut line_links);

            current_y += line.spacing_after;
        }

        (current_y - position.y, checkbox_positions, link_positions)
    }

    fn calculate_line_height(&self, line: &TextLine) -> f32 {
        let max_size = line.segments.iter()
            .map(|seg| seg.style.size)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(14.0);
        // Use same line height calculation as editor: 21.0 * (size/14.0)
        21.0 * (max_size / 14.0)
    }

    fn render_line(&self, frame: &mut Frame, line: &TextLine, x: f32, y: f32, line_height: f32, start_y: f32) -> (f32, Option<CheckboxPosition>, Vec<LinkPosition>) {
        let mut current_x = x;
        let mut current_y = y;
        // (text, style, x_pos, link_url)
        let mut line_segments: Vec<(String, TextStyle, f32, Option<String>)> = Vec::new();
        let mut checkbox_pos: Option<CheckboxPosition> = None;
        let mut link_positions: Vec<LinkPosition> = Vec::new();

        // Render checkbox if present
        if let Some(checkbox) = &line.checkbox {
            let checkbox_size = 14.0;
            let checkbox_x = x - 20.0;
            let checkbox_y = y + 2.0;

            let checkbox_rect = Rectangle {
                x: checkbox_x,
                y: checkbox_y,
                width: checkbox_size,
                height: checkbox_size,
            };

            // Draw checkbox as a clean rectangle (no arc glitches)
            frame.stroke(
                &Path::rectangle(Point::new(checkbox_x, checkbox_y), iced::Size::new(checkbox_size, checkbox_size)),
                Stroke::default().with_color(self.text_color).with_width(1.5),
            );

            if checkbox.checked {
                let check_offset = 3.0;
                let check_path = Path::new(|builder| {
                    builder.move_to(Point::new(checkbox_x + check_offset, checkbox_y + checkbox_size / 2.0));
                    builder.line_to(Point::new(checkbox_x + checkbox_size / 2.5, checkbox_y + checkbox_size - check_offset));
                    builder.line_to(Point::new(checkbox_x + checkbox_size - check_offset, checkbox_y + check_offset));
                });
                frame.stroke(&check_path, Stroke::default().with_color(self.text_color).with_width(2.0));
            }

            checkbox_pos = Some(CheckboxPosition {
                rect: checkbox_rect,
                line_index: checkbox.line_index,
                checked: checkbox.checked,
            });
        }

        // Build segments for this visual line, wrapping as needed
        for segment in &line.segments {
            let char_width = 8.43 * (segment.style.size / 14.0);
            let max_chars = ((self.max_width / char_width).floor() as usize).saturating_sub(1).max(1);

            let mut buffer = String::new();
            let chars: Vec<char> = segment.text.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                if let Some(max_h) = self.max_height {
                    if current_y + line_height - start_y > max_h {
                        return (current_y, checkbox_pos, link_positions);
                    }
                }

                let ch = chars[i];

                if buffer.chars().count() >= max_chars {
                    if let Some(last_space_idx) = buffer.rfind(' ') {
                        if !segment.style.is_code {
                            let before_space = buffer[..last_space_idx].to_string();
                            let after_space = buffer[last_space_idx + 1..].to_string();

                            if !before_space.is_empty() {
                                line_segments.push((before_space, segment.style, current_x, segment.link_url.clone()));
                            }

                            self.render_segments(frame, &line_segments, current_y, &mut link_positions, line_height);
                            line_segments.clear();
                            current_x = x;
                            current_y += line_height;

                            if let Some(max_h) = self.max_height {
                                if current_y + line_height - start_y > max_h {
                                    return (current_y, checkbox_pos, link_positions);
                                }
                            }

                            buffer = after_space;
                            i += 1;
                            continue;
                        }
                    }

                    if !buffer.is_empty() {
                        line_segments.push((buffer.clone(), segment.style, current_x, segment.link_url.clone()));
                    }

                    self.render_segments(frame, &line_segments, current_y, &mut link_positions, line_height);
                    line_segments.clear();
                    current_x = x;
                    current_y += line_height;

                    if let Some(max_h) = self.max_height {
                        if current_y + line_height - start_y > max_h {
                            return (current_y, checkbox_pos, link_positions);
                        }
                    }

                    buffer.clear();
                }

                buffer.push(ch);
                i += 1;
            }

            if !buffer.is_empty() {
                line_segments.push((buffer.clone(), segment.style, current_x, segment.link_url.clone()));
                current_x += buffer.chars().count() as f32 * char_width;
            }
        }

        if !line_segments.is_empty() {
            if let Some(max_h) = self.max_height {
                if current_y + line_height - start_y > max_h {
                    return (current_y, checkbox_pos, link_positions);
                }
            }

            self.render_segments(frame, &line_segments, current_y, &mut link_positions, line_height);
            current_y += line_height;
        }

        (current_y, checkbox_pos, link_positions)
    }

    fn render_segments(&self, frame: &mut Frame, segments: &[(String, TextStyle, f32, Option<String>)], y: f32, link_positions: &mut Vec<LinkPosition>, line_height: f32) {
        for (text, style, x, link_url) in segments {
            self.render_text_segment(frame, text, *x, y, *style);
            // Record link hit-rect
            if let Some(url) = link_url {
                let char_width = style.size * 0.55;
                let text_width = text.chars().count() as f32 * char_width;
                link_positions.push(LinkPosition {
                    rect: Rectangle { x: *x, y, width: text_width, height: line_height },
                    url: url.clone(),
                });
            }
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
            if style.is_link {
                self.link_color
            } else if style.is_code {
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

        let char_width = style.size * 0.55;
        let text_width = text.len() as f32 * char_width;

        // Draw inline code background before the text
        if style.is_code {
            let pad_x = 3.0_f32;
            let pad_y = 1.0_f32;
            let bg_rect = iced::Rectangle {
                x: x - pad_x,
                y: y - pad_y,
                width: text_width + pad_x * 2.0,
                height: style.size * 1.3 + pad_y * 2.0,
            };
            let bg_path = Path::rounded_rectangle(
                iced::Point::new(bg_rect.x, bg_rect.y),
                iced::Size::new(bg_rect.width, bg_rect.height),
                3.0_f32.into(),
            );
            frame.fill(&bg_path, Fill {
                style: iced::widget::canvas::Style::Solid(self.code_bg_color),
                ..Default::default()
            });
        }

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

