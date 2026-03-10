use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, Options, CodeBlockKind};
use crate::text_document::{TextDocument, TextLine, TextStyle};
use iced::Color;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

/// Markdown parser that converts markdown to a TextDocument
pub struct MarkdownParser {
    options: Options,
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
    pub base_font_size: f32,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self::with_font_size(14.0)
    }

    pub fn with_font_size(base_font_size: f32) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        // Load syntax definitions and theme
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        // Use a theme that works well with both light and dark backgrounds
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self {
            options,
            syntax_set,
            theme,
            base_font_size,
        }
    }

    pub fn set_base_font_size(&mut self, size: f32) {
        self.base_font_size = size;
    }

    /// Parse markdown text into a TextDocument
    pub fn parse(&self, markdown: &str) -> TextDocument {
        let parser = Parser::new_ext(markdown, self.options);
        let mut document = TextDocument::new();
        let mut current_line = TextLine::new();
        let mut text_buffer = String::new();
        let mut current_style = TextStyle::with_base_size(self.base_font_size);
        let mut in_list = false;
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_content = String::new();
        let mut checkbox_counter = 0;
        let mut in_link = false;
        let mut current_link_url: Option<String> = None;

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
                            current_style = TextStyle::heading_with_base(level_num, self.base_font_size);
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
                        Tag::Link(_, url, _) => {
                            self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                            in_link = true;
                            current_link_url = Some(url.to_string());
                            current_style = current_style.with_link();
                        }
                        Tag::List(_) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            in_list = true;
                            current_line = TextLine::new().with_spacing_before(4.0);
                        }
                        Tag::Item => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);

                            // Create new line with indent - checkbox will be added by TaskListMarker event if present
                            // Use larger indent to add spacing from left edge
                            current_line = TextLine::new().with_indent(30.0);
                            // Don't add bullet yet - wait to see if TaskListMarker comes
                        }
                        Tag::CodeBlock(kind) => {
                            self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                            in_code_block = true;

                            // Extract language from code block kind
                            code_block_lang = match kind {
                                CodeBlockKind::Fenced(lang) => {
                                    if lang.is_empty() {
                                        None
                                    } else {
                                        Some(lang.to_string())
                                    }
                                }
                                CodeBlockKind::Indented => None,
                            };

                            code_block_content.clear();
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
                            current_style = TextStyle::with_base_size(self.base_font_size);
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
                        Tag::Link(_, _, _) => {
                            // Flush link text with its URL
                            if !text_buffer.is_empty() {
                                if let Some(url) = &current_link_url {
                                    current_line.add_link_segment(text_buffer.clone(), current_style, url.clone());
                                } else {
                                    current_line.add_segment(text_buffer.clone(), current_style);
                                }
                                text_buffer.clear();
                            }
                            in_link = false;
                            current_link_url = None;
                            current_style = TextStyle {
                                is_link: false,
                                underline: false,
                                ..current_style
                            };
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
                            // Process the collected code block content with syntax highlighting
                            if !code_block_content.is_empty() {
                                self.add_highlighted_code_block(
                                    &mut document,
                                    &code_block_content,
                                    code_block_lang.as_deref(),
                                );
                            }

                            in_code_block = false;
                            code_block_lang = None;
                            code_block_content.clear();
                            current_style = TextStyle::with_base_size(self.base_font_size);
                            current_line = TextLine::new().with_spacing_after(4.0);
                        }
                        _ => {}
                    }
                }
                Event::TaskListMarker(checked) => {
                    // TaskListMarker comes AFTER Tag::Item starts
                    // Add checkbox to the current line
                    current_line = current_line.with_checkbox(checked, checkbox_counter);
                    checkbox_counter += 1;
                }
                Event::Text(text) => {
                    if in_code_block {
                        // Collect code block content for later highlighting
                        code_block_content.push_str(&text);
                    } else {
                        // If we're in a list item and haven't added a bullet or checkbox, add bullet now
                        if in_list && current_line.checkbox.is_none() && !text_buffer.contains("• ") && current_line.segments.is_empty() {
                            text_buffer.push_str("• ");
                        }
                        text_buffer.push_str(&text);
                    }
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
                Event::Rule => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style);
                    // is_rule=true — the renderer will draw a full-width line
                    let rule_line = TextLine::new()
                        .with_spacing_before(8.0)
                        .with_spacing_after(8.0)
                        .as_rule();
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

    /// Apply syntax highlighting to a code block
    fn add_highlighted_code_block(
        &self,
        document: &mut TextDocument,
        code: &str,
        language: Option<&str>,
    ) {
        // Try to find the syntax definition for the language
        let syntax = if let Some(lang) = language {
            self.syntax_set
                .find_syntax_by_token(lang)
                .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
        } else {
            None
        };

        if let Some(syntax) = syntax {
            // Syntax highlighting available
            let mut highlighter = HighlightLines::new(syntax, &self.theme);

            for line in LinesWithEndings::from(code) {
                let mut text_line = TextLine::new()
                    .with_indent(10.0)
                    .with_spacing_before(2.0);

                // Highlight the line
                if let Ok(ranges) = highlighter.highlight_line(line, &self.syntax_set) {
                    for (style, text) in ranges {
                        let color = Color::from_rgb8(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        );

                        let text_style = TextStyle {
                            is_code: true,
                            color: Some(color),
                            ..Default::default()
                        };

                        // Remove trailing newline from text for cleaner rendering
                        let text = text.trim_end_matches('\n').trim_end_matches('\r');
                        if !text.is_empty() {
                            text_line.add_segment(text.to_string(), text_style);
                        }
                    }
                }

                // Add the line even if empty (to preserve code structure)
                document.add_line(text_line);
            }
        } else {
            // No syntax highlighting available, render as plain code
            for line in code.lines() {
                let mut text_line = TextLine::new()
                    .with_indent(10.0)
                    .with_spacing_before(2.0);

                let code_style = TextStyle::code();
                text_line.add_segment(line.to_string(), code_style);
                document.add_line(text_line);
            }
        }

        // Add spacing after code block
        document.add_line(TextLine::new().with_spacing_after(4.0));
    }
}

