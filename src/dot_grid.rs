use iced::widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke, Frame, path::Builder};
use iced::{Color, Element, Length, Point, Rectangle, Theme as IcedTheme, mouse, Vector};
use crate::card::Card;
use crate::markdown::MarkdownRenderer;

pub struct DotGridState {
    last_cursor_pos: Option<Point>,
    is_panning: bool,
    pan_start: Option<Point>,
    dragging_card: Option<usize>,
    drag_offset: Option<Point>, // Changed from drag_start_offset to drag_offset
    resizing_card: Option<usize>,
    resize_start_size: Option<(f32, f32)>, // (width, height)
    resize_start_pos: Option<Point>,
    hovered_card: Option<usize>, // NEW: Track which card is hovered
    selecting_text_card: Option<usize>, // Track which card is being text-selected
}

impl Default for DotGridState {
    fn default() -> Self {
        Self {
            last_cursor_pos: None,
            is_panning: false,
            pan_start: None,
            dragging_card: None,
            drag_offset: None,
            resizing_card: None,
            resize_start_size: None,
            resize_start_pos: None,
            hovered_card: None,
            selecting_text_card: None,
        }
    }
}

pub struct DotGrid {
    dot_spacing: f32,
    dot_radius: f32,
    influence_radius: f32,
    line_radius: f32,
    pull_strength: f32,
    dot_color: Color,
    background_color: Color,
    static_cache: Cache,
    cards_cache: Cache,
    offset: Vector,
    exclude_region: Option<Rectangle>,
    effect_enabled: bool,
    cards: Vec<Card>,
    card_background: Color,
    card_border: Color,
    card_text: Color,
    font: iced::Font,
    font_size: f32,
    debug_mode: bool,
}

impl DotGrid {
    pub fn new(dot_color: Color, background_color: Color) -> Self {
        Self {
            dot_spacing: 30.0,
            dot_radius: 2.0,
            influence_radius: 150.0,
            line_radius: 150.0,
            pull_strength: 5.0,
            dot_color,
            background_color,
            static_cache: Cache::new(),
            cards_cache: Cache::new(),
            offset: Vector::new(0.0, 0.0),
            exclude_region: None,
            effect_enabled: true,
            cards: Vec::new(),
            card_background: Color::WHITE,
            card_border: Color::from_rgb8(200, 200, 200),
            card_text: Color::from_rgb8(51, 51, 51),
            font: iced::Font::MONOSPACE,
            font_size: 14.0,
            debug_mode: false,
        }
    }

    pub fn set_font(&mut self, font: iced::Font, size: f32) {
        if self.debug_mode {
            println!("DEBUG: DotGrid.set_font called - size: {}, cards count: {}", size, self.cards.len());
        }
        self.font = font;
        self.font_size = size;
        // Update all existing cards
        for card in &mut self.cards {
            card.content.set_font(font, size);
            if self.debug_mode {
                println!("DEBUG: Updated card {} with font size {}", card.id, size);
            }
        }
        self.cards_cache.clear();
        if self.debug_mode {
            println!("DEBUG: Cards cache cleared");
        }
    }

    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    pub fn set_dot_color(&mut self, color: Color) {
        self.dot_color = color;
        self.static_cache.clear();
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_offset(&mut self, offset: Vector) {
        self.offset = offset;
        self.static_cache.clear();
        self.cards_cache.clear();
    }

    pub fn set_exclude_region(&mut self, region: Option<Rectangle>) {
        let changed = match (&self.exclude_region, &region) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(old), Some(new)) => {
                old.x != new.x || old.y != new.y || old.width != new.width || old.height != new.height
            }
        };

