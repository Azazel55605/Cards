use iced::widget::canvas::{Path, Frame, Stroke, Text};
use iced::{Color, Point, Rectangle};
use std::time::Instant;

// Monospace font metrics
const MONOSPACE_CHAR_WIDTH: f32 = 8.43; // Precise width for Iced's monospace at 14px
const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 21.0;

/// A simple custom text editor with visible cursor
#[derive(Debug, Clone)]
pub struct CustomTextEditor {
    pub text: String,
    pub cursor_position: usize, // Character position in string
    pub last_blink: Instant,
    pub selection_start: Option<usize>,
    pub scroll_offset: usize, // Line offset for scrolling
}

impl Default for CustomTextEditor {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            last_blink: Instant::now(),
            selection_start: None,
            scroll_offset: 0,
        }
    }
}

impl CustomTextEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: &str) -> Self {
        let cursor_position = text.len();
        Self {
            text: text.to_string(),
            cursor_position,
            last_blink: Instant::now(),
            selection_start: None,
            scroll_offset: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn insert_char(&mut self, c: char) {
        // Delete selection if any
        if let Some(sel_start) = self.selection_start {
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        }

        self.text.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
        self.last_blink = Instant::now();
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn backspace(&mut self) {
        if let Some(sel_start) = self.selection_start {
            // Delete selection
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        } else if self.cursor_position > 0 {
            // Find previous character boundary
            let mut pos = self.cursor_position - 1;
            while pos > 0 && !self.text.is_char_boundary(pos) {
                pos -= 1;
            }
            self.text.remove(pos);
            self.cursor_position = pos;
        }
        self.last_blink = Instant::now();
    }

    pub fn delete_word_backward(&mut self) {
        if let Some(sel_start) = self.selection_start {
            // Delete selection
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        } else if self.cursor_position > 0 {
            let before_cursor = &self.text[..self.cursor_position];

            // Skip trailing whitespace
            let mut pos = self.cursor_position;
            while pos > 0 {
                let prev_char = before_cursor.chars().nth_back((self.cursor_position - pos) as usize);
                if let Some(ch) = prev_char {
                    if !ch.is_whitespace() {
                        break;
                    }
                }
                pos -= 1;
            }

            // Delete word characters
            while pos > 0 {
                if let Some(ch) = before_cursor.chars().nth_back((self.cursor_position - pos) as usize) {
                    if ch.is_whitespace() {
                        break;
                    }
                }
                pos -= 1;
            }

            self.text.drain(pos..self.cursor_position);
            self.cursor_position = pos;
        }
        self.last_blink = Instant::now();
    }

    pub fn delete_word_forward(&mut self) {
        if let Some(sel_start) = self.selection_start {
            // Delete selection
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        } else if self.cursor_position < self.text.len() {
            let after_cursor = &self.text[self.cursor_position..];
            let mut chars_to_delete = 0;
            let mut found_word = false;

            // Skip leading whitespace
            for ch in after_cursor.chars() {
                if !ch.is_whitespace() {
                    found_word = true;
                    break;
                }
                chars_to_delete += ch.len_utf8();
            }

            // Delete word characters
            if found_word {
                for ch in after_cursor[chars_to_delete..].chars() {
                    if ch.is_whitespace() {
                        break;
                    }
                    chars_to_delete += ch.len_utf8();
                }
            }

            for _ in 0..chars_to_delete {
                if self.cursor_position < self.text.len() {
                    self.text.remove(self.cursor_position);
                }
            }
        }
        self.last_blink = Instant::now();
    }

    pub fn delete(&mut self) {
        if let Some(sel_start) = self.selection_start {
            // Delete selection
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        } else if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_left(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.selection_start = None;
        }

        if self.cursor_position > 0 {
            let mut pos = self.cursor_position - 1;
            while pos > 0 && !self.text.is_char_boundary(pos) {
                pos -= 1;
            }
            self.cursor_position = pos;
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_right(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.selection_start = None;
        }

        if self.cursor_position < self.text.len() {
            let mut pos = self.cursor_position + 1;
            while pos < self.text.len() && !self.text.is_char_boundary(pos) {
                pos += 1;
            }
            self.cursor_position = pos;
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_up(&mut self) {
        // Find current line start
        let before_cursor = &self.text[..self.cursor_position];
        if let Some(prev_newline) = before_cursor.rfind('\n') {
            let line_start = prev_newline + 1;
            let col = self.cursor_position - line_start;
            
            // Find previous line start
            let before_line = &self.text[..prev_newline];
            if let Some(prev_prev_newline) = before_line.rfind('\n') {
                let prev_line_start = prev_prev_newline + 1;
                let prev_line_len = prev_newline - prev_line_start;
                self.cursor_position = prev_line_start + col.min(prev_line_len);
            } else {
                // First line, go to start
                self.cursor_position = col.min(prev_newline);
            }
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_down(&mut self) {
        // Find current line
        let before_cursor = &self.text[..self.cursor_position];
        let line_start = before_cursor.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let col = self.cursor_position - line_start;
        
        // Find next line
        let after_cursor = &self.text[self.cursor_position..];
        if let Some(next_newline) = after_cursor.find('\n') {
            let next_line_start = self.cursor_position + next_newline + 1;
            let rest = &self.text[next_line_start..];
            let next_line_end = rest.find('\n').map(|p| next_line_start + p).unwrap_or(self.text.len());
            let next_line_len = next_line_end - next_line_start;
            self.cursor_position = next_line_start + col.min(next_line_len);
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        self.selection_start = None;
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
        self.selection_start = None;
        self.last_blink = Instant::now();
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor_position = self.text.len();
        self.last_blink = Instant::now();
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let Some(sel_start) = self.selection_start {
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            if start < end {
                return Some(self.text[start..end].to_string());
            }
        }
        None
    }

    pub fn delete_selection(&mut self) {
        if let Some(sel_start) = self.selection_start {
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
            self.last_blink = Instant::now();
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        // Delete selection if any
        if self.selection_start.is_some() {
            self.delete_selection();
        }

        // Insert the text
        self.text.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        self.last_blink = Instant::now();
    }

    /// Render the text editor content with cursor
    pub fn render(
        &self,
        frame: &mut Frame,
        bounds: Rectangle,
        text_color: Color,
        cursor_color: Color,
        selection_color: Color,
    ) {
        let padding = 10.0;
        let text_x = bounds.x + padding;
        let text_y = bounds.y + padding;
        let max_width = bounds.width - (padding * 2.0);
        let max_height = bounds.height - (padding * 2.0);

        // Calculate how many lines can fit
        let max_lines = ((max_height / LINE_HEIGHT).floor() as usize).max(1);

        // Wrap text and build position map
        let (wrapped_lines, position_map) = self.wrap_and_map_positions(&self.text, max_width);

        // Find cursor position using the map
        let (cursor_line_idx, cursor_col) = position_map
            .iter()
            .find(|(_, _, start, end)| self.cursor_position >= *start && self.cursor_position <= *end)
            .map(|(line, col, start, _)| {
                // Get the text from line start to cursor position
                let text_before_cursor = if self.cursor_position <= self.text.len() && *start <= self.text.len() {
                    &self.text[*start..self.cursor_position]
                } else {
                    ""
                };
                // Count actual characters, not bytes
                let char_count = text_before_cursor.chars().count();
                (*line, *col + char_count)
            })
            .unwrap_or_else(|| {
                // Fallback: cursor at end
                let last_line_idx = wrapped_lines.len().saturating_sub(1);
                let last_line_len = wrapped_lines.last().map(|l| l.chars().count()).unwrap_or(0);
                (last_line_idx, last_line_len)
            });

        // Calculate scroll offset to keep cursor visible
        let mut scroll_offset = self.scroll_offset;
        if cursor_line_idx < scroll_offset {
            scroll_offset = cursor_line_idx;
        } else if cursor_line_idx >= scroll_offset + max_lines {
            scroll_offset = cursor_line_idx - max_lines + 1;
        }

        let max_scroll = wrapped_lines.len().saturating_sub(max_lines);
        scroll_offset = scroll_offset.min(max_scroll);

        let visible_start = scroll_offset;
        let visible_end = (scroll_offset + max_lines).min(wrapped_lines.len());

        // Draw selection background if any
        if let Some(sel_start) = self.selection_start {
            let sel_begin = sel_start.min(self.cursor_position);
            let sel_end = sel_start.max(self.cursor_position);

            // Find which lines are selected
            for (line_idx, line) in wrapped_lines.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
                let (_, _, line_start, line_end) = position_map[line_idx];

                // Check if this line contains selection
                if sel_end >= line_start && sel_begin <= line_end {
                    let display_idx = line_idx - visible_start;
                    let y = text_y + (display_idx as f32 * LINE_HEIGHT);

                    // Calculate selection range within this line using character counts
                    let line_text_start = if line_start <= self.text.len() { line_start } else { self.text.len() };
                    let line_text_end = if line_end <= self.text.len() { line_end } else { self.text.len() };

                    let sel_start_byte = sel_begin.max(line_text_start);
                    let sel_end_byte = sel_end.min(line_text_end);

                    // Count characters from line start to selection start
                    let chars_before_sel = if sel_start_byte > line_text_start && line_text_start < self.text.len() && sel_start_byte <= self.text.len() {
                        self.text[line_text_start..sel_start_byte].chars().count()
                    } else {
                        0
                    };

                    // Count characters in selection
                    let chars_in_sel = if sel_start_byte < sel_end_byte && sel_start_byte < self.text.len() && sel_end_byte <= self.text.len() {
                        self.text[sel_start_byte..sel_end_byte].chars().count()
                    } else {
                        0
                    };

                    let sel_x_start = text_x + (chars_before_sel as f32 * MONOSPACE_CHAR_WIDTH);
                    let sel_x_end = text_x + ((chars_before_sel + chars_in_sel) as f32 * MONOSPACE_CHAR_WIDTH);

                    // Draw selection rectangle
                    use iced::widget::canvas::{path::Builder, Fill};
                    let mut path_builder = Builder::new();
                    path_builder.rectangle(
                        Point::new(sel_x_start, y),
                        iced::Size::new(sel_x_end - sel_x_start, LINE_HEIGHT)
                    );
                    let selection_path = path_builder.build();

                    frame.fill(&selection_path, Fill {
                        style: iced::widget::canvas::fill::Style::Solid(selection_color),
                        rule: iced::widget::canvas::fill::Rule::NonZero,
                    });
                }
            }
        }

        // Draw text line by line (only visible lines)
        for (line_idx, line) in wrapped_lines.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
            let display_idx = line_idx - visible_start;
            let y = text_y + (display_idx as f32 * LINE_HEIGHT);

            frame.fill_text(Text {
                content: line.clone(),
                position: Point::new(text_x, y),
                color: text_color,
                size: FONT_SIZE.into(),
                font: iced::Font::MONOSPACE,
                shaping: iced::widget::text::Shaping::Advanced,
                ..Default::default()
            });
        }

        // Draw cursor (blinking) - only if it's visible within bounds
        if cursor_line_idx >= visible_start && cursor_line_idx < visible_end {
            let elapsed = self.last_blink.elapsed().as_millis();
            // Show cursor always for first 100ms after action, then blink
            let should_show_cursor = elapsed < 100 || (elapsed % 1000) < 500;

            if should_show_cursor {
                let display_line = cursor_line_idx - visible_start;
                let cursor_x = text_x + (cursor_col as f32 * MONOSPACE_CHAR_WIDTH);
                let cursor_y = text_y + (display_line as f32 * LINE_HEIGHT);

                let cursor_path = Path::line(
                    Point::new(cursor_x, cursor_y),
                    Point::new(cursor_x, cursor_y + FONT_SIZE),
                );

                frame.stroke(
                    &cursor_path,
                    Stroke::default()
                        .with_color(cursor_color)
                        .with_width(2.0),
                );
            }
        }
    }

    /// Update scroll offset - call this from update loop
    pub fn update_scroll(&mut self, bounds: Rectangle) {
        let max_height = bounds.height - 20.0;
        let max_lines = ((max_height / LINE_HEIGHT).floor() as usize).max(1);
        let max_width = bounds.width - 20.0;

        let (wrapped_lines, position_map) = self.wrap_and_map_positions(&self.text, max_width);

        let (cursor_line_idx, _) = position_map
            .iter()
            .find(|(_, _, start, end)| self.cursor_position >= *start && self.cursor_position <= *end)
            .map(|(line, col, start, _)| {
                let offset = self.cursor_position - start;
                (*line, *col + offset)
            })
            .unwrap_or((wrapped_lines.len().saturating_sub(1), 0));

        // Auto-scroll
        if cursor_line_idx < self.scroll_offset {
            self.scroll_offset = cursor_line_idx;
        } else if cursor_line_idx >= self.scroll_offset + max_lines {
            self.scroll_offset = cursor_line_idx - max_lines + 1;
        }

        let max_scroll = wrapped_lines.len().saturating_sub(max_lines);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    /// Wrap text and build a position map: (line_idx, col_in_line, original_start_pos, original_end_pos)
    fn wrap_and_map_positions(&self, text: &str, max_width: f32) -> (Vec<String>, Vec<(usize, usize, usize, usize)>) {
        let max_chars = ((max_width / MONOSPACE_CHAR_WIDTH).floor() as usize).max(1);

        let mut wrapped_lines = Vec::new();
        let mut position_map = Vec::new();
        let mut original_pos = 0;

        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                position_map.push((wrapped_lines.len(), 0, original_pos, original_pos));
                wrapped_lines.push(String::new());
                original_pos += 1;
                continue;
            }

            // Process character by character to preserve ALL spaces
            let mut line = String::new();
            let mut line_start_pos = original_pos;
            
            for ch in paragraph.chars() {
                // Check if adding this character would exceed max width
                if line.chars().count() >= max_chars {
                    // Flush current line
                    let line_end_pos = line_start_pos + line.len();
                    position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                    wrapped_lines.push(line.clone());
                    
                    // Start new line
                    line.clear();
                    line_start_pos = line_end_pos;
                }
                
                // Add the character (including spaces!)
                line.push(ch);
            }

            // Flush remaining line
            if !line.is_empty() {
                let line_end_pos = line_start_pos + line.len();
                position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                wrapped_lines.push(line);
            }

            original_pos += paragraph.len() + 1; // +1 for \n
        }

        if wrapped_lines.is_empty() {
            wrapped_lines.push(String::new());
            position_map.push((0, 0, 0, 0));
        }

        (wrapped_lines, position_map)
    }
}
