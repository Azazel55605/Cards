use iced::Color;

/// Represents a styled text segment
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub style: TextStyle,
    pub link_url: Option<String>,
}

/// Represents a checkbox item
#[derive(Debug, Clone)]
pub struct CheckboxItem {
    pub checked: bool,
    pub line_index: usize, // Index in the document to identify which checkbox this is
}

/// Text styling options
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub is_code: bool,
    pub is_link: bool,
    pub is_math: bool,
    pub color: Option<Color>, // None means use default
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::with_base_size(14.0)
    }
}

impl TextStyle {
    pub fn with_base_size(base_size: f32) -> Self {
        Self {
            size: base_size,
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            is_code: false,
            is_link: false,
            is_math: false,
            color: None,
        }
    }

    pub fn heading(level: u32) -> Self {
        Self::heading_with_base(level, 14.0)
    }

    pub fn heading_with_base(level: u32, base_size: f32) -> Self {
        let size_multiplier = match level {
            1 => 1.43,  // 20/14
            2 => 1.29,  // 18/14
            3 => 1.14,  // 16/14
            4 => 1.0,   // 14/14
            5 => 0.93,  // 13/14
            _ => 0.86,  // 12/14
        };
        Self {
            size: base_size * size_multiplier,
            bold: true,
            ..Self::with_base_size(base_size)
        }
    }

    pub fn bold() -> Self {
        Self {
            bold: true,
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }

    pub fn code() -> Self {
        Self {
            is_code: true,
            ..Default::default()
        }
    }

    pub fn math() -> Self {
        Self {
            is_math: true,
            italic: true,
            ..Default::default()
        }
    }

    pub fn strikethrough() -> Self {
        Self {
            strikethrough: true,
            ..Default::default()
        }
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn with_strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }

    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    pub fn with_link(mut self) -> Self {
        self.is_link = true;
        self.underline = true;
        self
    }
}

/// Represents a line of text segments
#[derive(Debug, Clone)]
pub struct TextLine {
    pub segments: Vec<TextSegment>,
    pub indent: f32,
    pub spacing_before: f32,
    pub spacing_after: f32,
    pub checkbox: Option<CheckboxItem>, // If this line has a checkbox
    pub is_rule: bool,       // Horizontal rule — rendered as a full-width line
    pub quote_depth: u8,     // 0 = not a blockquote; >0 = nesting depth
    pub is_math_block: bool, // Display math block ($$...$$)
    /// When `Some`, this line is a table row. Each inner Vec is one cell's segments.
    /// `is_table_header` marks the first row (bold, followed by a separator rule).
    pub table_cells: Option<Vec<Vec<TextSegment>>>,
    pub is_table_header: bool,
}

impl Default for TextLine {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            indent: 0.0,
            spacing_before: 0.0,
            spacing_after: 0.0,
            checkbox: None,
            is_rule: false,
            quote_depth: 0,
            is_math_block: false,
            table_cells: None,
            is_table_header: false,
        }
    }
}

impl TextLine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_spacing_before(mut self, spacing: f32) -> Self {
        self.spacing_before = spacing;
        self
    }

    pub fn with_spacing_after(mut self, spacing: f32) -> Self {
        self.spacing_after = spacing;
        self
    }

    pub fn add_segment(&mut self, text: String, style: TextStyle) {
        self.segments.push(TextSegment { text, style, link_url: None });
    }

    pub fn add_link_segment(&mut self, text: String, style: TextStyle, url: String) {
        self.segments.push(TextSegment { text, style, link_url: Some(url) });
    }

    pub fn with_checkbox(mut self, checked: bool, line_index: usize) -> Self {
        self.checkbox = Some(CheckboxItem { checked, line_index });
        self
    }

    pub fn as_rule(mut self) -> Self {
        self.is_rule = true;
        self
    }

    pub fn with_quote_depth(mut self, depth: u8) -> Self {
        self.quote_depth = depth;
        if depth > 0 && self.indent < (depth as f32 * 12.0) {
            self.indent = depth as f32 * 12.0;
        }
        self
    }

    pub fn as_math_block(mut self) -> Self {
        self.is_math_block = true;
        self
    }

    pub fn as_table_row(mut self, cells: Vec<Vec<TextSegment>>, is_header: bool) -> Self {
        self.table_cells = Some(cells);
        self.is_table_header = is_header;
        self
    }

    pub fn is_empty(&self) -> bool {
        !self.is_rule && !self.is_math_block && self.table_cells.is_none()
            && (self.segments.is_empty() || self.segments.iter().all(|s| s.text.trim().is_empty()))
    }
}

/// Document structure - collection of text lines
#[derive(Debug, Clone)]
pub struct TextDocument {
    pub lines: Vec<TextLine>,
}

impl Default for TextDocument {
    fn default() -> Self {
        Self { lines: Vec::new() }
    }
}

impl TextDocument {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line(&mut self, line: TextLine) {
        self.lines.push(line);
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