        if changed {
            self.exclude_region = region;
            self.static_cache.clear();
        }
    }

    pub fn set_effect_enabled(&mut self, enabled: bool) {
        self.effect_enabled = enabled;
    }

    pub fn set_dot_spacing(&mut self, spacing: f32) {
        self.dot_spacing = spacing;
        self.static_cache.clear();
    }

    pub fn set_dot_radius(&mut self, radius: f32) {
        self.dot_radius = radius;
        self.static_cache.clear();
    }

    pub fn set_card_colors(&mut self, background: Color, border: Color, text: Color) {
        self.card_background = background;
        self.card_border = border;
        self.card_text = text;
        self.cards_cache.clear();
    }

    pub fn add_card(&mut self, screen_position: Point) -> usize {
        let id = self.cards.len();
        let world_position = Point::new(
            screen_position.x - self.offset.x,
            screen_position.y - self.offset.y,
        );
        let snapped_position = Card::snap_to_grid(world_position, self.dot_spacing);
        let mut card = Card::new(id, snapped_position);
        if self.debug_mode {
            println!("DEBUG: Setting font on new card {}: font_size={}", id, self.font_size);
        }
        card.content.set_font(self.font, self.font_size);
        self.cards.push(card);
        self.cards_cache.clear();
        if self.debug_mode {
            println!("Added card {} at world position {:?}", id, snapped_position);
        }
        id
    }

    pub fn add_card_with_content(
        &mut self,
        screen_position: Point,
        content: &str,
        icon: crate::card::CardIcon,
        color: Color,
    ) -> usize {
        let id = self.cards.len();
        let world_position = Point::new(
            screen_position.x - self.offset.x,
            screen_position.y - self.offset.y,
        );
        let snapped_position = Card::snap_to_grid(world_position, self.dot_spacing);
        let mut card = Card::new(id, snapped_position);
        card.content = crate::custom_text_editor::CustomTextEditor::with_text(content);
        if self.debug_mode {
            println!("DEBUG: Setting font on new card {}: font_size={}", id, self.font_size);
        }
        card.content.set_font(self.font, self.font_size);
        card.icon = icon;
        card.color = color;
        self.cards.push(card);
        self.cards_cache.clear();
        id
    }

    pub fn add_card_with_size(
        &mut self,
        screen_position: Point,
        content: &str,
        icon: crate::card::CardIcon,
        color: Color,
        width: f32,
        height: f32,
    ) -> usize {
        let id = self.cards.len();
        let world_position = Point::new(
            screen_position.x - self.offset.x,
            screen_position.y - self.offset.y,
        );
        let snapped_position = Card::snap_to_grid(world_position, self.dot_spacing);
        let mut card = Card::new(id, snapped_position);
        card.content = crate::custom_text_editor::CustomTextEditor::with_text(content);
        card.content.set_font(self.font, self.font_size);
        card.icon = icon;
        card.color = color;
        // Set the custom size
        card.width = width;
        card.height = height;
        card.target_width = width;
        card.target_height = height;
        self.cards.push(card);
        self.cards_cache.clear();
        id
    }

    pub fn clear_cards_cache(&mut self) {
        self.cards_cache.clear();
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn cards_mut(&mut self) -> &mut [Card] {
        &mut self.cards
    }
    
    pub fn load_cards(&mut self, cards: Vec<Card>) {
        self.cards = cards;
        self.cards_cache.clear();
    }

    pub fn delete_card(&mut self, card_id: usize) {
        self.cards.retain(|c| c.id != card_id);
        self.cards_cache.clear();
    }

    pub fn update_card_animation(&mut self, delta_time: f32) {
        for card in &mut self.cards {
            card.update_animation(delta_time);
        }
        self.cards_cache.clear();
    }

    pub fn dot_spacing(&self) -> f32 {
        self.dot_spacing
    }

    pub fn offset(&self) -> Vector {
        self.offset
    }

    pub fn is_editing_any_card(&self) -> bool {
        self.cards.iter().any(|card| card.is_editing)
    }

    /// Update checkbox positions for a card after rendering
    pub fn update_card_checkbox_positions(&mut self, card_id: usize) {
        use crate::text_processor::TextProcessor;

        if let Some(card) = self.cards.iter_mut().find(|c| c.id == card_id) {
            card.checkbox_positions.clear();

            if !card.is_editing {
                let content_text = card.content.text();
                if !content_text.is_empty() {
                    // Process the text to get the document (matches what renderer does)
                    let processor = TextProcessor::with_font_size(self.font_size);
                    let document = processor.process(&content_text);

                    // Calculate checkbox positions exactly as the renderer does
                    let top_bar_height = 30.0;
                    let card_screen_x = card.current_position.x + self.offset.x;
                    let card_screen_y = card.current_position.y + self.offset.y;
                    let text_x = 10.0;
                    let text_y = top_bar_height + 10.0;

                    let mut current_y = 0.0;

                    for line in &document.lines {
                        if line.is_empty() {
                            current_y += 8.0;
                            continue;
                        }

                        // Add spacing before line
                        current_y += line.spacing_before;

                        // Calculate line height matching renderer logic
                        let max_size = line.segments.iter()
                            .map(|seg| seg.style.size)
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(14.0);
                        let line_height = 21.0 * (max_size / 14.0);

                        // If line has a checkbox, store its position
                        if let Some(checkbox) = &line.checkbox {
                            let checkbox_size = 14.0;
                            let checkbox_x = text_x - 20.0 + line.indent;
                            let checkbox_y = current_y + 2.0;

                            let checkbox_rect = Rectangle {
                                x: card_screen_x + checkbox_x,
                                y: card_screen_y + text_y + checkbox_y,
                                width: checkbox_size,
                                height: checkbox_size,
                            };

                            card.checkbox_positions.push(crate::text_renderer::CheckboxPosition {
                                rect: checkbox_rect,
                                line_index: checkbox.line_index,
                                checked: checkbox.checked,
                            });
                        }

                        // Update Y position
                        current_y += line_height + line.spacing_after;
                    }
                }
            }
        }
    }

    /// Check if a point clicks on a checkbox by computing positions on-the-fly
    pub fn find_clicked_checkbox(&self, screen_pos: Point) -> Option<(usize, usize)> {
        if self.debug_mode {
            println!("DEBUG: find_clicked_checkbox at pos: {:?}", screen_pos);
        }

        // Use the stored checkbox positions from rendering
        // These are the ACTUAL positions where checkboxes were rendered
        for card in &self.cards {
            if !card.is_editing {
                if self.debug_mode {
                    println!("DEBUG: Checking card {} with {} stored checkbox positions",
                        card.id, card.checkbox_positions.len());
                }

                for checkbox_pos in &card.checkbox_positions {
                    if self.debug_mode {
                        println!("DEBUG: Stored checkbox line_index={}, rect={:?}",
                            checkbox_pos.line_index, checkbox_pos.rect);
                    }

                    if checkbox_pos.rect.contains(screen_pos) {
                        if self.debug_mode {
                            println!("DEBUG: CHECKBOX CLICKED! Returning line_index={}", checkbox_pos.line_index);
                        }
                        return Some((card.id, checkbox_pos.line_index));
                    }
                }
            }
        }

        if self.debug_mode {
            println!("DEBUG: No checkbox found at click position");
        }
        None
    }

    pub fn view(&self) -> Element<'_, DotGridMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn is_in_exclude_region(&self, x: f32, y: f32) -> bool {
        if let Some(region) = self.exclude_region {
            // Add some padding around the region
            let padded = Rectangle {
                x: region.x - 5.0,
                y: region.y - 5.0,
                width: region.width + 10.0,
                height: region.height + 10.0,
            };
            padded.contains(Point::new(x, y))
        } else {
            false
        }
    }

    fn draw_static_dots(&self, frame: &mut Frame, bounds: Rectangle) {
        // Calculate the visible grid range based on offset
        let offset_x = self.offset.x % self.dot_spacing;
        let offset_y = self.offset.y % self.dot_spacing;

        let cols = (bounds.width / self.dot_spacing) as i32 + 2;
        let rows = (bounds.height / self.dot_spacing) as i32 + 2;

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * self.dot_spacing + offset_x;
                let y = row as f32 * self.dot_spacing + offset_y;

                // Skip dots in the exclude region
                if !self.is_in_exclude_region(x, y) {
                    frame.fill(&Path::circle(Point::new(x, y), self.dot_radius), self.dot_color);
                }
            }
        }
    }

    fn draw_cards(&self, frame: &mut Frame, _bounds: Rectangle, hovered_card: Option<usize>) {
        for card in &self.cards {
            let screen_x = card.current_position.x + self.offset.x;
            let screen_y = card.current_position.y + self.offset.y;

            let card_rect = Rectangle {
                x: screen_x,
                y: screen_y,
                width: card.width,
                height: card.height,
            };

            // Sidebar now uses renderer.with_layer() to ensure it renders on top of canvas
            let corner_radius = 12.0;

            // Draw card background with theme color
            frame.fill(
                &rounded_rectangle(card_rect, corner_radius),
                self.card_background,
            );

            // Draw card border
            frame.stroke(
                &rounded_rectangle(card_rect, corner_radius),
                Stroke::default()
                    .with_color(self.card_border)
                    .with_width(1.0),
            );

            // Draw top bar background
            let top_bar_height = 30.0;
            let top_bar_rect = Rectangle {
                x: card_rect.x,
                y: card_rect.y,
                width: card_rect.width,
                height: top_bar_height,
            };
            frame.fill(
                &rounded_rectangle_top(top_bar_rect, corner_radius),
                Color::from_rgba(
                    self.card_border.r,
                    self.card_border.g,
                    self.card_border.b,
                    0.3,
                ),
            );

            // Icons are now rendered as SVG widget overlays (not in canvas)

            // Draw content or custom editor
            let content_text = card.content.text();
            
            if card.is_editing {
                // Render custom editor with cursor directly in canvas
                let editor_bounds = Rectangle {
                    x: card_rect.x,
                    y: card_rect.y + top_bar_height,
                    width: card_rect.width,
                    height: card_rect.height - top_bar_height,
                };
                
                // Cursor color based on theme
                let cursor_color = if self.card_text.r > 0.5 {
                    Color::WHITE
                } else {
                    Color::BLACK
                };
                
                // Selection color based on theme
                let selection_color = if self.card_text.r > 0.5 {
                    // Dark mode - use light grey/white with transparency
                    Color::from_rgba(1.0, 1.0, 1.0, 0.3)
                } else {
                    // Light mode - use dark grey with transparency
                    Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                };

                card.content.render(
                    frame,
                    editor_bounds,
                    self.card_text,
                    cursor_color,
                    selection_color,
                );
            } else if !content_text.is_empty() {
                // Render as markdown when not editing
                let text_x = card_rect.x + 10.0;
                let text_y = card_rect.y + top_bar_height + 10.0;
                let max_width = card_rect.width - 20.0;
                let max_height = card_rect.height - top_bar_height - 20.0;

                let renderer = MarkdownRenderer::with_fonts_size_and_height(
                    self.card_text, 
                    max_width,
                    max_height,
                    self.font, 
                    self.font_size
                );
                let (_height, _checkbox_positions) = renderer.render(frame, &content_text, Point::new(text_x, text_y));

                // Note: Checkbox positions are updated via update_card_checkbox_positions()
                // which is called after editing or content changes
            }

            // Draw editing indicator (use card's color for border when editing)
            if card.is_editing {
                frame.stroke(
                    &rounded_rectangle(card_rect, corner_radius),
                    Stroke::default()
                        .with_color(card.color)
                        .with_width(3.0),
                );
            }

            // Draw resize handle when editing OR hovering
            let show_resize_handle = card.is_editing || hovered_card == Some(card.id);
            if show_resize_handle {
                let handle_size = 16.0;
                let handle_x = card_rect.x + card_rect.width - handle_size;
                let handle_y = card_rect.y + card_rect.height - handle_size;

                // Draw resize handle background
                frame.fill(
                    &Path::rectangle(Point::new(handle_x, handle_y), iced::Size::new(handle_size, handle_size)),
                    card.color,
                );

                // Draw resize grip lines
                let grip_color = if self.card_text.r > 0.5 { Color::BLACK } else { Color::WHITE };
                for i in 0..3 {
                    let offset = (i as f32 * 4.0) + 4.0;
                    let line = Path::line(
                        Point::new(handle_x + offset, handle_y + handle_size - 2.0),
                        Point::new(handle_x + handle_size - 2.0, handle_y + offset),
                    );
                    frame.stroke(&line, Stroke::default().with_color(grip_color).with_width(1.5));
                }
            }
        }
    }
}

