use iced::widget::canvas::{Path, Frame, Stroke, Text};
use iced::{Color, Point, Rectangle};
use std::time::Instant;

// Default font configuration
const DEFAULT_CHAR_WIDTH: f32 = 8.43; // Character width for monospace at 14px
const DEFAULT_LINE_HEIGHT: f32 = 21.0;

/// A simple custom text editor with visible cursor
#[derive(Debug, Clone)]
pub struct CustomTextEditor {
    pub text: String,
    pub cursor_position: usize, // Character position in string
    pub last_blink: Instant,
    pub selection_start: Option<usize>,
    pub scroll_offset: usize, // Line offset for scrolling
    pub font: iced::Font,
    pub font_size: f32,
    pub char_width: f32,
    pub line_height: f32,
}

impl Default for CustomTextEditor {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            last_blink: Instant::now(),
            selection_start: None,
            scroll_offset: 0,
            font: iced::Font::MONOSPACE,
            font_size: 14.0,
            char_width: DEFAULT_CHAR_WIDTH,
            line_height: DEFAULT_LINE_HEIGHT,
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
            font: iced::Font::MONOSPACE,
            font_size: 14.0,
            char_width: DEFAULT_CHAR_WIDTH,
            line_height: DEFAULT_LINE_HEIGHT,
        }
    }

    pub fn set_font(&mut self, font: iced::Font, size: f32) {
        self.font = font;
        self.font_size = size;
        // Recalculate character width and line height based on font size
        // These are empirically determined scaling factors for monospace fonts
        self.char_width = 8.43 * (size / 14.0);
        self.line_height = 21.0 * (size / 14.0);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        // Move cursor to end
        self.cursor_position = self.text.len();
        self.selection_start = None;
        self.last_blink = Instant::now();
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

    pub fn move_cursor_word_left(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.selection_start = None;
        }

        if self.cursor_position > 0 {
            let before_cursor = &self.text[..self.cursor_position];
            let mut pos = self.cursor_position;

            // Skip trailing whitespace
            while pos > 0 {
                if let Some(ch) = before_cursor.chars().nth_back((self.cursor_position - pos) as usize) {
                    if !ch.is_whitespace() {
                        break;
                    }
                    pos -= ch.len_utf8();
                } else {
                    break;
                }
            }

            // Skip word characters
            while pos > 0 {
                if let Some(ch) = before_cursor.chars().nth_back((self.cursor_position - pos) as usize) {
                    if ch.is_whitespace() {
                        break;
                    }
                    pos -= ch.len_utf8();
                } else {
                    break;
                }
            }

            self.cursor_position = pos;
        }
        self.last_blink = Instant::now();
    }

    pub fn move_cursor_word_right(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.selection_start = None;
        }

        if self.cursor_position < self.text.len() {
            let after_cursor = &self.text[self.cursor_position..];
            let mut chars_to_move = 0;
            let mut found_word = false;

            // Skip leading whitespace
            for ch in after_cursor.chars() {
                if !ch.is_whitespace() {
                    found_word = true;
                    break;
                }
                chars_to_move += ch.len_utf8();
            }

            // Skip word characters
            if found_word {
                for ch in after_cursor[chars_to_move..].chars() {
                    if ch.is_whitespace() {
                        break;
                    }
                    chars_to_move += ch.len_utf8();
                }
            }

            self.cursor_position += chars_to_move;
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

    /// Position cursor based on mouse click coordinates.
    /// `relative_x` / `relative_y` are relative to the text area origin.
    /// `max_width` must match the rendered width so wrapped lines map correctly.
    pub fn click_at_position(&mut self, relative_x: f32, relative_y: f32, max_width: f32) {
        let (_, position_map) = self.wrap_and_map_positions(&self.text, max_width);

        if position_map.is_empty() {
            self.cursor_position = 0;
            self.selection_start = None;
            self.last_blink = Instant::now();
            return;
        }

        let visual_line = ((relative_y / self.line_height) as usize)
            .min(position_map.len() - 1);
        let (_, _, line_start, line_end) = position_map[visual_line];

        self.cursor_position = self.col_to_byte(line_start, line_end, relative_x);
        self.selection_start = None;
        self.last_blink = Instant::now();
    }

    /// Position cursor and start/extend selection based on drag.
    /// `max_width` must match the rendered width so wrapped lines map correctly.
    pub fn drag_to_position(&mut self, relative_x: f32, relative_y: f32, max_width: f32) {
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor_position);
        }

        let (_, position_map) = self.wrap_and_map_positions(&self.text, max_width);

        if position_map.is_empty() {
            self.cursor_position = self.text.len();
            self.last_blink = Instant::now();
            return;
        }

        let visual_line = ((relative_y / self.line_height) as usize)
            .min(position_map.len() - 1);
        let (_, _, line_start, line_end) = position_map[visual_line];

        self.cursor_position = self.col_to_byte(line_start, line_end, relative_x);
        self.last_blink = Instant::now();
    }

    /// Map an x pixel offset within a visual line (given by its [start, end] byte range)
    /// to a byte position in `self.text`.
    fn col_to_byte(&self, line_start: usize, line_end: usize, relative_x: f32) -> usize {
        let start = line_start.min(self.text.len());
        let end = line_end.min(self.text.len());
        let line_text = &self.text[start..end];
        let char_index = (relative_x / self.char_width).round() as usize;
        let mut byte_offset = 0;
        let mut char_count = 0;
        for ch in line_text.chars() {
            if char_count >= char_index { break; }
            byte_offset += ch.len_utf8();
            char_count += 1;
        }
        (start + byte_offset).min(self.text.len())
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

    /// Wrap the selected text (or insert at cursor if no selection) with prefix and suffix
    pub fn wrap_selection(&mut self, prefix: &str, suffix: &str) {
        if let Some(sel_start) = self.selection_start {
            // Get selection bounds
            let start = sel_start.min(self.cursor_position);
            let end = sel_start.max(self.cursor_position);
            
            // Build new text by cloning the current text first
            let old_text = self.text.clone();
            let before = &old_text[..start];
            let selected = &old_text[start..end];
            let after = &old_text[end..];
            
            // Build new text
            self.text = format!("{}{}{}{}{}", before, prefix, selected, suffix, after);
            
            // Position cursor after the wrapped text
            self.cursor_position = start + prefix.len() + selected.len() + suffix.len();
            self.selection_start = None;
        } else {
            // No selection, just insert prefix+suffix at cursor
            let old_text = self.text.clone();
            let before = &old_text[..self.cursor_position];
            let after = &old_text[self.cursor_position..];
            
            self.text = format!("{}{}{}{}", before, prefix, suffix, after);
            // Position cursor between prefix and suffix
            self.cursor_position += prefix.len();
        }
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
        let max_lines = ((max_height / self.line_height).floor() as usize).max(1);

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
            for (line_idx, _line) in wrapped_lines.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
                let (_, _, line_start, line_end) = position_map[line_idx];

                // Check if this line contains selection
                if sel_end >= line_start && sel_begin <= line_end {
                    let display_idx = line_idx - visible_start;
                    let y = text_y + (display_idx as f32 * self.line_height);

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

                    let sel_x_start = text_x + (chars_before_sel as f32 * self.char_width);
                    let sel_x_end = text_x + ((chars_before_sel + chars_in_sel) as f32 * self.char_width);

                    // Draw selection rectangle
                    use iced::widget::canvas::{path::Builder, Fill};
                    let mut path_builder = Builder::new();
                    path_builder.rectangle(
                        Point::new(sel_x_start, y),
                        iced::Size::new(sel_x_end - sel_x_start, self.line_height)
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
            let y = text_y + (display_idx as f32 * self.line_height);

            frame.fill_text(Text {
                content: line.clone(),
                position: Point::new(text_x, y),
                color: text_color,
                size: self.font_size.into(),
                font: self.font,
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
                let cursor_x = text_x + (cursor_col as f32 * self.char_width);
                let cursor_y = text_y + (display_line as f32 * self.line_height);

                let cursor_path = Path::line(
                    Point::new(cursor_x, cursor_y),
                    Point::new(cursor_x, cursor_y + self.font_size),
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
        let max_lines = ((max_height / self.line_height).floor() as usize).max(1);
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
        let max_chars = ((max_width / self.char_width).floor() as usize).saturating_sub(1).max(1);

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

            let mut line = String::new();
            let mut line_start_pos = original_pos;
            let chars: Vec<char> = paragraph.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                let ch = chars[i];

                // Check if adding this character would exceed max width
                if line.chars().count() >= max_chars {
                    // Try to find last space in current line to break there
                    if let Some(last_space_idx) = line.rfind(' ') {
                        // Break at space
                        let before_space = line[..last_space_idx].to_string();
                        let after_space = line[last_space_idx + 1..].to_string();

                        if !before_space.is_empty() {
                            let line_end_pos = line_start_pos + before_space.len() + 1; // +1 for space
                            position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                            wrapped_lines.push(before_space);

                            // Start new line with remainder after space
                            line = after_space;
                            line_start_pos = line_end_pos;
                        }
                    } else {
                        // No space found, break at character boundary (long word)
                        let line_end_pos = line_start_pos + line.len();
                        position_map.push((wrapped_lines.len(), 0, line_start_pos, line_end_pos));
                        wrapped_lines.push(line.clone());

                        line.clear();
                        line_start_pos = line_end_pos;
                    }
                }
                
                // Add the character
                line.push(ch);
                i += 1;
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
