/// CardLayer — draws all cards in z-order, each in its own compositor layer via
/// `renderer.with_layer()`.  This guarantees that card A's text/SVGs never bleed
/// above card B's background even though Iced's canvas pipeline batches text globally.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::widget::canvas::{gradient, Frame, Path, Stroke};
use iced::widget::canvas::path::Builder;
use iced::advanced::svg::{Svg as SvgDrawable, Handle as SvgHandle};
use iced::{Color, Element, Length, Point, Rectangle, Size, Vector};
use std::collections::{HashMap, HashSet};

use crate::card::{Card, CardSide, CardType, Connection, LineStyle};
use crate::markdown::MarkdownRenderer;
use crate::icon_util;

const ICON_TYPE_TEXT:     &[u8] = include_bytes!("icons/type-text.svg");
const ICON_TYPE_MARKDOWN: &[u8] = include_bytes!("icons/type-markdown.svg");
const ICON_TYPE_IMAGE:    &[u8] = include_bytes!("icons/type-image.svg");

fn pin_fill_svg() -> Vec<u8> {
    crate::icon_util::icon_to_svg(icondata_bs::BsPinFill)
}

pub struct CardLayer<'a> {
    cards:                &'a [Card],
    offset:               Vector,
    card_background:      Color,
    card_border:          Color,
    card_text:            Color,
    accent_color:         Color,
    font:                 iced::Font,
    font_size:            f32,
    selected_cards:       &'a HashSet<usize>,
    single_selected_card: Option<usize>,
    hovered_card:         Option<usize>,
    // Connection rendering
    connections:          &'a [Connection],
    pending_conn:         Option<(usize, CardSide)>,
    pending_cursor:       Point,
    conn_anim_phase:      f32,
    conn_slot_anim:       Option<&'a HashMap<(usize, CardSide, usize, CardSide), (f32, f32)>>,
    // Zoom
    zoom:                 f32,
}