/// Create a rounded rectangle path
fn rounded_rectangle(rect: Rectangle, radius: f32) -> Path {
    let mut builder = Builder::new();

    let x = rect.x;
    let y = rect.y;
    let width = rect.width;
    let height = rect.height;
    let r = radius.min(width / 2.0).min(height / 2.0);

    // Start at top-left, after the corner
    builder.move_to(Point::new(x + r, y));

    // Top edge
    builder.line_to(Point::new(x + width - r, y));

    // Top-right corner
    builder.arc_to(
        Point::new(x + width, y),
        Point::new(x + width, y + r),
        r,
    );

    // Right edge
    builder.line_to(Point::new(x + width, y + height - r));

    // Bottom-right corner
    builder.arc_to(
        Point::new(x + width, y + height),
        Point::new(x + width - r, y + height),
        r,
    );

    // Bottom edge
    builder.line_to(Point::new(x + r, y + height));

    // Bottom-left corner
    builder.arc_to(
        Point::new(x, y + height),
        Point::new(x, y + height - r),
        r,
    );

    // Left edge
    builder.line_to(Point::new(x, y + r));

    // Top-left corner
    builder.arc_to(
        Point::new(x, y),
        Point::new(x + r, y),
        r,
    );

    builder.close();
    builder.build()
}

