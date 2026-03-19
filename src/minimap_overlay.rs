/// MinimapOverlay — a full-window overlay widget that draws a minimap in the top-right corner.
///
/// Handles click-to-navigate and drag-to-pan. Renders above all other widgets when added
/// as the last Overlay in the view tree.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::widget::canvas::path::Builder;
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

const MM_W: f32 = 200.0;
const MM_H: f32 = 150.0;
const MM_PAD: f32 = 14.0;
const INNER: f32 = 8.0;

pub struct MinimapCard {
    pub pos: Point,
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

/// A pre-computed pair of world-space center points for a connection line.
pub struct MinimapConn {
    pub from: Point,
    pub to: Point,
}

pub struct MinimapOverlay<Message: Clone> {
    cards:       Vec<MinimapCard>,
    connections: Vec<MinimapConn>,
    offset:      Vector,
    zoom:        f32,
    window_size: Size,
    on_pan:      Box<dyn Fn(Vector) -> Message>,
}

impl<Message: Clone + 'static> MinimapOverlay<Message> {
    pub fn new(
        cards:       Vec<MinimapCard>,
        connections: Vec<MinimapConn>,
        offset:      Vector,
        zoom:        f32,
        window_size: Size,
        on_pan:      impl Fn(Vector) -> Message + 'static,
    ) -> Self {
        Self {
            cards,
            connections,
            offset,
            zoom,
            window_size,
            on_pan: Box::new(on_pan),
        }
    }

    fn mm_rect(&self) -> Rectangle {
        Rectangle {
            x:      self.window_size.width - MM_W - MM_PAD,
            y:      MM_PAD,
            width:  MM_W,
            height: MM_H,
        }
    }

    /// Returns (min_x, min_y, scale) for the current card layout.
    /// Returns None if there are no cards.
    fn world_layout(&self) -> Option<(f32, f32, f32)> {
        if self.cards.is_empty() { return None; }
        let pad = 100.0_f32;
        let mut min_x = f32::MAX; let mut min_y = f32::MAX;
        let mut max_x = f32::MIN; let mut max_y = f32::MIN;
        for c in &self.cards {
            min_x = min_x.min(c.pos.x); min_y = min_y.min(c.pos.y);
            max_x = max_x.max(c.pos.x + c.width);
            max_y = max_y.max(c.pos.y + c.height);
        }
        min_x -= pad; min_y -= pad;
        max_x += pad; max_y += pad;
        let world_w = (max_x - min_x).max(1.0);
        let world_h = (max_y - min_y).max(1.0);
        let scale_x = (MM_W - INNER * 2.0) / world_w;
        let scale_y = (MM_H - INNER * 2.0) / world_h;
        Some((min_x, min_y, scale_x.min(scale_y)))
    }

    fn world_to_mm(&self, wx: f32, wy: f32, min_x: f32, min_y: f32, scale: f32) -> Point {
        let mm = self.mm_rect();
        Point {
            x: mm.x + INNER + (wx - min_x) * scale,
            y: mm.y + INNER + (wy - min_y) * scale,
        }
    }

    fn mm_to_world(&self, sx: f32, sy: f32, min_x: f32, min_y: f32, scale: f32) -> Point {
        let mm = self.mm_rect();
        Point {
            x: (sx - mm.x - INNER) / scale + min_x,
            y: (sy - mm.y - INNER) / scale + min_y,
        }
    }
}

