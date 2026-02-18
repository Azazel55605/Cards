use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, Options};
use crate::text_document::{TextDocument, TextLine, TextStyle};

/// Markdown parser that converts markdown to a TextDocument
pub struct MarkdownParser {
    options: Options,
}

impl MarkdownParser {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        Self { options }
    }

    /// Parse markdown text into a TextDocument
    pub fn parse(&self, markdown: &str) -> TextDocument {
        let parser = Parser::new_ext(markdown, self.options);
        let mut document = TextDocument::new();
        let mut current_line = TextLine::new();
        let mut text_buffer = String::new();
        let mut current_style = TextStyle::default();
        let mut in_list = false;
        let mut in_code_block = false;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            if !document.lines.is_empty() {
                                current_line = current_line.with_spacing_before(4.0);
                            }
                        }
                        Tag::Heading(level, _, _) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);

                            let level_num = match level {
                                HeadingLevel::H1 => 1,
                                HeadingLevel::H2 => 2,
                                HeadingLevel::H3 => 3,
                                HeadingLevel::H4 => 4,
                                HeadingLevel::H5 => 5,
                                HeadingLevel::H6 => 6,
                            };
                            current_style = TextStyle::heading(level_num);
                            current_line = TextLine::new().with_spacing_before(8.0).with_spacing_after(4.0);
                        }
                        Tag::Strong => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_bold(true);
                        }
                        Tag::Emphasis => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_italic(true);
                        }
                        Tag::Strikethrough => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_strikethrough(true);
                        }
                        Tag::List(_) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            in_list = true;
                            current_line = TextLine::new().with_spacing_before(4.0);
                        }
                        Tag::Item => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            current_line = TextLine::new().with_indent(10.0);
                            text_buffer.push_str("• ");
                        }
                        Tag::CodeBlock(_) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            in_code_block = true;
                            current_style = TextStyle::code();
                            current_line = TextLine::new().with_indent(10.0).with_spacing_before(4.0);
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            current_line = TextLine::new().with_spacing_after(4.0);
                        }
                        Tag::Heading(_, _, _) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            current_style = TextStyle::default();
                            current_line = TextLine::new();
                        }
                        Tag::Strong => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_bold(false);
                        }
                        Tag::Emphasis => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_italic(false);
                        }
                        Tag::Strikethrough => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            current_style = current_style.with_strikethrough(false);
                        }
                        Tag::Item => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            current_line = TextLine::new();
                        }
                        Tag::List(_) => {
                            in_list = false;
                            current_line = TextLine::new().with_spacing_after(4.0);
                        }
                        Tag::CodeBlock(_) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            in_code_block = false;
                            current_style = TextStyle::default();
                            current_line = TextLine::new().with_spacing_after(4.0);
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    text_buffer.push_str(&text);
                }
                Event::Code(code) => {
                    self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                    let code_style = TextStyle::code();
                    current_line.add_segment(format!("`{}`", code), code_style);
                }
                Event::SoftBreak => {
                    text_buffer.push(' ');
                }
                Event::HardBreak => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                    current_line = TextLine::new();
                }
                Event::TaskListMarker(checked) => {
                    text_buffer.insert_str(0, if checked { "[x] " } else { "[ ] " });
                }
                Event::Rule => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                    // Add a horizontal rule line (represented as special text for now)
                    let mut rule_line = TextLine::new().with_spacing_before(8.0).with_spacing_after(8.0);
                    rule_line.add_segment("─".repeat(50), TextStyle::default());
                    document.add_line(rule_line);
                    current_line = TextLine::new();
                }
                _ => {}
            }
        }

        // Flush remaining content
        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);

        document
    }

    fn flush_text_to_line(&self, line: &mut TextLine, text_buffer: &mut String, style: &TextStyle) {
        if !text_buffer.is_empty() {
            line.add_segment(text_buffer.clone(), *style);
            text_buffer.clear();
        }
    }

    fn flush_current_line(
        &self,
        document: &mut TextDocument,
        line: &mut TextLine,
        text_buffer: &mut String,
        style: &TextStyle,
    ) {
        self.flush_text_to_line(line, text_buffer, style);
        if !line.is_empty() {
            document.add_line(line.clone());
            *line = TextLine::new();
        }
    }
}

