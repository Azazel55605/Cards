use iced::widget::canvas::{Path, Frame, Stroke, Text};
use iced::{Color, Point, Rectangle};
use std::time::Instant;

// Monospace font metrics
const MONOSPACE_CHAR_WIDTH: f32 = 8.4; // Fixed width for monospace at 14px
const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 21.0;

/// A simple custom text editor with visible cursor
#[derive(Debug, Clone)]
pub struct CustomTextEditor {
    pub text: String,
    pub cursor_position: usize, // Character position in string
    pub last_blink: Instant,
    pub selection_start: Option<usize>,
}

impl Default for CustomTextEditor {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            last_blink: Instant::now(),
            selection_start: None,
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

    /// Render the text editor content with cursor
    pub fn render(
        &self,
        frame: &mut Frame,
        bounds: Rectangle,
        text_color: Color,
        cursor_color: Color,
        _selection_color: Color,
    ) {
        let padding = 10.0;
        let text_x = bounds.x + padding;
        let text_y = bounds.y + padding;
        let max_width = bounds.width - (padding * 2.0);

        // Wrap text and build position map
        let (wrapped_lines, position_map) = self.wrap_and_map_positions(&self.text, max_width);

        // Find cursor position using the map
        let (cursor_line_idx, cursor_col) = position_map
            .iter()
            .find(|(_, _, start, end)| self.cursor_position >= *start && self.cursor_position <= *end)
            .map(|(line, col, start, _)| {
                let offset = self.cursor_position - start;
                (*line, *col + offset)
            })
            .unwrap_or((wrapped_lines.len().saturating_sub(1),
                       wrapped_lines.last().map(|l| l.len()).unwrap_or(0)));

        // Draw text line by line
        for (line_idx, line) in wrapped_lines.iter().enumerate() {
            let y = text_y + (line_idx as f32 * LINE_HEIGHT);

            frame.fill_text(Text {
                content: line.clone(),
                position: Point::new(text_x, y),
                color: text_color,
                size: FONT_SIZE.into(),
                font: iced::Font::MONOSPACE,
                ..Default::default()
            });
        }

        // Draw cursor (blinking)
        let elapsed = self.last_blink.elapsed().as_millis();
        let should_show_cursor = (elapsed % 1000) < 500;

        if should_show_cursor {
            let cursor_x = text_x + (cursor_col as f32 * MONOSPACE_CHAR_WIDTH);
            let cursor_y = text_y + (cursor_line_idx as f32 * LINE_HEIGHT);

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

    /// Wrap text and build a position map: (line_idx, col_in_line, original_start_pos, original_end_pos)
    fn wrap_and_map_positions(&self, text: &str, max_width: f32) -> (Vec<String>, Vec<(usize, usize, usize, usize)>) {
        let max_chars = ((max_width / MONOSPACE_CHAR_WIDTH).floor() as usize).max(1);

        let mut wrapped_lines = Vec::new();
        let mut position_map = Vec::new(); // (line_idx, col_start, char_start, char_end)

        let mut original_pos = 0;

        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                // Empty line
                position_map.push((wrapped_lines.len(), 0, original_pos, original_pos));
                wrapped_lines.push(String::new());
                original_pos += 1; // newline
                continue;
            }

            let mut line = String::new();
            let mut line_start_pos = original_pos;

            for word in paragraph.split_whitespace() {
                let needs_space = !line.is_empty();
                let test = if needs_space {
                    format!("{} {}", line, word)
                } else {
                    word.to_string()
                };

                if test.len() <= max_chars {
                    line = test;
                } else if !line.is_empty() {
                    // Flush current line
                    let line_end_pos = line_start_pos + line.len();
                    position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                    wrapped_lines.push(line.clone());

                    // Start new line with this word
                    line = word.to_string();
                    line_start_pos = line_end_pos + 1; // +1 for space that was consumed
                } else {
                    // Single word too long, break it
                    line = word.to_string();
                }
            }

            // Push remaining line
            if !line.is_empty() {
                let line_end_pos = line_start_pos + line.len();
                position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                wrapped_lines.push(line);
            }

            original_pos += paragraph.len() + 1; // +1 for newline
        }

        if wrapped_lines.is_empty() {
            wrapped_lines.push(String::new());
            position_map.push((0, 0, 0, 0));
        }

        (wrapped_lines, position_map)
    }
}