#[derive(Default)]
struct MinimapState {
    dragging:      bool,
    last_drag_pos: Option<Point>,
}

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer>
    for MinimapOverlay<Message>
{
    fn size(&self) -> Size<Length> { Size { width: Length::Fill, height: Length::Fill } }

    fn layout(
        &self,
        _tree:     &mut widget::Tree,
        _renderer: &iced::Renderer,
        limits:    &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn tag(&self)   -> widget::tree::Tag   { widget::tree::Tag::of::<MinimapState>() }
    fn state(&self) -> widget::tree::State { widget::tree::State::new(MinimapState::default()) }
    fn children(&self) -> Vec<widget::Tree> { vec![] }
    fn diff(&self, _tree: &mut widget::Tree) {}

    fn draw(
        &self,
        _tree:     &widget::Tree,
        renderer:  &mut iced::Renderer,
        _theme:    &iced::Theme,
        _style:    &renderer::Style,
        layout:    Layout<'_>,
        _cursor:   mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::graphics::geometry::Renderer as GeoRenderer;

        let Some((min_x, min_y, scale)) = self.world_layout() else { return };

        let mm  = self.mm_rect();
        let bounds = layout.bounds();
        // Expand clip rect so the entire minimap pill is visible even if it overlaps bounds.
        let clip = bounds.union(&mm);
        let inner_rect = Rectangle {
            x:      mm.x + 1.0,
            y:      mm.y + 1.0,
            width:  mm.width  - 2.0,
            height: mm.height - 2.0,
        };

        renderer.with_layer(clip, |renderer| {
            let mut frame = Frame::new(&*renderer, bounds.size());

            // ── Background + border ────────────────────────────────────────
            let bg_col = Color::from_rgba(0.05, 0.05, 0.08, 0.82);
            frame.fill(&rounded_rect(mm, 8.0), bg_col);
            frame.stroke(
                &rounded_rect(mm, 8.0),
                Stroke::default()
                    .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.12))
                    .with_width(1.0),
            );

            // ── Connection lines (simplified — no arrows, no dash) ─────────
            for conn in &self.connections {
                let a = self.world_to_mm(conn.from.x, conn.from.y, min_x, min_y, scale);
                let b = self.world_to_mm(conn.to.x,   conn.to.y,   min_x, min_y, scale);
                // Clamp endpoints to the inner minimap rect to avoid overdraw.
                let a = clamp_point(a, inner_rect);
                let b = clamp_point(b, inner_rect);
                let line = Path::new(|p| { p.move_to(a); p.line_to(b); });
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.28))
                        .with_width(1.0),
                );
            }

            // ── Cards ──────────────────────────────────────────────────────
            for card in &self.cards {
                let cx = mm.x + INNER + (card.pos.x - min_x) * scale;
                let cy = mm.y + INNER + (card.pos.y - min_y) * scale;
                let cw = (card.width  * scale).max(3.0);
                let ch = (card.height * scale).max(2.0);
                let cr = clip_rect(Rectangle { x: cx, y: cy, width: cw, height: ch }, inner_rect);
                if cr.width > 0.0 && cr.height > 0.0 {
                    let col = Color { a: 0.82, ..card.color };
                    frame.fill(&rounded_rect(cr, 1.5), col);
                }
            }

            // ── Viewport rectangle ─────────────────────────────────────────
            let vp_world_w  = self.window_size.width  / self.zoom;
            let vp_world_h  = self.window_size.height / self.zoom;
            let vp_world_cx = self.window_size.width  / 2.0 - self.offset.x;
            let vp_world_cy = self.window_size.height / 2.0 - self.offset.y;
            let vp_mm_x     = (vp_world_cx - vp_world_w / 2.0 - min_x) * scale + mm.x + INNER;
            let vp_mm_y     = (vp_world_cy - vp_world_h / 2.0 - min_y) * scale + mm.y + INNER;
            let vp_mm_w     = vp_world_w * scale;
            let vp_mm_h     = vp_world_h * scale;

            // Clip to minimap interior.
            let clip_x1 = (mm.x + 1.0).max(vp_mm_x);
            let clip_y1 = (mm.y + 1.0).max(vp_mm_y);
            let clip_x2 = (mm.x + MM_W - 1.0).min(vp_mm_x + vp_mm_w);
            let clip_y2 = (mm.y + MM_H - 1.0).min(vp_mm_y + vp_mm_h);
            if clip_x2 > clip_x1 && clip_y2 > clip_y1 {
                let vp_rect = Rectangle {
                    x:      clip_x1,
                    y:      clip_y1,
                    width:  clip_x2 - clip_x1,
                    height: clip_y2 - clip_y1,
                };
                frame.stroke(
                    &rounded_rect(vp_rect, 2.0),
                    Stroke::default()
                        .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.55))
                        .with_width(1.5),
                );
            }

            GeoRenderer::draw_geometry(renderer, frame.into_geometry());
        });
    }

    fn on_event(
        &mut self,
        tree:       &mut widget::Tree,
        event:      Event,
        _layout:    Layout<'_>,
        cursor:     mouse::Cursor,
        _renderer:  &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell:      &mut iced::advanced::Shell<'_, Message>,
        _viewport:  &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<MinimapState>();
        let Some((min_x, min_y, scale)) = self.world_layout() else {
            return iced::event::Status::Ignored;
        };
        let mm = self.mm_rect();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if mm.contains(pos) {
                        state.dragging      = true;
                        state.last_drag_pos = Some(pos);
                        // Click-to-navigate: pan so the clicked world position is centered.
                        let world = self.mm_to_world(pos.x, pos.y, min_x, min_y, scale);
                        let pan = Vector::new(
                            self.window_size.width  / 2.0 - self.offset.x - world.x,
                            self.window_size.height / 2.0 - self.offset.y - world.y,
                        );
                        shell.publish((self.on_pan)(pan));
                        return iced::event::Status::Captured;
                    }
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    if let Some(pos) = cursor.position() {
                        if let Some(last) = state.last_drag_pos {
                            // Incremental drag: cursor delta in minimap pixels → world pan.
                            // Moving cursor right = viewport moves right = offset decreases.
                            let pan = Vector::new(
                                -(pos.x - last.x) / scale,
                                -(pos.y - last.y) / scale,
                            );
                            shell.publish((self.on_pan)(pan));
                        }
                        state.last_drag_pos = Some(pos);
                        return iced::event::Status::Captured;
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging {
                    state.dragging      = false;
                    state.last_drag_pos = None;
                    return iced::event::Status::Captured;
                }
            }

            _ => {}
        }

        iced::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree:      &widget::Tree,
        _layout:   Layout<'_>,
        cursor:    mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<MinimapState>();
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        if let Some(pos) = cursor.position() {
            if self.mm_rect().contains(pos) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<MinimapOverlay<Message>> for Element<'a, Message> {
    fn from(m: MinimapOverlay<Message>) -> Self { Element::new(m) }
}