impl<'a> CardLayer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cards:                &'a [Card],
        offset:               Vector,
        card_background:      Color,
        card_border:          Color,
        card_text:            Color,
        accent_color:         Color,
        font:                 iced::Font,
        font_size:            f32,
        selected_cards:       &'a HashSet<usize>,
        single_selected_card: Option<usize>,
        hovered_card:         Option<usize>,
    ) -> Self {
        Self {
            cards,
            offset,
            card_background,
            card_border,
            card_text,
            accent_color,
            font,
            font_size,
            selected_cards,
            single_selected_card,
            hovered_card,
            connections: &[],
            pending_conn: None,
            pending_cursor: Point::ORIGIN,
            conn_anim_phase: 0.0,
            conn_slot_anim: None,
            zoom: 1.0,
        }
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// Attach connection rendering data.
    pub fn with_connections(
        mut self,
        connections:     &'a [Connection],
        pending_conn:    Option<(usize, CardSide)>,
        pending_cursor:  Point,
        conn_anim_phase: f32,
        conn_slot_anim:  &'a HashMap<(usize, CardSide, usize, CardSide), (f32, f32)>,
    ) -> Self {
        self.connections     = connections;
        self.pending_conn    = pending_conn;
        self.pending_cursor  = pending_cursor;
        self.conn_anim_phase = conn_anim_phase;
        self.conn_slot_anim  = Some(conn_slot_anim);
        self
    }

    /// Draw a single card into a Frame (same logic as DotGrid::draw_single_card).
    fn draw_card(&self, frame: &mut Frame, card: &Card) {
        let screen_x = card.current_position.x + self.offset.x;
        let screen_y = card.current_position.y + self.offset.y;
        let corner_radius = 12.0;
        let top_bar_height = 30.0;

        let card_rect = Rectangle { x: screen_x, y: screen_y, width: card.width, height: card.height };

        // Background
        frame.fill(&rounded_rectangle(card_rect, corner_radius), self.card_background);

        // Border
        frame.stroke(
            &rounded_rectangle(card_rect, corner_radius),
            Stroke::default().with_color(self.card_border).with_width(1.0),
        );

        // Top bar gradient
        let top_bar_rect = Rectangle { x: card_rect.x, y: card_rect.y, width: card_rect.width, height: top_bar_height };
        {
            let bar_left  = Color::from_rgba(self.card_border.r, self.card_border.g, self.card_border.b, 0.15);
            let bar_right = Color { r: card.color.r, g: card.color.g, b: card.color.b, a: 0.30 };
            let grad = gradient::Linear::new(
                Point::new(top_bar_rect.x, top_bar_rect.y),
                Point::new(top_bar_rect.x + top_bar_rect.width, top_bar_rect.y),
            )
            .add_stop(0.0, bar_left)
            .add_stop(1.0, bar_right);
            frame.fill(&rounded_rectangle_top(top_bar_rect, corner_radius), grad);
        }

        // Icons
        {
            let icon_size = 18.0;
            let icon_y   = screen_y + (top_bar_height - icon_size) / 2.0;

            let left_bounds = Rectangle { x: screen_x + 8.0, y: icon_y, width: icon_size, height: icon_size };
            let icon_data   = icon_util::icon_to_svg(card.icon.get_icondata());
            let left_handle = SvgHandle::from_memory(icon_data);
            frame.draw_svg(left_bounds, SvgDrawable::new(left_handle).color(card.color));

            // Pin indicator — drawn to the left of the type icon when card is pinned
            let pin_offset = if card.pinned {
                let pin_size = 14.0;
                let pin_x = screen_x + card.width - icon_size - 8.0 - pin_size - 4.0;
                let pin_y = screen_y + (top_bar_height - pin_size) / 2.0;
                let pin_bounds = Rectangle { x: pin_x, y: pin_y, width: pin_size, height: pin_size };
                let pin_color = Color { a: 0.85, ..card.color };
                frame.draw_svg(pin_bounds, SvgDrawable::new(SvgHandle::from_memory(pin_fill_svg())).color(pin_color));
                pin_size + 4.0
            } else {
                0.0
            };

            let right_bounds = Rectangle {
                x: screen_x + card.width - icon_size - 8.0,
                y: icon_y,
                width: icon_size,
                height: icon_size,
            };
            let type_data: &[u8] = match card.card_type {
                CardType::Text     => ICON_TYPE_TEXT,
                CardType::Markdown => ICON_TYPE_MARKDOWN,
                CardType::Image    => ICON_TYPE_IMAGE,
            };
            let right_handle = SvgHandle::from_memory(type_data);
            frame.draw_svg(right_bounds, SvgDrawable::new(right_handle).color(card.color));
            let _ = pin_offset;

            // Collapse chevron button
            {
                let cx = screen_x + card.width - 42.0;
                let cy = screen_y + 15.0;
                let s = 5.0_f32;
                let chevron = if card.collapsed {
                    // right-pointing (▶)
                    Path::new(|b| {
                        b.move_to(Point::new(cx - s * 0.5, cy - s));
                        b.line_to(Point::new(cx + s * 0.5, cy));
                        b.line_to(Point::new(cx - s * 0.5, cy + s));
                        b.close();
                    })
                } else {
                    // down-pointing (▼)
                    Path::new(|b| {
                        b.move_to(Point::new(cx - s, cy - s * 0.5));
                        b.line_to(Point::new(cx + s, cy - s * 0.5));
                        b.line_to(Point::new(cx, cy + s * 0.5));
                        b.close();
                    })
                };
                frame.fill(&chevron, Color { a: 0.55, ..self.card_text });
            }
        }

        // Content (skip when collapsed)
        if !card.collapsed {

        // Content (skip text rendering for Image cards — image is drawn by the renderer)
        if card.card_type != CardType::Image {
            let content_text = card.content.text();
            if card.is_editing {
                let editor_bounds = Rectangle {
                    x: card_rect.x,
                    y: card_rect.y + top_bar_height,
                    width: card_rect.width,
                    height: card_rect.height - top_bar_height,
                };
                let cursor_color    = if self.card_text.r > 0.5 { Color::WHITE } else { Color::BLACK };
                let selection_color = Color { a: 0.28, ..self.accent_color };
                card.content.render(frame, editor_bounds, self.card_text, cursor_color, selection_color);
            } else if !content_text.is_empty() {
                let text_x      = card_rect.x + 10.0;
                let text_y      = card_rect.y + top_bar_height + 10.0;
                let max_width   = card_rect.width - 20.0;
                let max_height  = card_rect.height - top_bar_height - 20.0;

                match card.card_type {
                    CardType::Markdown => {
                        let code_bg   = Color { a: 0.12, ..self.card_text };
                        let mut md_rr = MarkdownRenderer::with_fonts_size_height_and_link(
                            self.card_text, max_width, max_height, self.font, self.font_size, card.color,
                        );
                        md_rr.set_code_bg(code_bg);
                        let _ = md_rr.render_as_markdown(frame, &content_text, Point::new(text_x, text_y));
                    }
                    CardType::Text => {
                        use crate::text_document::{TextDocument, TextLine, TextStyle};
                        use crate::text_renderer::TextRenderer;
                        let default_style = TextStyle::with_base_size(self.font_size);
                        let mut doc = TextDocument::new();
                        for line in content_text.lines() {
                            let mut text_line = TextLine::new();
                            text_line.add_segment(line.to_string(), default_style);
                            doc.add_line(text_line);
                        }
                        let tr = TextRenderer::with_fonts_and_height(
                            self.card_text, max_width, max_height, self.font, self.font,
                        );
                        let _ = tr.render(frame, &doc, Point::new(text_x, text_y));
                    }
                    CardType::Image => unreachable!(),
                }
            }
        } else if card.image_handle.is_none() {
            // Empty image card — draw "click to add image" placeholder
            let cx = card_rect.x + card_rect.width  / 2.0;
            let cy = card_rect.y + top_bar_height + (card_rect.height - top_bar_height) / 2.0;
            let ph_w = 48.0_f32;
            let ph_h = 40.0_f32;
            let ph_x = cx - ph_w / 2.0;
            let ph_y = cy - ph_h / 2.0 - 10.0;
            let placeholder_icon = SvgHandle::from_memory(ICON_TYPE_IMAGE);
            let placeholder_col  = Color { a: 0.25, ..self.card_text };
            frame.draw_svg(
                Rectangle { x: ph_x, y: ph_y, width: ph_w, height: ph_h },
                SvgDrawable::new(placeholder_icon).color(placeholder_col),
            );
            let label_col = Color { a: 0.35, ..self.card_text };
            frame.fill_text(iced::widget::canvas::Text {
                content: "Click to add image".to_string(),
                position: Point::new(cx, cy + ph_h / 2.0 - 4.0),
                color: label_col,
                size: iced::Pixels(12.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment:   iced::alignment::Vertical::Top,
                font: self.font,
                ..Default::default()
            });
        }

        } // end if !card.collapsed (content area)

        // Selection / editing border
        if card.is_editing {
            frame.stroke(
                &rounded_rectangle(card_rect, corner_radius),
                Stroke::default().with_color(card.color).with_width(3.0),
            );
        } else if self.selected_cards.contains(&card.id) || self.single_selected_card == Some(card.id) {
            frame.stroke(
                &rounded_rectangle(card_rect, corner_radius),
                Stroke::default().with_color(self.accent_color).with_width(2.5),
            );
        }

        // Resize handle (show when editing or hovered, but not when collapsed)
        if !card.collapsed && (card.is_editing || self.hovered_card == Some(card.id)) {
            let handle_size = 16.0;
            let handle_x    = card_rect.x + card_rect.width  - handle_size;
            let handle_y    = card_rect.y + card_rect.height - handle_size;
            frame.fill(
                &Path::rectangle(Point::new(handle_x, handle_y), iced::Size::new(handle_size, handle_size)),
                card.color,
            );
            let grip_color = if self.card_text.r > 0.5 { Color::BLACK } else { Color::WHITE };
            for i in 0..3_u32 {
                let off  = (i as f32 * 4.0) + 4.0;
                let line = Path::line(
                    Point::new(handle_x + off,           handle_y + handle_size - 2.0),
                    Point::new(handle_x + handle_size - 2.0, handle_y + off),
                );
                frame.stroke(&line, Stroke::default().with_color(grip_color).with_width(1.5));
            }
        }
    }
}

// ── Widget impl ────────────────────────────────────────────────────────────────

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for CardLayer<'a> {
    fn size(&self) -> Size<Length> {
        Size { width: Length::Fill, height: Length::Fill }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::graphics::geometry::Renderer as GeoRenderer;

        let bounds = layout.bounds();
        let z  = self.zoom;
        // Viewport center in canvas-local coords (frame coords start at bounds origin)
        let cx = bounds.width  / 2.0;
        let cy = bounds.height / 2.0;

        // Helper: apply frame zoom transform around viewport center.
        // frame coordinates are 0-based (bounds.size()), so center = (cx, cy).
        let apply_zoom = |frame: &mut Frame| {
            if z != 1.0 {
                frame.translate(Vector::new(cx, cy));
                frame.scale(z);
                frame.translate(Vector::new(-cx, -cy));
            }
        };

        // Helper: convert a zoom-1 canvas local rect to a zoomed absolute screen rect
        // (for renderer calls that use absolute coordinates).
        let zoomed_abs_rect = |local_x: f32, local_y: f32, w: f32, h: f32| -> Rectangle {
            let ax = bounds.x + local_x;
            let ay = bounds.y + local_y;
            let acx = bounds.x + cx;
            let acy = bounds.y + cy;
            Rectangle {
                x:      acx + (ax - acx) * z,
                y:      acy + (ay - acy) * z,
                width:  w * z,
                height: h * z,
            }
        };

        // ── connection lines (behind cards) ────────────────────────────────
        if !self.connections.is_empty() {
            renderer.with_layer(bounds, |renderer| {
                let mut frame = Frame::new(&*renderer, bounds.size());
                frame.with_save(|frame| {
                    apply_zoom(frame);
                    // Fallback slot map — lazily computed if anim map has no entry for a connection.
                    let mut fallback_slots: Option<HashMap<(usize, CardSide), Vec<usize>>> = None;
                    for (conn_idx, conn) in self.connections.iter().enumerate() {
                        let from = self.cards.iter().find(|c| c.id == conn.from_card);
                        let to   = self.cards.iter().find(|c| c.id == conn.to_card);
                        if let (Some(from), Some(to)) = (from, to) {
                            let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
                            let (from_perp, to_perp) = {
                                // Try the animated offsets first, fall back to slot-based computation.
                                let animated = self.conn_slot_anim.and_then(|a| a.get(&key).copied());
                                if let Some(perps) = animated {
                                    perps
                                } else {
                                    let slots = fallback_slots.get_or_insert_with(|| {
                                        bundle_slots(self.connections, self.cards)
                                    });
                                    let fg = slots.get(&(conn.from_card, conn.from_side)).map(|v| v.as_slice()).unwrap_or(&[]);
                                    let tg = slots.get(&(conn.to_card,   conn.to_side  )).map(|v| v.as_slice()).unwrap_or(&[]);
                                    let fs = fg.iter().position(|&i| i == conn_idx).unwrap_or(0);
                                    let ts = tg.iter().position(|&i| i == conn_idx).unwrap_or(0);
                                    (conn_perp_target(from, conn.from_side, fs, fg.len()),
                                     conn_perp_target(to,   conn.to_side,   ts, tg.len()))
                                }
                            };
                            draw_connection(frame, from, from_perp, to, to_perp, conn, self.offset);
                        }
                    }
                });
                GeoRenderer::draw_geometry(renderer, frame.into_geometry());
            });
        }

        // ── cards ──────────────────────────────────────────────────────────
        for card in self.cards.iter() {
            renderer.with_layer(bounds, |renderer| {
                let mut frame = Frame::new(&*renderer, bounds.size());
                frame.with_save(|frame| {
                    apply_zoom(frame);
                    self.draw_card(frame, card);
                });
                GeoRenderer::draw_geometry(renderer, frame.into_geometry());

                // Draw image content for Image cards (must be after geometry flush).
                // Image renderer uses absolute screen coordinates, so apply zoom manually.
                if card.card_type == CardType::Image && !card.collapsed {
                    let screen_x = card.current_position.x + self.offset.x;
                    let screen_y = card.current_position.y + self.offset.y;
                    let pad = 6.0_f32;
                    let content_rect = zoomed_abs_rect(
                        screen_x + pad,
                        screen_y + 30.0 + pad,
                        card.width  - pad * 2.0,
                        card.height - 30.0 - pad * 2.0,
                    );
                    match &card.image_handle {
                        Some(crate::card::CardImageHandle::Raster(handle)) => {
                            use iced::advanced::image::{Renderer as ImgRenderer, Image};
                            let nat = ImgRenderer::measure_image(renderer, handle);
                            let fit = contain_rect(content_rect,
                                Size::new(nat.width as f32, nat.height as f32));
                            ImgRenderer::draw_image(renderer, Image::new(handle.clone()), fit);
                        }
                        Some(crate::card::CardImageHandle::Svg(handle)) => {
                            use iced::advanced::svg::{Renderer as SvgRenderer, Svg};
                            let nat = SvgRenderer::measure_svg(renderer, handle);
                            let fit = contain_rect(content_rect,
                                Size::new(nat.width as f32, nat.height as f32));
                            SvgRenderer::draw_svg(renderer, Svg::new(handle.clone()), fit);
                        }
                        None => {}
                    }
                }
            });
        }

        // ── connection dots + pending line (above cards) ───────────────────
        let is_connecting = self.pending_conn.is_some();
        let show_dots_on = |card_id: usize| -> bool {
            if is_connecting {
                // While connecting, show dots on all cards so user sees targets
                self.pending_conn.map_or(true, |(fid, _)| card_id != fid)
            } else {
                // Normally only show dots on the hovered card
                self.hovered_card == Some(card_id)
            }
        };

        let needs_overlay = self.hovered_card.is_some() || is_connecting;
        if needs_overlay {
            renderer.with_layer(bounds, |renderer| {
                let mut frame = Frame::new(&*renderer, bounds.size());
                frame.with_save(|frame| {
                    apply_zoom(frame);

                    // Side connection dots
                    for card in self.cards.iter() {
                        if show_dots_on(card.id) {
                            draw_side_dots(frame, card, self.offset, card.color, self.accent_color);
                        }
                    }

                    // Pending animated connection line
                    if let Some((from_id, from_side)) = self.pending_conn {
                        if let Some(card) = self.cards.iter().find(|c| c.id == from_id) {
                            draw_pending_line(
                                frame, card, from_side,
                                self.pending_cursor, self.offset,
                                card.color, self.conn_anim_phase,
                            );
                        }
                    }
                });
                GeoRenderer::draw_geometry(renderer, frame.into_geometry());
            });
        }
    }

    fn state(&self) -> widget::tree::State { widget::tree::State::None }
    fn children(&self) -> Vec<widget::Tree> { vec![] }
    fn diff(&self, _tree: &mut widget::Tree) {}
}

