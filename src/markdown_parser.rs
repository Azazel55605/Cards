use pulldown_cmark::{
    Event, HeadingLevel, Options, Parser, Tag, TagEnd, CodeBlockKind,
};
use crate::text_document::{TextDocument, TextLine, TextSegment, TextStyle};
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
        options.insert(Options::ENABLE_MATH);

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self { options, syntax_set, theme, base_font_size }
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

        // list_stack: each entry is Some(start) for ordered, None for unordered
        let mut list_stack: Vec<Option<u64>> = Vec::new();
        // ordered counter per level (parallel to list_stack)
        let mut ordered_counters: Vec<u64> = Vec::new();

        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_content = String::new();
        let mut checkbox_counter = 0;
        let mut current_link_url: Option<String> = None;
        let mut quote_depth: u8 = 0;

        // Table state
        let mut in_table = false;
        let mut in_table_head = false;
        // Cells accumulated for the current row: each cell is a Vec<TextSegment>
        let mut table_row_cells: Vec<Vec<TextSegment>> = Vec::new();
        // Buffer for the current cell
        let mut table_cell_segments: Vec<TextSegment> = Vec::new();
        let mut table_cell_text = String::new();

        // Display math block
        let mut in_display_math = false;
        let mut math_buffer = String::new();

        for event in parser {
            match event {
                // ── Start tags ──────────────────────────────────────────────
                Event::Start(tag) => match tag {
                    Tag::Paragraph => {
                        if !document.lines.is_empty() {
                            current_line = current_line.with_spacing_before(4.0);
                        }
                    }
                    Tag::Heading { level, .. } => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        let level_num = heading_level_num(level);
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
                    Tag::Link { dest_url, .. } => {
                        self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                        current_link_url = Some(dest_url.to_string());
                        current_style = current_style.with_link();
                    }
                    Tag::List(start) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        list_stack.push(start);
                        ordered_counters.push(start.unwrap_or(0));
                        if list_stack.len() == 1 {
                            current_line = current_line.with_spacing_before(4.0);
                        }
                    }
                    Tag::Item => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        let depth = list_stack.len();
                        let indent = depth as f32 * 16.0;
                        current_line = TextLine::new().with_indent(indent);
                        // Determine bullet / number; will add it in Event::Text if no TaskListMarker
                        if let Some(Some(_)) = list_stack.last() {
                            // ordered — counter will be read when text arrives
                        }
                        // bump the counter for the current level
                        if let Some(c) = ordered_counters.last_mut() { *c += 1; }
                    }
                    Tag::CodeBlock(kind) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        in_code_block = true;
                        code_block_lang = match kind {
                            CodeBlockKind::Fenced(lang) => {
                                // "math" fenced block = display math
                                if lang.trim() == "math" {
                                    in_display_math = true;
                                    math_buffer.clear();
                                    None
                                } else if lang.is_empty() {
                                    None
                                } else {
                                    Some(lang.to_string())
                                }
                            }
                            CodeBlockKind::Indented => None,
                        };
                        code_block_content.clear();
                    }
                    Tag::BlockQuote(_kind) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        quote_depth = quote_depth.saturating_add(1);
                        current_line = TextLine::new().with_spacing_before(4.0);
                    }
                    Tag::Table(_) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        in_table = true;
                        table_row_cells.clear();
                        table_cell_segments.clear();
                        table_cell_text.clear();
                    }
                    Tag::TableHead => {
                        in_table_head = true;
                        table_row_cells.clear();
                    }
                    Tag::TableRow => {
                        table_row_cells.clear();
                        table_cell_segments.clear();
                        table_cell_text.clear();
                    }
                    Tag::TableCell => {
                        // Start of a new cell — reset cell buffers
                        table_cell_segments.clear();
                        table_cell_text.clear();
                        if in_table_head {
                            current_style = current_style.with_bold(true);
                        }
                    }
                    _ => {}
                },

                // ── End tags ─────────────────────────────────────────────────
                Event::End(tag) => match tag {
                    TagEnd::Paragraph => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        current_line = TextLine::new().with_spacing_after(4.0);
                    }
                    TagEnd::Heading(_) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        current_style = TextStyle::with_base_size(self.base_font_size);
                        current_line = TextLine::new();
                    }
                    TagEnd::Strong => {
                        self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                        current_style = current_style.with_bold(false);
                    }
                    TagEnd::Emphasis => {
                        self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                        current_style = current_style.with_italic(false);
                    }
                    TagEnd::Strikethrough => {
                        self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                        current_style = current_style.with_strikethrough(false);
                    }
                    TagEnd::Link => {
                        if !text_buffer.is_empty() {
                            if let Some(url) = &current_link_url {
                                current_line.add_link_segment(text_buffer.clone(), current_style, url.clone());
                            } else {
                                current_line.add_segment(text_buffer.clone(), current_style);
                            }
                            text_buffer.clear();
                        }
                        current_link_url = None;
                        current_style = TextStyle { is_link: false, underline: false, ..current_style };
                    }
                    TagEnd::Item => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        current_line = TextLine::new();
                    }
                    TagEnd::List(_) => {
                        list_stack.pop();
                        ordered_counters.pop();
                        current_line = TextLine::new().with_spacing_after(4.0);
                    }
                    TagEnd::CodeBlock => {
                        if in_display_math {
                            // Render display math as styled lines with background marker
                            for math_line in math_buffer.lines() {
                                let mut tl = TextLine::new()
                                    .with_indent(8.0)
                                    .with_spacing_before(1.0)
                                    .as_math_block()
                                    .with_quote_depth(quote_depth);
                                let mut ms = TextStyle::math();
                                ms.size = self.base_font_size;
                                tl.add_segment(math_line.to_string(), ms);
                                document.add_line(tl);
                            }
                            in_display_math = false;
                            math_buffer.clear();
                        } else if !code_block_content.is_empty() {
                            self.add_highlighted_code_block(&mut document, &code_block_content, code_block_lang.as_deref());
                        }
                        in_code_block = false;
                        code_block_lang = None;
                        code_block_content.clear();
                        current_style = TextStyle::with_base_size(self.base_font_size);
                        current_line = TextLine::new().with_spacing_after(4.0);
                    }
                    TagEnd::BlockQuote(_) => {
                        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                        quote_depth = quote_depth.saturating_sub(1);
                        current_line = TextLine::new().with_spacing_after(4.0);
                    }
                    TagEnd::Table => {
                        in_table = false;
                        table_row_cells.clear();
                        table_cell_segments.clear();
                        table_cell_text.clear();
                        document.add_line(TextLine::new().with_spacing_after(4.0));
                    }
                    TagEnd::TableHead => {
                        in_table_head = false;
                        current_style = current_style.with_bold(false);
                    }
                    TagEnd::TableRow => {
                        // Flush any remaining cell text
                        if !table_cell_text.is_empty() {
                            table_cell_segments.push(TextSegment {
                                text: table_cell_text.clone(),
                                style: current_style,
                                link_url: None,
                            });
                            table_cell_text.clear();
                        }
                        if !table_cell_segments.is_empty() {
                            table_row_cells.push(table_cell_segments.clone());
                            table_cell_segments.clear();
                        }
                        // Emit the table row as a TextLine with table_cells
                        if !table_row_cells.is_empty() {
                            // Header row gets paragraph-style spacing before; data rows are tight
                            let spacing_before = if in_table_head { 4.0 } else { 1.0 };
                            let row_line = TextLine::new()
                                .with_spacing_before(spacing_before)
                                .as_table_row(table_row_cells.clone(), in_table_head);
                            document.add_line(row_line);
                            // After the header row, add a separator rule
                            if in_table_head {
                                document.add_line(
                                    TextLine::new().with_spacing_before(1.0).with_spacing_after(1.0).as_rule()
                                );
                            }
                        }
                        table_row_cells.clear();
                    }
                    TagEnd::TableCell => {
                        // Flush the current cell text/segments into table_cell_segments
                        if !table_cell_text.is_empty() {
                            table_cell_segments.push(TextSegment {
                                text: table_cell_text.clone(),
                                style: current_style,
                                link_url: None,
                            });
                            table_cell_text.clear();
                        }
                        // Move completed cell into row
                        table_row_cells.push(table_cell_segments.clone());
                        table_cell_segments.clear();
                        // Restore non-bold style after header cell
                        if in_table_head {
                            current_style = current_style.with_bold(false);
                        }
                    }
                    _ => {}
                },

                // ── Leaf events ───────────────────────────────────────────────
                Event::TaskListMarker(checked) => {
                    current_line = current_line.with_checkbox(checked, checkbox_counter);
                    checkbox_counter += 1;
                }
                Event::Text(text) => {
                    if in_display_math {
                        math_buffer.push_str(&text);
                    } else if in_code_block {
                        code_block_content.push_str(&text);
                    } else if in_table {
                        // Table cell text goes directly into the cell buffer
                        table_cell_text.push_str(&text);
                    } else {
                        // List bullet / number prefix when no checkbox
                        if !list_stack.is_empty() && current_line.checkbox.is_none()
                            && current_line.segments.is_empty() && text_buffer.is_empty()
                        {
                            let depth = list_stack.len();
                            let is_ordered = list_stack.last().map(|s| s.is_some()).unwrap_or(false);
                            if is_ordered {
                                let n = *ordered_counters.last().unwrap_or(&1);
                                text_buffer.push_str(&format!("{}. ", n));
                            } else {
                                // Alternate bullet style by nesting depth
                                let bullet = match depth % 3 { 1 => "•", 2 => "◦", _ => "▸" };
                                text_buffer.push_str(&format!("{} ", bullet));
                            }
                        }
                        text_buffer.push_str(&text);
                    }
                }
                Event::Code(code) => {
                    if in_table {
                        // Flush pending text to cell, then add code segment
                        if !table_cell_text.is_empty() {
                            table_cell_segments.push(TextSegment {
                                text: table_cell_text.clone(),
                                style: current_style,
                                link_url: None,
                            });
                            table_cell_text.clear();
                        }
                        table_cell_segments.push(TextSegment {
                            text: code.to_string(),
                            style: TextStyle::code(),
                            link_url: None,
                        });
                    } else {
                        self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                        current_line.add_segment(code.to_string(), TextStyle::code());
                    }
                }
                // Inline math: $...$
                Event::InlineMath(math) => {
                    self.flush_text_to_line(&mut current_line, &mut text_buffer, &current_style);
                    let mut ms = TextStyle::math();
                    ms.size = self.base_font_size;
                    current_line.add_segment(math.to_string(), ms);
                }
                // Display math: $$...$$
                Event::DisplayMath(math) => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                    for math_line in math.lines() {
                        let mut tl = TextLine::new()
                            .with_indent(8.0)
                            .with_spacing_before(1.0)
                            .as_math_block()
                            .with_quote_depth(quote_depth);
                        let mut ms = TextStyle::math();
                        ms.size = self.base_font_size;
                        tl.add_segment(math_line.to_string(), ms);
                        document.add_line(tl);
                    }
                    current_line = TextLine::new().with_spacing_after(4.0);
                }
                Event::SoftBreak => {
                    if in_table { /* skip */ } else { text_buffer.push(' '); }
                }
                Event::HardBreak => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                    current_line = TextLine::new();
                }
                Event::Rule => {
                    self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
                    document.add_line(
                        TextLine::new().with_spacing_before(8.0).with_spacing_after(8.0).as_rule()
                    );
                    current_line = TextLine::new();
                }
                _ => {}
            }
        }

        self.flush_current_line(&mut document, &mut current_line, &mut text_buffer, &current_style, quote_depth);
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
        quote_depth: u8,
    ) {
        self.flush_text_to_line(line, text_buffer, style);
        if !line.is_empty() {
            if quote_depth > 0 && line.quote_depth == 0 {
                line.quote_depth = quote_depth;
                if line.indent < quote_depth as f32 * 12.0 {
                    line.indent = quote_depth as f32 * 12.0;
                }
            }
            document.add_line(line.clone());
            *line = TextLine::new();
        }
    }

    fn add_highlighted_code_block(&self, document: &mut TextDocument, code: &str, language: Option<&str>) {
        let syntax = language.and_then(|lang| {
            self.syntax_set.find_syntax_by_token(lang)
                .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
        });

        if let Some(syntax) = syntax {
            let mut highlighter = HighlightLines::new(syntax, &self.theme);
            for line in LinesWithEndings::from(code) {
                let mut text_line = TextLine::new().with_indent(10.0).with_spacing_before(2.0);
                if let Ok(ranges) = highlighter.highlight_line(line, &self.syntax_set) {
                    for (style, text) in ranges {
                        let color = Color::from_rgb8(style.foreground.r, style.foreground.g, style.foreground.b);
                        let text_style = TextStyle { is_code: true, color: Some(color), ..Default::default() };
                        let text = text.trim_end_matches('\n').trim_end_matches('\r');
                        if !text.is_empty() { text_line.add_segment(text.to_string(), text_style); }
                    }
                }
                document.add_line(text_line);
            }
        } else {
            for line in code.lines() {
                let mut text_line = TextLine::new().with_indent(10.0).with_spacing_before(2.0);
                text_line.add_segment(line.to_string(), TextStyle::code());
                document.add_line(text_line);
            }
        }

        document.add_line(TextLine::new().with_spacing_after(4.0));
    }
}

fn heading_level_num(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1, HeadingLevel::H2 => 2, HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4, HeadingLevel::H5 => 5, HeadingLevel::H6 => 6,
    }
}
