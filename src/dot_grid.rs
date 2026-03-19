use iced::widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke, Frame, path::Builder, gradient};
use iced::{Color, Element, Length, Point, Rectangle, Theme as IcedTheme, mouse, Vector};
use std::cell::Cell;
use crate::card::{Card, CardSide, Connection};
use std::collections::{HashMap, HashSet};


pub struct DotGridState {
    last_cursor_pos: Option<Point>,
    is_panning: bool,
    pan_start: Option<Point>,
    dragging_card: Option<usize>,
    drag_offset: Option<Point>,
    resizing_card: Option<usize>,
    resize_start_size: Option<(f32, f32)>,
    resize_start_pos: Option<Point>,
    hovered_card: Option<usize>,
    selecting_text_card: Option<usize>,
    // Box selection
    box_select_start: Option<Point>,
    box_select_end: Option<Point>,
    // Card connection drag
    connecting_from: Option<(usize, CardSide)>,
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
            box_select_start: None,
            box_select_end: None,
            connecting_from: None,
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
    /// One cache per card, parallel to `cards` Vec — gives each card its own
    /// Geometry layer so Iced composites them in z-order without type-batching.
    card_caches: Vec<Cache>,
    offset: Vector,
    exclude_region: Option<Rectangle>,
    effect_enabled: bool,
    cards: Vec<Card>,
    card_background: Color,
    card_border: Color,
    card_text: Color,
    accent_color: Color,
    font: iced::Font,
    font_size: f32,
    debug_mode: bool,
    /// Counter used when generating new card IDs (so loaded cards don't collide)
    next_card_id: usize,
    /// When true, all canvas input events are ignored (modal open etc.)
    pub blocked: bool,
    /// Card IDs currently selected via box selection
    selected_cards: HashSet<usize>,
    /// Single card selected (not via box selection) — for toolbar + Delete key
    single_selected_card: Option<usize>,
    /// Currently hovered card — updated via Cell since Program::update takes &self
    hovered_card: Cell<Option<usize>>,
    /// Active connections between cards
    connections: Vec<Connection>,
    /// Card+side that a pending connection is being dragged from (Cell for Program::update &self)
    pending_conn_from: Cell<Option<(usize, CardSide)>>,
    /// Current cursor position while connecting (Cell for Program::update &self)
    pending_conn_cursor: Cell<Point>,
    /// Animation phase [0,1) for the dashed pending-connection line
    conn_anim_phase: Cell<f32>,
    /// Index of the currently hovered connection (for toolbar display)
    hovered_conn: Cell<Option<usize>>,
    /// Screen-space rect of the connection toolbar pill (set by view() each frame)
    toolbar_region: Cell<Option<Rectangle>>,
    /// Zoom level (1.0 = 100%, zoomed around viewport center)
    zoom: f32,
    /// Viewport center in canvas-local coords (updated each frame for conn_screen_midpoint)
    viewport_center: Cell<Point>,
    /// Animated perpendicular offsets for each connection's endpoints.
    /// Key: (from_card, from_side, to_card, to_side) → (from_perp, to_perp).
    /// Updated each tick so connection bundle lines animate smoothly when slots reorder.
    conn_slot_anim: HashMap<(usize, CardSide, usize, CardSide), (f32, f32)>,
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
            card_caches: Vec::new(),
            offset: Vector::new(0.0, 0.0),
            exclude_region: None,
            effect_enabled: true,
            cards: Vec::new(),
            card_background: Color::WHITE,
            card_border: Color::from_rgb8(200, 200, 200),
            card_text: Color::from_rgb8(51, 51, 51),
            accent_color: Color::from_rgb8(124, 92, 252),
            font: iced::Font::MONOSPACE,
            font_size: 14.0,
            debug_mode: false,
            next_card_id: 0,
            blocked: false,
            selected_cards: HashSet::new(),
            single_selected_card: None,
            hovered_card: Cell::new(None),
            connections: Vec::new(),
            pending_conn_from: Cell::new(None),
            pending_conn_cursor: Cell::new(Point::ORIGIN),
            conn_anim_phase: Cell::new(0.0),
            hovered_conn: Cell::new(None),
            toolbar_region: Cell::new(None),
            zoom: 1.0,
            viewport_center: Cell::new(Point::ORIGIN),
            conn_slot_anim: HashMap::new(),
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
        self.clear_all_card_caches();
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
        self.clear_all_card_caches();
    }

    pub fn zoom(&self) -> f32 { self.zoom }

    pub fn set_zoom(&mut self, zoom: f32) {
        let clamped = zoom.clamp(0.4, 5.0);
        if (clamped - self.zoom).abs() > 1e-6 {
            self.zoom = clamped;
            self.static_cache.clear();
            self.clear_all_card_caches();
        }
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
        self.clear_all_card_caches();
    }

    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
        self.clear_all_card_caches();
    }

    pub fn set_next_card_id(&mut self, id: usize) {
        self.next_card_id = id;
    }

    pub fn next_card_id(&self) -> usize {
        self.next_card_id
    }

    /// Move a card to the end of the draw list so it renders on top.
    pub fn bring_card_to_front(&mut self, card_id: usize) {
        if let Some(idx) = self.cards.iter().position(|c| c.id == card_id) {
            let card = self.cards.remove(idx);
            let cache = self.card_caches.remove(idx);
            self.cards.push(card);
            self.card_caches.push(cache);
        }
    }

    /// Replace the multi-selection set (used by box selection).
    pub fn set_selected_cards(&mut self, ids: HashSet<usize>) {
        self.selected_cards = ids;
        self.clear_all_card_caches();
    }

    /// Clear the multi-selection set.
    pub fn clear_selected_cards(&mut self) {
        if !self.selected_cards.is_empty() {
            self.selected_cards.clear();
            self.clear_all_card_caches();
        }
    }

    /// Set (or clear) the single-card selection indicator.
    pub fn set_single_selected_card(&mut self, id: Option<usize>) {
        if self.single_selected_card != id {
            // Only invalidate the two affected cards, not all of them.
            let old = self.single_selected_card;
            self.single_selected_card = id;
            if let Some(old_id) = old {
                self.invalidate_card_cache(old_id);
            }
            if let Some(new_id) = id {
                self.invalidate_card_cache(new_id);
            }
        }
    }

    fn clear_all_card_caches(&self) {
        for c in &self.card_caches {
            c.clear();
        }
    }

    fn invalidate_card_cache(&self, card_id: usize) {
        if let Some(pos) = self.cards.iter().position(|c| c.id == card_id) {
            self.card_caches[pos].clear();
        }
    }

    /// Find a non-overlapping snapped spawn position near `candidate`.
    fn find_spawn_position(&self, candidate: Point) -> Point {
        let mut pos = candidate;
        for _ in 0..20 {
            let overlapping = self.cards.iter().any(|c| {
                let cx = c.current_position.x;
                let cy = c.current_position.y;
                (pos.x - cx).abs() < self.dot_spacing * 3.0
                    && (pos.y - cy).abs() < self.dot_spacing * 3.0
            });
            if !overlapping {
                break;
            }
            pos = Point::new(
                pos.x + self.dot_spacing * 2.0,
                pos.y + self.dot_spacing * 2.0,
            );
        }
        pos
    }

    pub fn add_card(&mut self, screen_position: Point, color: Color) -> usize {
        let id = self.next_card_id;
        self.next_card_id += 1;
        let world_position = Point::new(
            screen_position.x - self.offset.x,
            screen_position.y - self.offset.y,
        );
        let snapped_position = Card::snap_to_grid(world_position, self.dot_spacing);
        let spawn_position = self.find_spawn_position(snapped_position);
        let mut card = Card::new(id, spawn_position);
        if self.debug_mode {
            println!("DEBUG: Setting font on new card {}: font_size={}", id, self.font_size);
        }
        card.content.set_font(self.font, self.font_size);
        card.color = color;
        self.cards.push(card);
        self.card_caches.push(Cache::new());
        if self.debug_mode {
            println!("Added card {} at world position {:?}", id, spawn_position);
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
        let id = self.next_card_id;
        self.next_card_id += 1;
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
        self.card_caches.push(Cache::new());
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
        let id = self.next_card_id;
        self.next_card_id += 1;
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
        card.width = width;
        card.height = height;
        card.target_width = width;
        card.target_height = height;
        self.cards.push(card);
        self.card_caches.push(Cache::new());
        id
    }

    pub fn clear_cards_cache(&mut self) {
        self.clear_all_card_caches();
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn cards_mut(&mut self) -> &mut [Card] {
        &mut self.cards
    }
    
    pub fn load_cards(&mut self, cards: Vec<Card>) {
        let n = cards.len();
        self.cards = cards;
        self.card_caches = (0..n).map(|_| Cache::new()).collect();
    }

    pub fn delete_card(&mut self, card_id: usize) {
        if let Some(pos) = self.cards.iter().position(|c| c.id == card_id) {
            self.cards.remove(pos);
            self.card_caches.remove(pos);
        }
        self.connections.retain(|c| c.from_card != card_id && c.to_card != card_id);
    }

    pub fn connections(&self) -> &[Connection] { &self.connections }
    pub fn connections_mut(&mut self) -> &mut Vec<Connection> { &mut self.connections }
    pub fn add_connection(&mut self, conn: Connection) { self.connections.push(conn); }
    pub fn set_connections(&mut self, conns: Vec<Connection>) { self.connections = conns; }
    pub fn remove_connections_for_card(&mut self, card_id: usize) {
        self.connections.retain(|c| c.from_card != card_id && c.to_card != card_id);
    }
    pub fn pending_conn(&self) -> Option<(usize, CardSide)> { self.pending_conn_from.get() }
    pub fn pending_cursor(&self) -> Point { self.pending_conn_cursor.get() }
    pub fn conn_anim_phase(&self) -> f32 { self.conn_anim_phase.get() }
    pub fn advance_conn_anim(&self, dt: f32) {
        let phase = (self.conn_anim_phase.get() + dt * 1.5) % 1.0;
        self.conn_anim_phase.set(phase);
    }
    pub fn hovered_conn_idx(&self) -> Option<usize> { self.hovered_conn.get() }
    pub fn set_toolbar_region(&self, rect: Option<Rectangle>) { self.toolbar_region.set(rect); }
    /// Returns the screen-space midpoint of the hovered connection's bezier curve.
    pub fn hovered_conn_toolbar_pos(&self) -> Option<Point> {
        let idx  = self.hovered_conn.get()?;
        let conn = self.connections.get(idx)?;
        let from = self.cards.iter().find(|c| c.id == conn.from_card)?;
        let to   = self.cards.iter().find(|c| c.id == conn.to_card)?;
        let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
        let (from_perp, to_perp) = self.animated_perps(conn, idx, &key);
        let p0 = side_pos_with_perp(from, conn.from_side, self.offset, from_perp);
        let p3 = side_pos_with_perp(to,   conn.to_side,   self.offset, to_perp);
        let dist = ((p0.x - p3.x).powi(2) + (p0.y - p3.y).powi(2)).sqrt();
        let ctrl = (dist * 0.45).max(70.0);
        let (dx0, dy0) = conn.from_side.outward();
        let (dx3, dy3) = conn.to_side.outward();
        let p1 = Point::new(p0.x + dx0 * ctrl, p0.y + dy0 * ctrl);
        let p2 = Point::new(p3.x + dx3 * ctrl, p3.y + dy3 * ctrl);
        let mid = cubic_bezier_pt(0.5, p0, p1, p2, p3);
        Some(self.apply_zoom_to_point(mid))
    }
    /// Returns the hovered connection (by value) if any.
    pub fn hovered_connection(&self) -> Option<crate::card::Connection> {
        let idx = self.hovered_conn.get()?;
        self.connections.get(idx).copied()
    }
    /// Returns the screen-space bezier midpoint for a given connection (zoom-adjusted).
    pub fn conn_screen_midpoint(&self, conn: &crate::card::Connection) -> Option<Point> {
        let idx  = self.connections.iter().position(|c| c == conn)?;
        let from = self.cards.iter().find(|c| c.id == conn.from_card)?;
        let to   = self.cards.iter().find(|c| c.id == conn.to_card)?;
        let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
        let (from_perp, to_perp) = self.animated_perps(conn, idx, &key);
        let p0 = side_pos_with_perp(from, conn.from_side, self.offset, from_perp);
        let p3 = side_pos_with_perp(to,   conn.to_side,   self.offset, to_perp);
        let dist = ((p0.x - p3.x).powi(2) + (p0.y - p3.y).powi(2)).sqrt();
        let ctrl = (dist * 0.45).max(70.0);
        let (dx0, dy0) = conn.from_side.outward();
        let (dx3, dy3) = conn.to_side.outward();
        let p1 = Point::new(p0.x + dx0 * ctrl, p0.y + dy0 * ctrl);
        let p2 = Point::new(p3.x + dx3 * ctrl, p3.y + dy3 * ctrl);
        let mid = cubic_bezier_pt(0.5, p0, p1, p2, p3);
        Some(self.apply_zoom_to_point(mid))
    }

    /// Convert a "zoom-1 screen pos" (world + offset) to actual screen pos accounting for zoom.
    fn apply_zoom_to_point(&self, p: Point) -> Point {
        let vc = self.viewport_center.get();
        Point::new(vc.x + (p.x - vc.x) * self.zoom, vc.y + (p.y - vc.y) * self.zoom)
    }

    /// Inverse-transform a cursor position from actual screen space to zoom-1 canvas space.
    pub fn inverse_zoom_point(&self, p: Point) -> Point {
        if self.zoom == 1.0 { return p; }
        let vc = self.viewport_center.get();
        Point::new(vc.x + (p.x - vc.x) / self.zoom, vc.y + (p.y - vc.y) / self.zoom)
    }

    pub fn update_card_animation(&mut self, delta_time: f32) {
        for (card, cache) in self.cards.iter_mut().zip(self.card_caches.iter_mut()) {
            if card.update_animation(delta_time) {
                cache.clear();
            }
        }
    }

    /// Advance the animated perpendicular offsets for connection bundles toward their targets.
    /// When `animated` is false the offsets snap instantly (animations disabled).
    pub fn update_conn_slot_anim(&mut self, dt: f32, animated: bool) {
        let slots = bundle_slots(&self.connections, &self.cards);

        // Collect targets before mutably borrowing conn_slot_anim
        let mut targets: Vec<((usize, CardSide, usize, CardSide), (f32, f32))> = Vec::new();
        for (conn_idx, conn) in self.connections.iter().enumerate() {
            let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
            let fg = slots.get(&(conn.from_card, conn.from_side)).map(|v| v.as_slice()).unwrap_or(&[]);
            let tg = slots.get(&(conn.to_card, conn.to_side)).map(|v| v.as_slice()).unwrap_or(&[]);
            let from_slot = fg.iter().position(|&i| i == conn_idx).unwrap_or(0);
            let to_slot   = tg.iter().position(|&i| i == conn_idx).unwrap_or(0);
            let fc = self.cards.iter().find(|c| c.id == conn.from_card);
            let tc = self.cards.iter().find(|c| c.id == conn.to_card);
            if let (Some(fc), Some(tc)) = (fc, tc) {
                let tf = conn_perp_target(fc, conn.from_side, from_slot, fg.len());
                let tt = conn_perp_target(tc, conn.to_side,   to_slot,   tg.len());
                targets.push((key, (tf, tt)));
            }
        }

        // Remove entries for deleted connections
        let valid_keys: HashSet<_> = self.connections.iter()
            .map(|c| (c.from_card, c.from_side, c.to_card, c.to_side))
            .collect();
        self.conn_slot_anim.retain(|k, _| valid_keys.contains(k));

        let factor = if animated {
            let speed = 14.0_f32;
            1.0 - (-speed * dt).exp()
        } else {
            1.0
        };

        for (key, (tf, tt)) in targets {
            let entry = self.conn_slot_anim.entry(key).or_insert((tf, tt));
            entry.0 += (tf - entry.0) * factor;
            entry.1 += (tt - entry.1) * factor;
        }
    }

    pub fn conn_slot_anim(&self) -> &HashMap<(usize, CardSide, usize, CardSide), (f32, f32)> {
        &self.conn_slot_anim
    }

    /// Returns animated (from_perp, to_perp) for a connection, falling back to slot-based
    /// computation if the anim map doesn't have an entry yet (first frame).
    fn animated_perps(
        &self,
        conn: &crate::card::Connection,
        conn_idx: usize,
        key: &(usize, CardSide, usize, CardSide),
    ) -> (f32, f32) {
        if let Some(&perps) = self.conn_slot_anim.get(key) {
            return perps;
        }
        // Fallback: compute from bundle_slots (only needed before the first tick)
        let slots = bundle_slots(&self.connections, &self.cards);
        let fg = slots.get(&(conn.from_card, conn.from_side)).map(|v| v.as_slice()).unwrap_or(&[]);
        let tg = slots.get(&(conn.to_card,   conn.to_side  )).map(|v| v.as_slice()).unwrap_or(&[]);
        let from_slot = fg.iter().position(|&i| i == conn_idx).unwrap_or(0);
        let to_slot   = tg.iter().position(|&i| i == conn_idx).unwrap_or(0);
        let fc = self.cards.iter().find(|c| c.id == conn.from_card);
        let tc = self.cards.iter().find(|c| c.id == conn.to_card);
        let fp = fc.map(|c| conn_perp_target(c, conn.from_side, from_slot, fg.len())).unwrap_or(0.0);
        let tp = tc.map(|c| conn_perp_target(c, conn.to_side,   to_slot,   tg.len())).unwrap_or(0.0);
        (fp, tp)
    }

    pub fn dot_spacing(&self) -> f32 {
        self.dot_spacing
    }

    pub fn font(&self) -> iced::Font {
        self.font
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn offset(&self) -> Vector {
        self.offset
    }

    pub fn card_background(&self) -> Color { self.card_background }
    pub fn card_border(&self) -> Color { self.card_border }
    pub fn card_text(&self) -> Color { self.card_text }
    pub fn accent_color(&self) -> Color { self.accent_color }
    pub fn selected_cards(&self) -> &HashSet<usize> { &self.selected_cards }
    pub fn single_selected_card(&self) -> Option<usize> { self.single_selected_card }
    pub fn hovered_card(&self) -> Option<usize> { self.hovered_card.get() }

    pub fn is_editing_any_card(&self) -> bool {
        self.cards.iter().any(|card| card.is_editing)
    }

    /// Update checkbox positions for a card after rendering
    pub fn update_card_checkbox_positions(&mut self, card_id: usize) {
        use crate::text_processor::TextProcessor;
        use crate::card::CardType;

        if let Some(card) = self.cards.iter_mut().find(|c| c.id == card_id) {
            card.checkbox_positions.clear();

            if !card.is_editing {
                let content_text = card.content.text();
                let card_type = card.card_type;
                if !content_text.is_empty() {
                    let processor = TextProcessor::with_font_size(self.font_size);
                    let document = if card_type == CardType::Markdown {
                        processor.parse_full_markdown(&content_text)
                    } else {
                        processor.process(&content_text)
                    };

                    // Store positions relative to card origin (no canvas offset).
                    // The offset is applied at hit-test time so panning never invalidates them.
                    let top_bar_height = 30.0;
                    let text_x = 10.0;
                    let text_y = top_bar_height + 10.0;
                    let mut current_y = 0.0f32;

                    for line in &document.lines {
                        if line.is_rule {
                            current_y += line.spacing_before + 8.0 + line.spacing_after;
                            continue;
                        }
                        if line.is_empty() { current_y += 8.0; continue; }
                        current_y += line.spacing_before;

                        let max_size = line.segments.iter()
                            .map(|seg| seg.style.size)
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(14.0);
                        let line_height = 21.0 * (max_size / 14.0);

                        if let Some(checkbox) = &line.checkbox {
                            let checkbox_size = 14.0;
                            // card-local rect
                            let checkbox_rect = Rectangle {
                                x: card.current_position.x + text_x - 20.0 + line.indent,
                                y: card.current_position.y + text_y + current_y + 2.0,
                                width: checkbox_size,
                                height: checkbox_size,
                            };
                            card.checkbox_positions.push(crate::text_renderer::CheckboxPosition {
                                rect: checkbox_rect,
                                line_index: checkbox.line_index,
                                checked: checkbox.checked,
                            });
                        }
                        current_y += line_height + line.spacing_after;
                    }
                }
            }
        }
    }

    /// Update the stored link positions for a Markdown card (called after content changes)
    pub fn update_card_link_positions(&mut self, card_id: usize) {
        use crate::text_processor::TextProcessor;
        use crate::card::CardType;

        if let Some(card) = self.cards.iter_mut().find(|c| c.id == card_id) {
            card.link_positions.clear();
            if card.card_type != CardType::Markdown || card.is_editing {
                return;
            }
            let content = card.content.text();
            if content.is_empty() { return; }

            let top_bar_height = 30.0;
            // Card-local origin (no canvas offset — applied at hit-test time)
            let text_x = card.current_position.x + 10.0;
            let text_y = card.current_position.y + top_bar_height + 10.0;

            let processor = TextProcessor::with_font_size(self.font_size);
            let document = processor.parse_full_markdown(&content);

            let mut current_y = 0.0_f32;
            for line in &document.lines {
                if line.is_rule {
                    current_y += line.spacing_before + 8.0 + line.spacing_after;
                    continue;
                }
                if line.is_empty() { current_y += 8.0; continue; }
                current_y += line.spacing_before;
                let max_size = line.segments.iter().map(|s| s.style.size)
                    .max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(14.0);
                let line_height = 21.0 * (max_size / 14.0);
                let mut current_x = 0.0_f32;
                for seg in &line.segments {
                    if let Some(url) = &seg.link_url {
                        let char_width = seg.style.size * 0.55;
                        let seg_width = seg.text.chars().count() as f32 * char_width;
                        card.link_positions.push(crate::text_renderer::LinkPosition {
                            rect: iced::Rectangle {
                                x: text_x + line.indent + current_x,
                                y: text_y + current_y,
                                width: seg_width,
                                height: line_height,
                            },
                            url: url.clone(),
                        });
                    }
                    let char_width = seg.style.size * 0.55;
                    current_x += seg.text.chars().count() as f32 * char_width;
                }
                current_y += line_height + line.spacing_after;
            }
        }
    }

    /// Find a link at the given screen position
    pub fn find_clicked_link(&self, screen_pos: Point) -> Option<String> {
        for card in &self.cards {
            if !card.is_editing {
                for lp in &card.link_positions {
                    // link_positions are card-local; add canvas offset for screen test
                    let screen_rect = Rectangle {
                        x: lp.rect.x + self.offset.x,
                        y: lp.rect.y + self.offset.y,
                        ..lp.rect
                    };
                    if screen_rect.contains(screen_pos) {
                        return Some(lp.url.clone());
                    }
                }
            }
        }
        None
    }

    /// Check if a point clicks on a checkbox
    pub fn find_clicked_checkbox(&self, screen_pos: Point) -> Option<(usize, usize)> {
        for card in &self.cards {
            if !card.is_editing {
                for cp in &card.checkbox_positions {
                    // checkbox_positions are card-local; add canvas offset for screen test
                    let screen_rect = Rectangle {
                        x: cp.rect.x + self.offset.x,
                        y: cp.rect.y + self.offset.y,
                        ..cp.rect
                    };
                    if screen_rect.contains(screen_pos) {
                        return Some((card.id, cp.line_index));
                    }
                }
            }
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
        // Always fill the background first — on Windows the window may not
        // default to a dark background so we must paint it explicitly.
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            iced::Size::new(bounds.width, bounds.height),
            self.background_color,
        );

        let z  = self.zoom;
        let vs = self.dot_spacing * z; // visual spacing between dots in screen pixels
        let cx = bounds.width  / 2.0;
        let cy = bounds.height / 2.0;

        // Phase of the first dot column/row in screen space, derived from the zoom formula.
        // screen_pos = cx + (unscaled_pos - cx) * z, where unscaled_pos = k*dot_spacing + offset
        // → screen_pos = cx*(1-z) + (k*dot_spacing + offset)*z = cx*(1-z) + k*vs + offset*z
        // phase = (cx*(1-z) + offset*z) mod vs
        let fox = (cx * (1.0 - z) + self.offset.x * z).rem_euclid(vs);
        let foy = (cy * (1.0 - z) + self.offset.y * z).rem_euclid(vs);

        let cols = (bounds.width  / vs) as i32 + 2;
        let rows = (bounds.height / vs) as i32 + 2;

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * vs + fox;
                let y = row as f32 * vs + foy;

                // Skip dots in the exclude region
                if !self.is_in_exclude_region(x, y) {
                    frame.fill(&Path::circle(Point::new(x, y), self.dot_radius), self.dot_color);
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


/// Returns the screen-space center of a card's side (used for connection-dot hit-tests).
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

/// Screen-space attachment point with bundle offset applied (matches card_layer rendering).
fn bundled_side_pos(card: &Card, side: CardSide, offset: Vector, slot: usize, total: usize) -> Point {
    side_pos_with_perp(card, side, offset, conn_perp_target(card, side, slot, total))
}

/// Compute the target perpendicular offset for a given slot in a bundle.
fn conn_perp_target(card: &Card, side: CardSide, slot: usize, total: usize) -> f32 {
    let side_len = match side {
        CardSide::Top | CardSide::Bottom => card.width,
        CardSide::Left | CardSide::Right => card.height,
    };
    crate::card::conn_bundle_offset(slot, total, side_len)
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

/// Build a map from (card_id, side) → ordered list of connection indices that attach there.
/// Connections are sorted by the other endpoint's position along the side's tangent axis so that
/// bundle lines never cross each other when cards move.
fn bundle_slots(connections: &[crate::card::Connection], cards: &[Card]) -> HashMap<(usize, CardSide), Vec<usize>> {
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

fn cubic_bezier_pt(t: f32, p0: Point, p1: Point, p2: Point, p3: Point) -> Point {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    Point::new(
        u * u2 * p0.x + 3.0 * u2 * t * p1.x + 3.0 * u * t2 * p2.x + t * t2 * p3.x,
        u * u2 * p0.y + 3.0 * u2 * t * p1.y + 3.0 * u * t2 * p2.y + t * t2 * p3.y,
    )
}

/// Returns true if `cursor` is within `threshold` pixels of the bezier connection curve.
fn connection_hit(
    from: &Card, from_perp: f32,
    to: &Card, to_perp: f32,
    conn: &crate::card::Connection, offset: Vector, cursor: Point, threshold: f32,
) -> bool {
    let p0 = side_pos_with_perp(from, conn.from_side, offset, from_perp);
    let p3 = side_pos_with_perp(to,   conn.to_side,   offset, to_perp);
    let dist = ((p0.x - p3.x).powi(2) + (p0.y - p3.y).powi(2)).sqrt();
    let ctrl = (dist * 0.45).max(70.0);
    let (dx0, dy0) = conn.from_side.outward();
    let (dx3, dy3) = conn.to_side.outward();
    let p1 = Point::new(p0.x + dx0 * ctrl, p0.y + dy0 * ctrl);
    let p2 = Point::new(p3.x + dx3 * ctrl, p3.y + dy3 * ctrl);
    let t_sq = threshold * threshold;
    for i in 0..=24 {
        let bp = cubic_bezier_pt(i as f32 / 24.0, p0, p1, p2, p3);
        let dx = cursor.x - bp.x;
        let dy = cursor.y - bp.y;
        if dx * dx + dy * dy <= t_sq { return true; }
    }
    false
}

#[derive(Debug, Clone)]
pub enum DotGridMessage {
    Pan(Vector),
    RightClick(Point),
    CardRightClickIcon(usize),
    CardTypeIconClick(usize),
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
    LinkClick(String),            // url to open
    BoxSelectEnd(Rectangle),      // box selection finished — rect in screen coords
    ConnectionComplete { from_card: usize, from_side: CardSide, to_card: usize, to_side: CardSide },
    ConnectionCancel,
    DeleteConnection { from_card: usize, from_side: CardSide, to_card: usize, to_side: CardSide },
    ConnectionClick { from_card: usize, from_side: CardSide, to_card: usize, to_side: CardSide },
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
        // All input is blocked while a modal is open
        if self.blocked {
            state.is_panning = false;
            state.pan_start = None;
            state.dragging_card = None;
            state.drag_offset = None;
            state.resizing_card = None;
            state.selecting_text_card = None;
            state.box_select_start = None;
            state.box_select_end = None;
            state.connecting_from = None;
            self.pending_conn_from.set(None);
            return (iced::widget::canvas::event::Status::Ignored, None);
        }

        // Store viewport center for conn_screen_midpoint zoom transform.
        let vc = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        self.viewport_center.set(vc);

        // Inverse-transform cursor from actual screen space to zoom-1 canvas space so
        // all existing hit-test logic (card rects, connections, etc.) works unchanged.
        let current_pos = cursor.position_in(bounds).map(|pos| {
            if self.zoom == 1.0 { return pos; }
            Point::new(vc.x + (pos.x - vc.x) / self.zoom, vc.y + (pos.y - vc.y) / self.zoom)
        });

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
                            // Check connection-dot clicks first (before normal card handling)
                            let conn_dot_r_sq = 12.0_f32 * 12.0;
                            if let Some(hid) = self.hovered_card.get() {
                                if let Some(card) = self.cards.iter().find(|c| c.id == hid) {
                                    for &side in CardSide::all() {
                                        let sp = side_screen_pos(card, side, self.offset);
                                        let dx = pos.x - sp.x;
                                        let dy = pos.y - sp.y;
                                        if dx * dx + dy * dy <= conn_dot_r_sq {
                                            state.connecting_from = Some((card.id, side));
                                            self.pending_conn_from.set(Some((card.id, side)));
                                            self.pending_conn_cursor.set(pos);
                                            self.hovered_conn.set(None);
                                            return (iced::widget::canvas::event::Status::Captured, None);
                                        }
                                    }
                                }
                            }

                            let mut is_editing_any = false;

                            // Iterate in reverse so clicks hit the top-most (highest z-order) card first
                            for card in self.cards.iter().rev() {
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
                                        // Left user icon — opens icon/colour picker
                                        let icon_bounds = Rectangle {
                                            x: screen_bounds.x + 5.0,
                                            y: screen_bounds.y + 5.0,
                                            width: 20.0,
                                            height: 20.0,
                                        };
                                        // Right type icon — opens card-type menu
                                        let type_icon_bounds = Rectangle {
                                            x: screen_bounds.x + screen_bounds.width - 26.0,
                                            y: screen_bounds.y + 5.0,
                                            width: 20.0,
                                            height: 20.0,
                                        };

                                        if type_icon_bounds.contains(pos) {
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CardTypeIconClick(card.id)),
                                            );
                                        } else if icon_bounds.contains(pos) {
                                            // Left icon left-click → open icon/colour picker
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::CardRightClickIcon(card.id)),
                                            );
                                        } else {
                                            // Drag the card (blocked if pinned)
                                            if !card.pinned {
                                                state.dragging_card = Some(card.id);
                                                state.drag_offset = Some(Point::new(
                                                    pos.x - screen_bounds.x,
                                                    pos.y - screen_bounds.y,
                                                ));
                                            }
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

                                        // Check if clicking on a link
                                        if let Some(url) = self.find_clicked_link(pos) {
                                            return (
                                                iced::widget::canvas::event::Status::Captured,
                                                Some(DotGridMessage::LinkClick(url)),
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

                            // Check if clicking on a connection line
                            for (idx, conn) in self.connections.iter().enumerate() {
                                let from = self.cards.iter().find(|c| c.id == conn.from_card);
                                let to   = self.cards.iter().find(|c| c.id == conn.to_card);
                                if let (Some(from), Some(to)) = (from, to) {
                                    let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
                                    let (fp, tp) = self.animated_perps(conn, idx, &key);
                                    if connection_hit(from, fp, to, tp, conn, self.offset, pos, 8.0) {
                                        return (
                                            iced::widget::canvas::event::Status::Captured,
                                            Some(DotGridMessage::ConnectionClick {
                                                from_card: conn.from_card,
                                                from_side: conn.from_side,
                                                to_card:   conn.to_card,
                                                to_side:   conn.to_side,
                                            }),
                                        );
                                    }
                                }
                            }

                            // If editing and clicked outside all cards, stop editing
                            if is_editing_any {
                                return (
                                    iced::widget::canvas::event::Status::Captured,
                                    Some(DotGridMessage::CardLeftClickBody(usize::MAX)),
                                );
                            }
                            // Not editing — start box selection on empty canvas
                            state.box_select_start = Some(pos);
                            state.box_select_end = Some(pos);
                            return (iced::widget::canvas::event::Status::Captured, None);
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        // Finish connection drag if active
                        if let Some((from_card, from_side)) = state.connecting_from.take() {
                            self.pending_conn_from.set(None);
                            let conn_dot_r_sq = 28.0_f32 * 28.0; // larger radius to match snap zone
                            let msg = if let Some(pos) = current_pos {
                                let mut found = None;
                                'outer: for card in self.cards.iter() {
                                    if card.id == from_card { continue; }
                                    for &side in CardSide::all() {
                                        let sp = side_screen_pos(card, side, self.offset);
                                        let dx = pos.x - sp.x;
                                        let dy = pos.y - sp.y;
                                        if dx * dx + dy * dy <= conn_dot_r_sq {
                                            found = Some(DotGridMessage::ConnectionComplete {
                                                from_card, from_side,
                                                to_card: card.id, to_side: side,
                                            });
                                            break 'outer;
                                        }
                                    }
                                }
                                found.unwrap_or(DotGridMessage::ConnectionCancel)
                            } else {
                                DotGridMessage::ConnectionCancel
                            };
                            return (iced::widget::canvas::event::Status::Captured, Some(msg));
                        }

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

                        // Finish box selection
                        if let (Some(start), Some(end)) = (state.box_select_start.take(), state.box_select_end.take()) {
                            let x = start.x.min(end.x);
                            let y = start.y.min(end.y);
                            let w = (start.x - end.x).abs();
                            let h = (start.y - end.y).abs();
                            let rect = Rectangle { x, y, width: w, height: h };
                            self.clear_all_card_caches();
                            return (
                                iced::widget::canvas::event::Status::Captured,
                                Some(DotGridMessage::BoxSelectEnd(rect)),
                            );
                        }
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
                        if let Some(pos) = current_pos {
                            // Check if right-clicking a connection line first
                            for (idx, conn) in self.connections.iter().enumerate() {
                                let from = self.cards.iter().find(|c| c.id == conn.from_card);
                                let to   = self.cards.iter().find(|c| c.id == conn.to_card);
                                if let (Some(from), Some(to)) = (from, to) {
                                    let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
                                    let (fp, tp) = self.animated_perps(conn, idx, &key);
                                    if connection_hit(from, fp, to, tp, conn, self.offset, pos, 8.0) {
                                        return (
                                            iced::widget::canvas::event::Status::Captured,
                                            Some(DotGridMessage::DeleteConnection {
                                                from_card: conn.from_card,
                                                from_side: conn.from_side,
                                                to_card:   conn.to_card,
                                                to_side:   conn.to_side,
                                            }),
                                        );
                                    }
                                }
                            }

                            // Check if clicking on a card icon — iterate reverse for correct z-order
                            for card in self.cards.iter().rev() {
                                let screen_bounds = Rectangle {
                                    x: card.current_position.x + self.offset.x,
                                    y: card.current_position.y + self.offset.y,
                                    width: card.width,
                                    height: card.height,
                                };

                                if screen_bounds.contains(pos) {
                                    let icon_bounds = Rectangle {
                                        x: screen_bounds.x + 8.0,
                                        y: screen_bounds.y + 6.0,
                                        width: 18.0,
                                        height: 18.0,
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
                        // Update pending connection cursor — snap to nearby side dots
                        if let Some((from_id, _)) = state.connecting_from {
                            if let Some(pos) = current_pos {
                                let snap_r = 26.0_f32;
                                let snap_r_sq = snap_r * snap_r;
                                let mut best_pos = pos;
                                let mut best_dist_sq = snap_r_sq;
                                for card in self.cards.iter() {
                                    if card.id == from_id { continue; }
                                    for &side in CardSide::all() {
                                        let sp = side_screen_pos(card, side, self.offset);
                                        let dx = pos.x - sp.x;
                                        let dy = pos.y - sp.y;
                                        let d_sq = dx * dx + dy * dy;
                                        if d_sq < best_dist_sq {
                                            best_dist_sq = d_sq;
                                            best_pos = sp;
                                        }
                                    }
                                }
                                self.pending_conn_cursor.set(best_pos);
                            }
                            // still fall through so hovered_card updates for connection target dots
                        }

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
                        } else if state.box_select_start.is_some() {
                            if let Some(pos) = current_pos {
                                state.box_select_end = Some(pos);
                                return (iced::widget::canvas::event::Status::Captured, None);
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
            // Iterate in reverse so the top-most (highest z-order) card is found first
            for card in self.cards.iter().rev() {
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
                if let Some(old_id) = state.hovered_card {
                    self.invalidate_card_cache(old_id);
                }
                if let Some(new_id) = new_hovered {
                    self.invalidate_card_cache(new_id);
                }
                state.hovered_card = new_hovered;
                self.hovered_card.set(new_hovered);
            }
        } else if state.hovered_card.is_some() {
            self.invalidate_card_cache(state.hovered_card.unwrap());
            self.hovered_card.set(None);
            state.hovered_card = None;
        }

        // Update hovered connection (only when not in a drag/connect mode)
        if state.connecting_from.is_none() && !state.is_panning
            && state.dragging_card.is_none() && state.resizing_card.is_none()
        {
            if let Some(pos) = current_pos {
                // If cursor is inside the connection toolbar pill, keep hover stable
                let in_toolbar = self.toolbar_region.get().map_or(false, |r| r.contains(pos));
                if !in_toolbar {
                    let mut new_hovered_conn = None;
                    for (idx, conn) in self.connections.iter().enumerate() {
                        let from = self.cards.iter().find(|c| c.id == conn.from_card);
                        let to   = self.cards.iter().find(|c| c.id == conn.to_card);
                        if let (Some(from), Some(to)) = (from, to) {
                            let key = (conn.from_card, conn.from_side, conn.to_card, conn.to_side);
                            let (fp, tp) = self.animated_perps(conn, idx, &key);
                            if connection_hit(from, fp, to, tp, conn, self.offset, pos, 8.0) {
                                new_hovered_conn = Some(idx);
                                break;
                            }
                        }
                    }
                    if self.hovered_conn.get() != new_hovered_conn {
                        self.hovered_conn.set(new_hovered_conn);
                        self.clear_all_card_caches();
                    }
                }
            } else if self.hovered_conn.get().is_some() {
                self.hovered_conn.set(None);
                self.clear_all_card_caches();
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

        // Helper: build selection-box overlay geometry (uncached)
        let selection_overlay: Option<Geometry> = {
            if let (Some(start), Some(end)) = (state.box_select_start, state.box_select_end) {
                let w = (start.x - end.x).abs();
                let h = (start.y - end.y).abs();
                if w > 3.0 || h > 3.0 {
                    let sx = start.x.min(end.x);
                    let sy = start.y.min(end.y);
                    let sel_rect = Rectangle { x: sx, y: sy, width: w, height: h };
                    let fill_col = Color::from_rgba(
                        self.accent_color.r, self.accent_color.g, self.accent_color.b, 0.10);
                    let border_col = Color::from_rgba(
                        self.accent_color.r, self.accent_color.g, self.accent_color.b, 0.80);
                    let mut sel_frame = Frame::new(renderer, bounds.size());
                    sel_frame.fill_rectangle(
                        Point::new(sx, sy), iced::Size::new(w, h), fill_col);
                    sel_frame.stroke(
                        &rounded_rectangle(sel_rect, 2.0),
                        Stroke::default().with_color(border_col).with_width(1.5),
                    );
                    Some(sel_frame.into_geometry())
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If effect is disabled, just return static layer + selection overlay.
        // Cards are rendered by CardLayer widget (with_layer per card for correct z-ordering).
        if !self.effect_enabled {
            let mut layers = vec![static_layer];
            if let Some(ov) = selection_overlay { layers.push(ov); }
            return layers;
        }

        let local_mouse = cursor.position_in(bounds);

        // If no mouse in bounds, just return static layer + selection overlay.
        if local_mouse.is_none() {
            let mut layers = vec![static_layer];
            if let Some(ov) = selection_overlay { layers.push(ov); }
            return layers;
        }

        let mouse_pos = local_mouse.unwrap();

        // Draw dynamic overlay (lines + displaced dots)
        let mut dynamic_frame = Frame::new(renderer, bounds.size());

        // Zoom-aware dot grid positions (screen space)
        let z  = self.zoom;
        let vs = self.dot_spacing * z;
        let cx = bounds.width  / 2.0;
        let cy = bounds.height / 2.0;
        let fox = (cx * (1.0 - z) + self.offset.x * z).rem_euclid(vs);
        let foy = (cy * (1.0 - z) + self.offset.y * z).rem_euclid(vs);

        let cols = (bounds.width  / vs) as i32 + 2;
        let rows = (bounds.height / vs) as i32 + 2;

        // Calculate affected range (influence radius is in screen pixels, dots also in screen pixels)
        let affect_range = (self.influence_radius.max(self.line_radius) / vs).ceil() as i32 + 1;
        let mouse_col = ((mouse_pos.x - fox) / vs).round() as i32;
        let mouse_row = ((mouse_pos.y - foy) / vs).round() as i32;

        let min_col = (mouse_col - affect_range).max(0);
        let max_col = (mouse_col + affect_range).min(cols - 1);
        let min_row = (mouse_row - affect_range).max(0);
        let max_row = (mouse_row + affect_range).min(rows - 1);

        if max_col < min_col || max_row < min_row {
            let mut layers = vec![static_layer];
            if let Some(ov) = selection_overlay { layers.push(ov); }
            return layers;
        }

        let influence_radius_sq = self.influence_radius * self.influence_radius;
        let line_radius_sq = self.line_radius * self.line_radius;

        let affected_cols = (max_col - min_col + 1) as usize;
        let affected_rows = (max_row - min_row + 1) as usize;
        let mut affected_positions: Vec<(Point, Point, f32, bool)> = Vec::with_capacity(affected_cols * affected_rows);

        // Calculate affected dot positions
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let base_x = col as f32 * vs + fox;
                let base_y = row as f32 * vs + foy;
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

        // Cards are now rendered by CardLayer widget (with_layer per card).
        let mut layers = vec![static_layer, dynamic_frame.into_geometry()];
        if let Some(ov) = selection_overlay { layers.push(ov); }
        layers
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(raw_pos) = cursor.position_in(bounds) {
        let vc = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let pos = if self.zoom == 1.0 { raw_pos } else {
            Point::new(vc.x + (raw_pos.x - vc.x) / self.zoom, vc.y + (raw_pos.y - vc.y) / self.zoom)
        };
        {
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

                    // Type icon (right side of top bar) — pointer
                    let type_icon_bounds = Rectangle {
                        x: screen_bounds.x + screen_bounds.width - 26.0,
                        y: screen_bounds.y + 5.0,
                        width: 20.0,
                        height: 20.0,
                    };
                    if type_icon_bounds.contains(pos) {
                        return mouse::Interaction::Pointer;
                    }

                    // Left (user) icon — pointer
                    let left_icon_bounds = Rectangle {
                        x: screen_bounds.x + 5.0,
                        y: screen_bounds.y + 5.0,
                        width: 20.0,
                        height: 20.0,
                    };
                    if left_icon_bounds.contains(pos) {
                        return mouse::Interaction::Pointer;
                    }

                    // Link hit-rects — pointer (positions are card-local, add offset)
                    for lp in &card.link_positions {
                        let screen_rect = Rectangle {
                            x: lp.rect.x + self.offset.x,
                            y: lp.rect.y + self.offset.y,
                            ..lp.rect
                        };
                        if screen_rect.contains(pos) {
                            return mouse::Interaction::Pointer;
                        }
                    }

                    // Checkbox hit-rects — pointer
                    for cp in &card.checkbox_positions {
                        let screen_rect = Rectangle {
                            x: cp.rect.x + self.offset.x,
                            y: cp.rect.y + self.offset.y,
                            ..cp.rect
                        };
                        if screen_rect.contains(pos) {
                            return mouse::Interaction::Pointer;
                        }
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
        } // end inner zoom block
        } // end if let Some(raw_pos)

        mouse::Interaction::default()
    }
}









