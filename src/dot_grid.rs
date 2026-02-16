use iced::widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke, Frame, path::Builder, Text};
use iced::advanced::layout::Layout;
use iced::{Color, Element, Length, Point, Rectangle, Theme as IcedTheme, mouse, Vector, Size};
use iced::alignment;
use crate::card::Card;
use crate::markdown::MarkdownRenderer;

#[derive(Default, Clone)]
pub struct DotGridState {
    last_cursor_pos: Option<Point>,
    is_panning: bool,
    pan_start: Option<Point>,
    dragging_card: Option<usize>,
    drag_offset: Option<Point>, // Changed from drag_start_offset to drag_offset
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
        }
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
        self.cards.push(Card::new(id, snapped_position));
        self.cards_cache.clear();
        println!("Added card {} at world position {:?}", id, snapped_position);
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

    fn draw_cards(&self, frame: &mut Frame, _bounds: Rectangle) {
        for card in &self.cards {
            let screen_x = card.current_position.x + self.offset.x;
            let screen_y = card.current_position.y + self.offset.y;

            let card_rect = Rectangle {
                x: screen_x,
                y: screen_y,
                width: card.width,
                height: card.height,
            };

            // Don't skip drawing - the sidebar/settings will clip using layers
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

            // Draw icon with card's custom color
            let icon_size = 20.0;
            let icon_x = card_rect.x + 5.0;
            let icon_y = card_rect.y + 5.0;

            frame.fill(
                &Path::circle(Point::new(icon_x + icon_size/2.0, icon_y + icon_size/2.0), icon_size/2.0),
                card.color,
            );

            // Only draw content when NOT editing
            // When editing, the text_editor widget will be overlaid on top
            if !card.is_editing {
                let content_text = card.content.text();
                if !content_text.is_empty() {
                    let text_x = card_rect.x + 10.0;
                    let text_y = card_rect.y + top_bar_height + 10.0;
                    let max_width = card_rect.width - 20.0;

                    let renderer = MarkdownRenderer::new(self.card_text, max_width);
                    renderer.render(frame, &content_text, Point::new(text_x, text_y));
                }
            }

            // Draw editing indicator (blue border when editing)
            if card.is_editing {
                frame.stroke(
                    &rounded_rectangle(card_rect, corner_radius),
                    Stroke::default()
                        .with_color(Color::from_rgb8(100, 150, 255))
                        .with_width(2.0),
                );
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
    CardDrag(usize, Point, Point),
    CardDrop(usize),
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
                            let mut clicked_on_card = false;
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
                                    clicked_on_card = true;

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
                                            // If card is being edited, stop editing instead of dragging
                                            if card.is_editing {
                                                return (
                                                    iced::widget::canvas::event::Status::Ignored,
                                                    None,
                                                );
                                            }

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
                                        // Clicked on card body
                                        // If card is being edited, ignore the click so it passes to the overlay
                                        if card.is_editing {
                                            return (
                                                iced::widget::canvas::event::Status::Ignored,
                                                None,
                                            );
                                        }

                                        // Otherwise, start editing
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
                        if let Some(card_id) = state.dragging_card {
                            state.dragging_card = None;
                            state.drag_offset = None;
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(DotGridMessage::CardDrop(card_id)),
                            );
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
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds);
            });
            return vec![static_layer, cards_layer];
        }

        let local_mouse = cursor.position_in(bounds);

        // If no mouse in bounds, just return static and cards layers
        if local_mouse.is_none() {
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds);
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
            let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
                self.draw_cards(frame, bounds);
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
        let cards_layer = self.cards_cache.draw(renderer, bounds.size(), |frame| {
            self.draw_cards(frame, bounds);
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
            // Check if hovering over a card's top bar (excluding icon)
            for card in &self.cards {
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
        }

        mouse::Interaction::default()
    }
}

fn rects_intersect(a: Rectangle, b: Rectangle) -> bool {
    let ax2 = a.x + a.width;
    let ay2 = a.y + a.height;
    let bx2 = b.x + b.width;
    let by2 = b.y + b.height;

    !(ax2 <= b.x || bx2 <= a.x || ay2 <= b.y || by2 <= a.y)
}