/// Create a rounded rectangle path with only top corners rounded
fn rounded_rectangle_top(rect: Rectangle, radius: f32) -> Path {
    let mut builder = Builder::new();

    let x = rect.x;
    let y = rect.y;
    let width = rect.width;
    let height = rect.height;
    let r = radius.min(width / 2.0).min(height / 2.0);

    // Start at top-left, after the corner
    builder.move_to(Point::new(x + r, y));

    // Top edge
    builder.line_to(Point::new(x + width - r, y));

    // Top-right corner
    builder.arc_to(
        Point::new(x + width, y),
        Point::new(x + width, y + r),
        r,
    );

    // Right edge (full height)
    builder.line_to(Point::new(x + width, y + height));

    // Bottom edge (no rounding)
    builder.line_to(Point::new(x, y + height));

    // Left edge (full height)
    builder.line_to(Point::new(x, y + r));

    // Top-left corner
    builder.arc_to(
        Point::new(x, y),
        Point::new(x + r, y),
        r,
    );

    builder.close();
    builder.build()
}

#[derive(Debug, Clone)]
pub enum DotGridMessage {
    Pan(Vector),
    RightClick(Point),
    CardRightClickIcon(usize),
    CardLeftClickBar(usize, Point),
    CardLeftClickBody(usize),
    CardTextClick(usize, Point), // (card_id, click_position) - for text selection
    CardTextDrag(usize, Point), // (card_id, drag_position) - for text selection drag
    CardDrag(usize, Point, Point),
    CardDrop(usize),
    CardResizeStart(usize, Point),
    CardResize(usize, Point),
    CardResizeEnd(usize),
    CheckboxToggle(usize, usize), // (card_id, line_index)
}