impl<'a, Message: 'a> From<CardLayer<'a>> for Element<'a, Message> {
    fn from(layer: CardLayer<'a>) -> Self {
        Element::new(layer)
    }
}

// ── Connection rendering ───────────────────────────────────────────────────────

/// Screen-space center of a card's side (used for side-dot indicators and pending lines).
fn side_screen_pos(card: &Card, side: CardSide, offset: Vector) -> Point {
    let sx = card.current_position.x + offset.x;
    let sy = card.current_position.y + offset.y;
    match side {
        CardSide::Top    => Point::new(sx + card.width / 2.0, sy),
        CardSide::Bottom => Point::new(sx + card.width / 2.0, sy + card.height),
        CardSide::Left   => Point::new(sx, sy + card.height / 2.0),
        CardSide::Right  => Point::new(sx + card.width, sy + card.height / 2.0),
    }
}

/// Screen-space attachment point using a direct perpendicular offset (for animations).
fn side_pos_with_perp(card: &Card, side: CardSide, offset: Vector, perp: f32) -> Point {
    let sx = card.current_position.x + offset.x;
    let sy = card.current_position.y + offset.y;
    match side {
        CardSide::Top    => Point::new(sx + card.width / 2.0 + perp, sy),
        CardSide::Bottom => Point::new(sx + card.width / 2.0 + perp, sy + card.height),
        CardSide::Left   => Point::new(sx, sy + card.height / 2.0 + perp),
        CardSide::Right  => Point::new(sx + card.width, sy + card.height / 2.0 + perp),
    }
}