// ── Geometry helpers ──────────────────────────────────────────────────────────

fn rounded_rect(rect: Rectangle, radius: f32) -> Path {
    let mut b = Builder::new();
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let r = radius.min(w / 2.0).min(h / 2.0);
    b.move_to(Point::new(x + r, y));
    b.line_to(Point::new(x + w - r, y));
    b.arc_to(Point::new(x + w, y),     Point::new(x + w, y + r),     r);
    b.line_to(Point::new(x + w, y + h - r));
    b.arc_to(Point::new(x + w, y + h), Point::new(x + w - r, y + h), r);
    b.line_to(Point::new(x + r, y + h));
    b.arc_to(Point::new(x, y + h),     Point::new(x, y + h - r),     r);
    b.line_to(Point::new(x, y + r));
    b.arc_to(Point::new(x, y),         Point::new(x + r, y),         r);
    b.close();
    b.build()
}

fn clip_rect(r: Rectangle, clip: Rectangle) -> Rectangle {
    let x1 = r.x.max(clip.x);
    let y1 = r.y.max(clip.y);
    let x2 = (r.x + r.width ).min(clip.x + clip.width);
    let y2 = (r.y + r.height).min(clip.y + clip.height);
    Rectangle { x: x1, y: y1, width: (x2 - x1).max(0.0), height: (y2 - y1).max(0.0) }
}

fn clamp_point(p: Point, r: Rectangle) -> Point {
    Point {
        x: p.x.clamp(r.x, r.x + r.width),
        y: p.y.clamp(r.y, r.y + r.height),
    }
}