impl Program<DotGridMessage> for &DotGrid {
    type State = DotGridState;

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<DotGridMessage>) {
        let current_pos = cursor.position_in(bounds);

        match event {
            iced::widget::canvas::Event::Mouse(mouse_event) => {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                        if let Some(pos) = current_pos {
                            state.is_panning = true;
                            state.pan_start = Some(pos);
                            return (iced::widget::canvas::event::Status::Captured, None);
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Middle) => {
                        state.is_panning = false;
                        state.pan_start = None;
                        return (iced::widget::canvas::event::Status::Captured, None);
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(pos) = current_pos {
                            // First check if we're clicking outside all cards while editing
                            let clicked_on_card = false;
                            let mut is_editing_any = false;

                            for card in &self.cards {
                                if card.is_editing {
                                    is_editing_any = true;
                                }

                                let screen_bounds = Rectangle {
                                    x: card.current_position.x + self.offset.x,
                                    y: card.current_position.y + self.offset.y,
                                    width: card.width,
                                    height: card.height,
                                };

                                if screen_bounds.contains(pos) {
                                    let top_bar_bounds = Rectangle {
                                        x: screen_bounds.x,
                                        y: screen_bounds.y,
                                        width: screen_bounds.width,
                                        height: 30.0,
                                    };

                                    if top_bar_bounds.contains(pos) {
                                        // Don't start drag if clicking on icon
                                        let icon_bounds = Rectangle {
                                            x: screen_bounds.x + 5.0,
                                            y: screen_bounds.y + 5.0,
                                            width: 20.0,
                                            height: 20.0,
                                        };

                                        if !icon_bounds.contains(pos) {
                                            // Allow dragging regardless of editing state
                                            state.dragging_card = Some(card.id);
                                            state.drag_offset = Some(Point::new(
                                                pos.x - screen_bounds.x,
                                                pos.y - screen_bounds.y,
                                            ));
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CardLeftClickBar(card.id, pos)),
                                            );
                                        }
                                    } else {
                                        // Check if clicking on resize handle (always available, not just when editing)
                                        let handle_size = 16.0;
                                        let resize_handle_bounds = Rectangle {
                                            x: screen_bounds.x + screen_bounds.width - handle_size,
                                            y: screen_bounds.y + screen_bounds.height - handle_size,
                                            width: handle_size,
                                            height: handle_size,
                                        };

                                        if resize_handle_bounds.contains(pos) {
                                            state.resizing_card = Some(card.id);
                                            state.resize_start_size = Some((card.width, card.height));
                                            state.resize_start_pos = Some(pos);
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CardResizeStart(card.id, pos)),
                                            );
                                        }

                                        // If editing, send click to text editor for cursor positioning
                                        if card.is_editing {
                                            state.selecting_text_card = Some(card.id);
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CardTextClick(card.id, pos)),
                                            );
                                        }

                                        // Check if clicking on a checkbox
                                        if let Some((card_id, line_index)) = self.find_clicked_checkbox(pos) {
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CheckboxToggle(card_id, line_index)),
                                            );
                                        }

                                        // Clicked on card body - start editing
                                        return (
                                            iced::widget::canvas::event::Status::Captured,
                                            Some(DotGridMessage::CardLeftClickBody(card.id)),
                                        );
                                    }

                                    return (iced::widget::canvas::event::Status::Captured, None);
                                }
                            }

                            // If we're editing and clicked outside all cards, stop editing
                            if is_editing_any && !clicked_on_card {
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::CardLeftClickBody(usize::MAX)),
                                );
                            }
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        if let Some(card_id) = state.resizing_card {
                            state.resizing_card = None;
                            state.resize_start_size = None;
                            state.resize_start_pos = None;
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(DotGridMessage::CardResizeEnd(card_id)),
                            );
                        }

                        if let Some(card_id) = state.dragging_card {
                            state.dragging_card = None;
                            state.drag_offset = None;
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(DotGridMessage::CardDrop(card_id)),
                            );
                        }

                        // Clear text selection state
                        if state.selecting_text_card.is_some() {
                            state.selecting_text_card = None;
                        }
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
                        if let Some(pos) = current_pos {
                            // Check if clicking on a card icon
                            for card in &self.cards {
                                let screen_bounds = Rectangle {
                                    x: card.current_position.x + self.offset.x,
                                    y: card.current_position.y + self.offset.y,
                                    width: card.width,
                                    height: card.height,
                                };

                                if screen_bounds.contains(pos) {
                                    let icon_bounds = Rectangle {
                                        x: screen_bounds.x + 5.0,
                                        y: screen_bounds.y + 5.0,
                                        width: 20.0,
                                        height: 20.0,
                                    };

                                    if icon_bounds.contains(pos) {
                                        return (
                                            iced::widget::canvas::event::Status::Captured,
                                            Some(DotGridMessage::CardRightClickIcon(card.id)),
                                        );
                                    }
                                    return (iced::widget::canvas::event::Status::Captured, None);
                                }
                            }

                            // No card clicked, show grid context menu
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(DotGridMessage::RightClick(pos)),
                            );
                        }
                    }
                    mouse::Event::CursorMoved { .. } => {
                        if state.is_panning {
                            if let (Some(current), Some(start)) = (current_pos, state.pan_start) {
                                let delta = Vector::new(
                                    current.x - start.x,
                                    current.y - start.y,
                                );
                                state.pan_start = current_pos;
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::Pan(delta)),
                                );
                            }
                        } else if let Some(card_id) = state.selecting_text_card {
                            // Handle text selection drag
                            if let Some(pos) = current_pos {
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::CardTextDrag(card_id, pos)),
                                );
                            }
                        } else if let Some(card_id) = state.resizing_card {
                            if let Some(pos) = current_pos {
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::CardResize(card_id, pos)),
                                );
                            }
                        } else if let Some(card_id) = state.dragging_card {
                            if let Some(pos) = current_pos {
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::CardDrag(card_id, pos, state.drag_offset.unwrap_or(Point::ORIGIN))),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        // Update cursor position for dot effect
        if let (Some(current), Some(last)) = (current_pos, state.last_cursor_pos) {
            let dx = current.x - last.x;
            let dy = current.y - last.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < 4.0 {
                return (iced::widget::canvas::event::Status::Ignored, None);
            }
        }

        state.last_cursor_pos = current_pos;

        // Update hovered card for resize handle display
        if let Some(pos) = current_pos {
            let mut new_hovered = None;
            for card in &self.cards {
                let screen_bounds = Rectangle {
                    x: card.current_position.x + self.offset.x,
                    y: card.current_position.y + self.offset.y,
                    width: card.width,
                    height: card.height,
                };

                if screen_bounds.contains(pos) {
                    new_hovered = Some(card.id);
                    break;
                }
            }

            if state.hovered_card != new_hovered {
                state.hovered_card = new_hovered;
                self.cards_cache.clear(); // Force redraw to show/hide resize handle
            }
        } else {
            if state.hovered_card.is_some() {
                state.hovered_card = None;
                self.cards_cache.clear();
            }
        }

        (iced::widget::canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &IcedTheme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Draw static dots layer (cached)
        let static_layer = self.static_cache.draw(renderer, bounds.size(), |frame| {
            self.draw_static_dots(frame, bounds);
        });

        // If effect is disabled, just return static layer and draw cards on top
        if !self.effect_enabled {
            let hovered = state.hovered_card;
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds, hovered);
            });
            return vec![static_layer, cards_layer];
        }

        let local_mouse = cursor.position_in(bounds);

        // If no mouse in bounds, just return static and cards layers
        if local_mouse.is_none() {
            let hovered = state.hovered_card;
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds, hovered);
            });
            return vec![static_layer, cards_layer];
        }

        let mouse_pos = local_mouse.unwrap();

        // Draw dynamic overlay (lines + displaced dots)
        let mut dynamic_frame = Frame::new(renderer, bounds.size());

        // Calculate grid position accounting for offset
        let offset_x = self.offset.x % self.dot_spacing;
        let offset_y = self.offset.y % self.dot_spacing;

        let cols = (bounds.width / self.dot_spacing) as i32 + 2;
        let rows = (bounds.height / self.dot_spacing) as i32 + 2;

        // Calculate affected range
        let affect_range = (self.influence_radius.max(self.line_radius) / self.dot_spacing).ceil() as i32 + 1;
        let mouse_col = ((mouse_pos.x - offset_x) / self.dot_spacing).round() as i32;
        let mouse_row = ((mouse_pos.y - offset_y) / self.dot_spacing).round() as i32;

        let min_col = (mouse_col - affect_range).max(0);
        let max_col = (mouse_col + affect_range).min(cols - 1);
        let min_row = (mouse_row - affect_range).max(0);
        let max_row = (mouse_row + affect_range).min(rows - 1);

        if max_col < min_col || max_row < min_row {
            let hovered = state.hovered_card;
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds, hovered);
            });
            return vec![static_layer, cards_layer];
        }

        let influence_radius_sq = self.influence_radius * self.influence_radius;
        let line_radius_sq = self.line_radius * self.line_radius;

        let affected_cols = (max_col - min_col + 1) as usize;
        let affected_rows = (max_row - min_row + 1) as usize;
        let mut affected_positions: Vec<(Point, Point, f32, bool)> = Vec::with_capacity(affected_cols * affected_rows);

        // Calculate affected dot positions
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let base_x = col as f32 * self.dot_spacing + offset_x;
                let base_y = row as f32 * self.dot_spacing + offset_y;
                let base_pos = Point::new(base_x, base_y);

                // Check if in exclude region
                let in_exclude = self.is_in_exclude_region(base_x, base_y);

                let dx = mouse_pos.x - base_x;
                let dy = mouse_pos.y - base_y;
                let dist_sq = dx * dx + dy * dy;

                let line_factor = if dist_sq < line_radius_sq && dist_sq > 0.0 {
                    let distance = dist_sq.sqrt();
                    1.0 - (distance / self.line_radius)
                } else {
                    0.0
                };

                let draw_pos = if dist_sq < influence_radius_sq && dist_sq > 0.0 {
                    let distance = dist_sq.sqrt();
                    let factor = 1.0 - (distance / self.influence_radius);
                    let pull = factor * factor * self.pull_strength;
                    let pull_x = (dx / distance) * pull;
                    let pull_y = (dy / distance) * pull;
                    Point::new(base_x + pull_x, base_y + pull_y)
                } else {
                    base_pos
                };

                affected_positions.push((base_pos, draw_pos, line_factor, in_exclude));
            }
        }

        // Draw lines between affected dots (skip if either end is in exclude region)
        for (idx, &(_, draw_pos, line_factor, in_exclude)) in affected_positions.iter().enumerate() {
            if in_exclude {
                continue;
            }

            if line_factor > 0.01 {
                let row_idx = idx / affected_cols;
                let col_idx = idx % affected_cols;

                // Horizontal line
                if col_idx + 1 < affected_cols {
                    let (_, next_pos, next_factor, next_exclude) = affected_positions[idx + 1];
                    if !next_exclude {
                        let avg_factor = (line_factor + next_factor) * 0.5;
                        if avg_factor > 0.01 {
                            let line_color = Color::from_rgba(
                                self.dot_color.r,
                                self.dot_color.g,
                                self.dot_color.b,
                                self.dot_color.a * avg_factor * 0.8,
                            );
                            dynamic_frame.stroke(
                                &Path::line(draw_pos, next_pos),
                                Stroke::default().with_color(line_color).with_width(1.0),
                            );
                        }
                    }
                }

                // Vertical line
                if row_idx + 1 < affected_rows {
                    let next_idx = idx + affected_cols;
                    if next_idx < affected_positions.len() {
                        let (_, next_pos, next_factor, next_exclude) = affected_positions[next_idx];
                        if !next_exclude {
                            let avg_factor = (line_factor + next_factor) * 0.5;
                            if avg_factor > 0.01 {
                                let line_color = Color::from_rgba(
                                    self.dot_color.r,
                                    self.dot_color.g,
                                    self.dot_color.b,
                                    self.dot_color.a * avg_factor * 0.8,
                                );
                                dynamic_frame.stroke(
                                    &Path::line(draw_pos, next_pos),
                                    Stroke::default().with_color(line_color).with_width(1.0),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Redraw affected dots (skip if in exclude region)
        for (base_pos, draw_pos, _, in_exclude) in &affected_positions {
            if !in_exclude {
                dynamic_frame.fill(&Path::circle(*base_pos, self.dot_radius + 1.0), self.background_color);
                dynamic_frame.fill(&Path::circle(*draw_pos, self.dot_radius), self.dot_color);
            }
        }

        // Draw cards layer LAST so they appear on top
        let hovered = state.hovered_card;
        let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
            self.draw_cards(frame, bounds, hovered);
        });

        vec![static_layer, dynamic_frame.into_geometry(), cards_layer]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position_in(bounds) {
            // Check if hovering over resize handle
            for card in &self.cards {
                let screen_bounds = Rectangle {
                    x: card.current_position.x + self.offset.x,
                    y: card.current_position.y + self.offset.y,
                    width: card.width,
                    height: card.height,
                };

                if screen_bounds.contains(pos) {
                    // Check resize handle first
                    let handle_size = 16.0;
                    let resize_handle_bounds = Rectangle {
                        x: screen_bounds.x + screen_bounds.width - handle_size,
                        y: screen_bounds.y + screen_bounds.height - handle_size,
                        width: handle_size,
                        height: handle_size,
                    };

                    if resize_handle_bounds.contains(pos) {
                        return mouse::Interaction::ResizingDiagonallyDown;
                    }

                    // Check top bar for dragging
                    let top_bar_bounds = Rectangle {
                        x: screen_bounds.x,
                        y: screen_bounds.y,
                        width: screen_bounds.width,
                        height: 30.0,
                    };

                    if top_bar_bounds.contains(pos) {
                        let icon_bounds = Rectangle {
                            x: screen_bounds.x + 5.0,
                            y: screen_bounds.y + 5.0,
                            width: 20.0,
                            height: 20.0,
                        };

                        if !icon_bounds.contains(pos) {
                            return mouse::Interaction::Grabbing;
                        }
                    }
                }
            }

            // If dragging a card, show grabbing cursor
            if state.dragging_card.is_some() {
                return mouse::Interaction::Grabbing;
            }

            // If resizing a card, show resize cursor
            if state.resizing_card.is_some() {
                return mouse::Interaction::ResizingDiagonallyDown;
            }
        }

        mouse::Interaction::default()
    }
}