/// Compute the target perpendicular offset for a given slot in a bundle (fallback for first frame).
fn conn_perp_target(card: &Card, side: CardSide, slot: usize, total: usize) -> f32 {
    let side_len = match side {
        CardSide::Top | CardSide::Bottom => card.width,
        CardSide::Left | CardSide::Right => card.height,
    };
    crate::card::conn_bundle_offset(slot, total, side_len)
}

/// Build a map from (card_id, side) → ordered list of connection indices that attach there.
/// Connections are sorted by the other endpoint's position along the side's tangent axis so that
/// bundle lines never cross each other when cards move.
fn bundle_slots(connections: &[Connection], cards: &[Card]) -> HashMap<(usize, CardSide), Vec<usize>> {
    let card_center = |id: usize| -> (f32, f32) {
        cards.iter().find(|c| c.id == id)
            .map(|c| (c.current_position.x + c.width / 2.0, c.current_position.y + c.height / 2.0))
            .unwrap_or((0.0, 0.0))
    };
    let mut map: HashMap<(usize, CardSide), Vec<usize>> = HashMap::new();
    for (i, conn) in connections.iter().enumerate() {
        map.entry((conn.from_card, conn.from_side)).or_default().push(i);
        map.entry((conn.to_card,   conn.to_side  )).or_default().push(i);
    }
    // Sort each group so slots are assigned in spatial order → no crossing lines.
    for ((card_id, side), indices) in map.iter_mut() {
        indices.sort_by(|&a, &b| {
            let other_a = if connections[a].from_card == *card_id { connections[a].to_card } else { connections[a].from_card };
            let other_b = if connections[b].from_card == *card_id { connections[b].to_card } else { connections[b].from_card };
            let (ax, ay) = card_center(other_a);
            let (bx, by) = card_center(other_b);
            let key_a = match side { CardSide::Top | CardSide::Bottom => ax, CardSide::Left | CardSide::Right => ay };
            let key_b = match side { CardSide::Top | CardSide::Bottom => bx, CardSide::Left | CardSide::Right => by };
            key_a.partial_cmp(&key_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    map
}

fn cubic_bezier(t: f32, p0: Point, p1: Point, p2: Point, p3: Point) -> Point {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    Point::new(
        u * u2 * p0.x + 3.0 * u2 * t * p1.x + 3.0 * u * t2 * p2.x + t * t2 * p3.x,
        u * u2 * p0.y + 3.0 * u2 * t * p1.y + 3.0 * u * t2 * p2.y + t * t2 * p3.y,
    )
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: a.a + (b.a - a.a) * t }
}

/// Approximate arc length of the cubic bezier by sampling.
fn bezier_arc_length(p0: Point, p1: Point, p2: Point, p3: Point) -> f32 {
    let samples = 100usize;
    let mut len = 0.0_f32;
    let mut prev = p0;
    for i in 1..=samples {
        let t = i as f32 / samples as f32;
        let pt = cubic_bezier(t, p0, p1, p2, p3);
        let dx = pt.x - prev.x;
        let dy = pt.y - prev.y;
        len += (dx * dx + dy * dy).sqrt();
        prev = pt;
    }
    len
}

fn draw_connection(
    frame: &mut Frame,
    from: &Card, from_perp: f32,
    to: &Card, to_perp: f32,
    conn: &Connection, offset: Vector,
) {
    let p0 = side_pos_with_perp(from, conn.from_side, offset, from_perp);
    let p3 = side_pos_with_perp(to,   conn.to_side,   offset, to_perp);

    let dist = ((p0.x - p3.x).powi(2) + (p0.y - p3.y).powi(2)).sqrt();
    let ctrl = (dist * 0.45).max(70.0);

    let (dx0, dy0) = conn.from_side.outward();
    let (dx3, dy3) = conn.to_side.outward();
    let p1 = Point::new(p0.x + dx0 * ctrl, p0.y + dy0 * ctrl);
    let p2 = Point::new(p3.x + dx3 * ctrl, p3.y + dy3 * ctrl);

    let col_a = Color { a: 0.85, ..from.color };
    let col_b = Color { a: 0.85, ..to.color };

    let arc_len = bezier_arc_length(p0, p1, p2, p3);

    match conn.line_style {
        LineStyle::Solid => {
            let segments = 40usize;
            for i in 0..segments {
                let t0 = i as f32 / segments as f32;
                let t1 = (i + 1) as f32 / segments as f32;
                let q0 = cubic_bezier(t0, p0, p1, p2, p3);
                let q1 = cubic_bezier(t1, p0, p1, p2, p3);
                let col = lerp_color(col_a, col_b, (t0 + t1) * 0.5);
                frame.stroke(&Path::line(q0, q1), Stroke::default().with_color(col).with_width(2.5));
            }
        }
        LineStyle::Dashed => {
            // Physical dash/gap in pixels — density adapts to curve length
            let dash_px = 12.0_f32;
            let gap_px  =  8.0_f32;
            let period  = dash_px + gap_px;
            // Fine sampling: ~1 step per pixel for smooth dash edges
            let steps = (arc_len as usize).max(40);
            let mut cumulative = 0.0_f32;
            let mut prev = p0;
            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let pt = cubic_bezier(t, p0, p1, p2, p3);
                let dx = pt.x - prev.x;
                let dy = pt.y - prev.y;
                let seg_len = (dx * dx + dy * dy).sqrt();
                // Draw if we're inside a dash portion
                if cumulative % period < dash_px {
                    let col = lerp_color(col_a, col_b, t);
                    frame.stroke(&Path::line(prev, pt), Stroke::default().with_color(col).with_width(2.5));
                }
                cumulative += seg_len;
                prev = pt;
            }
        }
        LineStyle::Dotted => {
            // Fixed physical gap between dot centres; count adapts to curve length
            let dot_gap = 10.0_f32;
            let dots = ((arc_len / dot_gap).round() as usize).max(2);
            for i in 0..=dots {
                let t = i as f32 / dots as f32;
                let q = cubic_bezier(t, p0, p1, p2, p3);
                let col = lerp_color(col_a, col_b, t);
                frame.fill(&Path::circle(q, 2.5), col);
            }
        }
    }

    // Arrowheads — wings always point outward from card so they're visible
    if conn.arrow_to {
        draw_arrowhead(frame, p3, (-dx3, -dy3), col_b);
    }
    if conn.arrow_from {
        // Negate so wings point outward (away from from_card), matching arrow_to style
        draw_arrowhead(frame, p0, (-dx0, -dy0), col_a);
    }

    // Small filled circle at each endpoint (connection point marker)
    frame.fill(&Path::circle(p0, 3.5), Color { a: 0.9, ..from.color });
    frame.fill(&Path::circle(p3, 3.5), Color { a: 0.9, ..to.color });
}

fn draw_arrowhead(frame: &mut Frame, tip: Point, dir: (f32, f32), color: Color) {
    let size  = 11.0_f32;
    let angle = 30_f32.to_radians();
    let (cos_a, sin_a) = (angle.cos(), angle.sin());
    let (ivx, ivy) = (-dir.0, -dir.1);
    let wing1 = Point::new(
        tip.x + (ivx * cos_a - ivy * sin_a) * size,
        tip.y + (ivx * sin_a + ivy * cos_a) * size,
    );
    let wing2 = Point::new(
        tip.x + (ivx * cos_a + ivy * sin_a) * size,
        tip.y + (-ivx * sin_a + ivy * cos_a) * size,
    );
    let s = Stroke::default().with_color(color).with_width(2.0);
    frame.stroke(&Path::line(tip, wing1), s.clone());
    frame.stroke(&Path::line(tip, wing2), s);
}

fn draw_side_dots(frame: &mut Frame, card: &Card, offset: Vector, card_color: Color, _accent: Color) {
    for &side in CardSide::all() {
        let sp = side_screen_pos(card, side, offset);
        frame.fill(&Path::circle(sp, 5.5), Color { a: 0.9, ..card_color });
        frame.stroke(
            &Path::circle(sp, 5.5),
            Stroke::default().with_color(Color { a: 0.6, r: 1.0, g: 1.0, b: 1.0 }).with_width(1.5),
        );
    }
}

fn draw_pending_line(
    frame: &mut Frame,
    from_card: &Card,
    from_side: CardSide,
    cursor: Point,
    offset: Vector,
    color: Color,
    anim_phase: f32,
) {
    let p0 = side_screen_pos(from_card, from_side, offset);
    let p3 = cursor;

    let dist = ((p0.x - p3.x).powi(2) + (p0.y - p3.y).powi(2)).sqrt();
    let ctrl = (dist * 0.45).max(60.0);
    let (dx0, dy0) = from_side.outward();
    let p1 = Point::new(p0.x + dx0 * ctrl, p0.y + dy0 * ctrl);
    let p2 = p3;

    let segments = 36usize;
    let dash_len  = 5usize;  // segments that are "on"
    let gap_len   = 3usize;  // segments that are "off"
    let period    = dash_len + gap_len;
    let phase_off = (anim_phase * period as f32) as usize;

    let col = Color { a: 0.80, ..color };

    for i in 0..segments {
        if (i + phase_off) % period < dash_len {
            let t0 = i as f32 / segments as f32;
            let t1 = (i + 1) as f32 / segments as f32;
            let q0 = cubic_bezier(t0, p0, p1, p2, p3);
            let q1 = cubic_bezier(t1, p0, p1, p2, p3);
            frame.stroke(&Path::line(q0, q1), Stroke::default().with_color(col).with_width(2.5));
        }
    }

    // Highlight origin dot
    frame.fill(&Path::circle(p0, 6.0), Color { a: 0.95, ..color });
    frame.stroke(
        &Path::circle(p0, 6.0),
        Stroke::default().with_color(Color { a: 0.7, r: 1.0, g: 1.0, b: 1.0 }).with_width(1.5),
    );
}

// ── Path helpers (mirrors dot_grid.rs) ────────────────────────────────────────

fn rounded_rectangle(rect: Rectangle, radius: f32) -> Path {
    let mut builder = Builder::new();
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let r = radius.min(w / 2.0).min(h / 2.0);
    builder.move_to(Point::new(x + r, y));
    builder.line_to(Point::new(x + w - r, y));
    builder.arc_to(Point::new(x + w, y),     Point::new(x + w, y + r),     r);
    builder.line_to(Point::new(x + w, y + h - r));
    builder.arc_to(Point::new(x + w, y + h), Point::new(x + w - r, y + h), r);
    builder.line_to(Point::new(x + r, y + h));
    builder.arc_to(Point::new(x, y + h),     Point::new(x, y + h - r),     r);
    builder.line_to(Point::new(x, y + r));
    builder.arc_to(Point::new(x, y),         Point::new(x + r, y),         r);
    builder.close();
    builder.build()
}

/// Rounded rectangle with rounded top corners only (for the top bar).
/// Scale `natural_size` to fit inside `container` preserving aspect ratio (letterbox/pillarbox).
fn contain_rect(container: Rectangle, natural_size: Size) -> Rectangle {
    if natural_size.width <= 0.0 || natural_size.height <= 0.0 {
        return container;
    }
    let scale = (container.width / natural_size.width)
        .min(container.height / natural_size.height);
    let w = natural_size.width  * scale;
    let h = natural_size.height * scale;
    Rectangle {
        x:      container.x + (container.width  - w) / 2.0,
        y:      container.y + (container.height - h) / 2.0,
        width:  w,
        height: h,
    }
}

fn rounded_rectangle_top(rect: Rectangle, radius: f32) -> Path {
    let mut builder = Builder::new();
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let r = radius.min(w / 2.0).min(h / 2.0);
    builder.move_to(Point::new(x + r, y));
    builder.line_to(Point::new(x + w - r, y));
    builder.arc_to(Point::new(x + w, y), Point::new(x + w, y + r), r);
    builder.line_to(Point::new(x + w, y + h));
    builder.line_to(Point::new(x,     y + h));
    builder.line_to(Point::new(x, y + r));
    builder.arc_to(Point::new(x, y), Point::new(x + r, y), r);
    builder.close();
    builder.build()
}
