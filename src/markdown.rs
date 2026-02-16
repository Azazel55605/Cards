use iced::widget::canvas::{Frame, Text};
use iced::{Color, Point, alignment};
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, CodeBlockKind, Options};

pub struct MarkdownRenderer {
    pub text_color: Color,
    pub max_width: f32,
}


impl MarkdownRenderer {
    pub fn new(text_color: Color, max_width: f32) -> Self {
        Self {
            text_color,
            max_width,
        }
    }

    pub fn render(&self, frame: &mut Frame, markdown: &str, position: Point) -> f32 {
        // Enable all markdown extensions
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(markdown, options);

        let mut current_y = position.y;
        let mut current_x = position.x;
        let mut line_height = 14.0;
        let mut current_line = String::new();
        let mut current_size = 12.0;
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_strikethrough = false;
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            if current_y > position.y {
                                current_y += line_height * 0.5; // Add spacing before paragraph
                            }
                        }
                        Tag::Heading(level, _, _) => {
                            current_size = match level {
                                HeadingLevel::H1 => 20.0,
                                HeadingLevel::H2 => 18.0,
                                HeadingLevel::H3 => 16.0,
                                HeadingLevel::H4 => 14.0,
                                HeadingLevel::H5 => 13.0,
                                HeadingLevel::H6 => 12.0,
                            };
                            is_bold = true;
                            line_height = current_size + 4.0;
                            if current_y > position.y {
                                current_y += line_height * 0.5; // Add spacing before heading
                            }
                        }
                        Tag::Strong => is_bold = true,
                        Tag::Emphasis => is_italic = true,
                        Tag::Strikethrough => is_strikethrough = true,
                        Tag::List(_) => {
                            if current_y > position.y {
                                current_y += line_height * 0.3;
                            }
                        }
                        Tag::Item => {
                            // Render bullet point
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_x = position.x + 10.0;
                            current_line.push_str("• ");
                        }
                        Tag::CodeBlock(kind) => {
                            in_code_block = true;
                            code_block_lang = match kind {
                                CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                                _ => None,
                            };
                            current_size = 11.0;
                            line_height = 14.0;
                            if current_y > position.y {
                                current_y += line_height * 0.3;
                            }
                        }
                        Tag::Link(_, _url, _) => {
                            // For links, we'll just render the text for now
                            // In a full implementation, you could render the URL differently
                        }
                        Tag::BlockQuote => {
                            current_x = position.x + 15.0; // Indent blockquotes
                            if current_y > position.y {
                                current_y += line_height * 0.3;
                            }
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_y += line_height * 0.5; // Add spacing after paragraph
                        }
                        Tag::Heading(_, _, _) => {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_y += line_height * 0.3; // Add spacing after heading
                            is_bold = false;
                            current_size = 12.0;
                            line_height = 14.0;
                        }
                        Tag::Strong => is_bold = false,
                        Tag::Emphasis => is_italic = false,
                        Tag::Strikethrough => is_strikethrough = false,
                        Tag::Item => {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_x = position.x;
                        }
                        Tag::List(_) => {
                            current_y += line_height * 0.3;
                        }
                        Tag::CodeBlock(_) => {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, false, false, false, true);
                                current_y += line_height;
                                current_line.clear();
                            }
                            in_code_block = false;
                            code_block_lang = None;
                            current_size = 12.0;
                            line_height = 14.0;
                            current_y += line_height * 0.3;
                        }
                        Tag::BlockQuote => {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_x = position.x;
                            current_y += line_height * 0.3;
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    if in_code_block {
                        // For code blocks, preserve all text including whitespace
                        for line in text.lines() {
                            if !current_line.is_empty() {
                                self.render_line(frame, &current_line, current_x, current_y, current_size, false, false, false, true);
                                current_y += line_height;
                                current_line.clear();
                            }
                            current_line = line.to_string();
                            self.render_line(frame, &current_line, current_x + 10.0, current_y, current_size, false, false, false, true);
                            current_y += line_height;
                            current_line.clear();
                        }
                    } else {
                        // Word wrapping for normal text
                        let words: Vec<&str> = text.split_whitespace().collect();
                        for word in words {
                            let test_line = if current_line.is_empty() {
                                word.to_string()
                            } else {
                                format!("{} {}", current_line, word)
                            };

                            // Rough estimation of text width (chars * size * 0.6)
                            let estimated_width = test_line.len() as f32 * current_size * 0.6;

                            if estimated_width > self.max_width && !current_line.is_empty() {
                                // Render current line and start new one
                                self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                                current_y += line_height;
                                current_line = word.to_string();
                            } else {
                                if !current_line.is_empty() {
                                    current_line.push(' ');
                                }
                                current_line.push_str(word);
                            }
                        }
                    }
                }
                Event::SoftBreak => {
                    if in_code_block {
                        if !current_line.is_empty() {
                            self.render_line(frame, &current_line, current_x + 10.0, current_y, current_size, false, false, false, true);
                            current_line.clear();
                        }
                        current_y += line_height;
                    } else {
                        current_line.push(' ');
                    }
                }
                Event::HardBreak => {
                    if !current_line.is_empty() {
                        self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, in_code_block);
                        current_line.clear();
                    }
                    current_y += line_height;
                }
                Event::Code(code) => {
                    // Inline code - render with monospace style and background
                    let code_text = format!("`{}`", code);
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(&code_text);
                }
                Event::TaskListMarker(checked) => {
                    // Handle task list checkboxes
                    if checked {
                        current_line.insert_str(0, "☑ ");
                    } else {
                        current_line.insert_str(0, "☐ ");
                    }
                }
                Event::Rule => {
                    // Horizontal rule
                    if !current_line.is_empty() {
                        self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, false);
                        current_y += line_height;
                        current_line.clear();
                    }
                    // Draw a horizontal line
                    use iced::widget::canvas::{Path, Stroke};
                    let rule_y = current_y + line_height * 0.5;
                    let rule = Path::line(
                        Point::new(position.x, rule_y),
                        Point::new(position.x + self.max_width, rule_y)
                    );
                    frame.stroke(
                        &rule,
                        Stroke::default()
                            .with_color(Color { a: 0.3, ..self.text_color })
                            .with_width(1.0)
                    );
                    current_y += line_height * 1.5;
                }
                _ => {}
            }
        }

        // Render any remaining text
        if !current_line.is_empty() {
            self.render_line(frame, &current_line, current_x, current_y, current_size, is_bold, is_italic, is_strikethrough, in_code_block);
            current_y += line_height;
        }

        current_y - position.y // Return total height used
    }

    fn render_line(&self, frame: &mut Frame, text: &str, x: f32, y: f32, size: f32, bold: bool, italic: bool, strikethrough: bool, is_code: bool) {
        let font = if is_code {
            // Monospace font for code
            iced::Font {
                family: iced::font::Family::Monospace,
                ..Default::default()
            }
        } else if bold && italic {
            iced::Font {
                weight: iced::font::Weight::Bold,
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else if bold {
            iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }
        } else if italic {
            iced::Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            }
        } else {
            iced::Font::default()
        };

        let text_color = if is_code {
            // Slightly different color for code
            Color {
                r: self.text_color.r * 0.9,
                g: self.text_color.g * 0.9,
                b: self.text_color.b * 1.1,
                a: self.text_color.a,
            }
        } else if strikethrough {
            // Dimmed color for strikethrough text
            Color {
                r: self.text_color.r,
                g: self.text_color.g,
                b: self.text_color.b,
                a: self.text_color.a * 0.6,
            }
        } else {
            self.text_color
        };

        frame.fill_text(Text {
            content: text.to_string(),
            position: Point::new(x, y),
            color: text_color,
            size: size.into(),
            font,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: iced::widget::text::Shaping::Advanced,
            ..Default::default()
        });

        // Draw strikethrough line if needed
        if strikethrough {
            use iced::widget::canvas::{Path, Stroke};
            let text_width = text.len() as f32 * size * 0.6;
            let line_y = y + size * 0.5;
            let line = Path::line(
                Point::new(x, line_y),
                Point::new(x + text_width, line_y)
            );
            frame.stroke(
                &line,
                Stroke::default()
                    .with_color(text_color)
                    .with_width(1.0)
            );
        }
    }
}

