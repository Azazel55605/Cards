/// ConnectionToolbar — a floating pill toolbar that appears when the user
/// hovers over a connection line.  Drawn using canvas paths so no SVG assets
/// are needed for the custom line-style and arrow icons.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::advanced::graphics::geometry::Renderer as GeoRenderer;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector, mouse};

use crate::card::LineStyle;

// Geometry constants — same visual language as CardShelf
const BTN_SZ:    f32 = 26.0;
const SPACING:   f32 =  3.0;
const PAD_H:     f32 =  8.0;
const PAD_V:     f32 =  5.0;
const SEP_W:     f32 =  1.0;
const SEP_MG:    f32 =  4.0; // extra horizontal gap around each separator
const CORNER_R:  f32 = 10.0;
const ABOVE:     f32 = 18.0; // how far above the bezier midpoint the pill floats

// Button indices (for hovered-state tracking)
const IDX_SOLID:      usize = 0;
const IDX_DASHED:     usize = 1;
const IDX_DOTTED:     usize = 2;
const IDX_ARROW_FROM: usize = 3;
const IDX_ARROW_TO:   usize = 4;
const IDX_DELETE:     usize = 5;
const BTN_COUNT:      usize = 6;

/// Returns pill width for the 6-button layout.
fn pill_w() -> f32 {
    // Groups: [solid dashed dotted] [arrow_from arrow_to] [delete]
    // Spacings within groups: 2 + 1 + 0 = 3, between groups: 2 separators
    let buttons = BTN_COUNT as f32 * BTN_SZ;
    let inner_spacings = 3.0 * SPACING; // 2 within group1, 1 within group2
    let seps = 2.0 * (SEP_W + SEP_MG * 2.0);
    buttons + inner_spacings + seps + PAD_H * 2.0
}
fn pill_h() -> f32 { BTN_SZ + PAD_V * 2.0 }

pub struct ConnectionToolbar<Message: Clone> {
    /// Screen-space bezier midpoint — pill is centered above this.
    midpoint:    Point,
    window_size: Size,
    // Connection state
    pub line_style:  LineStyle,
    pub arrow_from:  bool,
    pub arrow_to:    bool,
    // Colours
    background:  Color,
    border_color: Color,
    shadow_color: Color,
    icon_color:   Color,
    hover_color:  Color,
    accent_color: Color,
    danger_color: Color,
    // Callbacks
    on_set_style:         Box<dyn Fn(LineStyle) -> Message>,
    on_toggle_arrow_from: Box<dyn Fn() -> Message>,
    on_toggle_arrow_to:   Box<dyn Fn() -> Message>,
    on_delete:            Box<dyn Fn() -> Message>,
}

impl<Message: Clone + 'static> ConnectionToolbar<Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        midpoint:             Point,
        window_size:          Size,
        line_style:           LineStyle,
        arrow_from:           bool,
        arrow_to:             bool,
        background:           Color,
        border_color:         Color,
        shadow_color:         Color,
        icon_color:           Color,
        hover_color:          Color,
        accent_color:         Color,
        danger_color:         Color,
        on_set_style:         impl Fn(LineStyle) -> Message + 'static,
        on_toggle_arrow_from: impl Fn() -> Message + 'static,
        on_toggle_arrow_to:   impl Fn() -> Message + 'static,
        on_delete:            impl Fn() -> Message + 'static,
    ) -> Self {
        Self {
            midpoint, window_size, line_style, arrow_from, arrow_to,
            background, border_color, shadow_color, icon_color, hover_color, accent_color, danger_color,
            on_set_style: Box::new(on_set_style),
            on_toggle_arrow_from: Box::new(on_toggle_arrow_from),
            on_toggle_arrow_to:   Box::new(on_toggle_arrow_to),
            on_delete:            Box::new(on_delete),
        }
    }

    /// Pill top-left, clamped inside the window.
    fn pill_rect(&self) -> Rectangle {
        let pw = pill_w();
        let ph = pill_h();
        let x = (self.midpoint.x - pw / 2.0)
            .max(4.0)
            .min(self.window_size.width - pw - 4.0);
        let y = (self.midpoint.y - ph - ABOVE)
            .max(4.0)
            .min(self.window_size.height - ph - 4.0);
        Rectangle { x, y, width: pw, height: ph }
    }

    /// Screen rectangles for each of the 6 buttons, in order.
    fn btn_rects(&self) -> [Rectangle; BTN_COUNT] {
        let pill = self.pill_rect();
        let y    = pill.y + PAD_V;
        let mut x = pill.x + PAD_H;

        let mut rects = [Rectangle::default(); BTN_COUNT];

        // Group 1: solid, dashed, dotted  (3 buttons, 2 SPACING gaps)
        for i in 0..3 {
            rects[i] = Rectangle { x, y, width: BTN_SZ, height: BTN_SZ };
            x += BTN_SZ + if i < 2 { SPACING } else { 0.0 };
        }
        // Separator 1
        x += SEP_MG + SEP_W + SEP_MG;
        // Group 2: arrow_from, arrow_to  (2 buttons, 1 SPACING gap)
        for i in 0..2 {
            rects[3 + i] = Rectangle { x, y, width: BTN_SZ, height: BTN_SZ };
            x += BTN_SZ + if i < 1 { SPACING } else { 0.0 };
        }
        // Separator 2
        x += SEP_MG + SEP_W + SEP_MG;
        // Group 3: delete
        rects[5] = Rectangle { x, y, width: BTN_SZ, height: BTN_SZ };
        rects
    }

    /// Returns the rectangle that should be passed to `DotGrid::set_toolbar_region`.
    pub fn pill_screen_rect(&self) -> Rectangle { self.pill_rect() }
}

// ── Widget state ───────────────────────────────────────────────────────────────

#[derive(Default)]
struct ToolbarState { hovered: Option<usize> }

// ── Widget impl ────────────────────────────────────────────────────────────────

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer>
    for ConnectionToolbar<Message>
{
    fn size(&self) -> Size<Length> { Size { width: Length::Fill, height: Length::Fill } }

    fn layout(&self, _tree: &mut widget::Tree, _renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn tag(&self)   -> widget::tree::Tag   { widget::tree::Tag::of::<ToolbarState>() }
    fn state(&self) -> widget::tree::State { widget::tree::State::new(ToolbarState::default()) }
    fn children(&self) -> Vec<widget::Tree> { vec![] }
    fn diff(&self, _tree: &mut widget::Tree) {}

    fn draw(
        &self,
        tree:     &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme:   &iced::Theme,
        _style:   &renderer::Style,
        layout:   Layout<'_>,
        _cursor:  mouse::Cursor,
        _vp:      &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let state  = tree.state.downcast_ref::<ToolbarState>();
        let bounds = layout.bounds();
        let pill   = self.pill_rect();
        let btns   = self.btn_rects();

        renderer.with_layer(bounds, |renderer| {
            // ── pill background ────────────────────────────────────────────
            renderer.fill_quad(
                renderer::Quad {
                    bounds: pill,
                    border: Border { color: self.border_color, width: 1.0, radius: CORNER_R.into() },
                    shadow: Shadow { color: self.shadow_color, offset: Vector::new(0.0, 4.0), blur_radius: 10.0 },
                },
                self.background,
            );

            // ── separator lines ────────────────────────────────────────────
            let sep_x = [
                btns[2].x + BTN_SZ + SEP_MG,
                btns[3].x + 2.0 * BTN_SZ + SPACING + SEP_MG,
            ];
            for sx in &sep_x {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle { x: *sx, y: pill.y + PAD_V + 3.0, width: SEP_W, height: BTN_SZ - 6.0 },
                        border: Border::default(),
                        shadow: Shadow::default(),
                    },
                    Color { a: 0.25, ..self.border_color },
                );
            }

            // ── button hover backgrounds ───────────────────────────────────
            for (i, btn) in btns.iter().enumerate() {
                let active = match i {
                    IDX_SOLID   => self.line_style == LineStyle::Solid,
                    IDX_DASHED  => self.line_style == LineStyle::Dashed,
                    IDX_DOTTED  => self.line_style == LineStyle::Dotted,
                    IDX_ARROW_FROM => self.arrow_from,
                    IDX_ARROW_TO   => self.arrow_to,
                    _ => false,
                };
                let is_del  = i == IDX_DELETE;
                let hovered = state.hovered == Some(i);

                if active {
                    let bg = if is_del { Color { a: 0.18, ..self.danger_color } } else { Color { a: 0.18, ..self.accent_color } };
                    renderer.fill_quad(
                        renderer::Quad { bounds: *btn, border: Border { radius: 5.0.into(), ..Border::default() }, shadow: Shadow::default() },
                        bg,
                    );
                } else if hovered {
                    let bg = if is_del { Color { a: 0.14, ..self.danger_color } } else { self.hover_color };
                    renderer.fill_quad(
                        renderer::Quad { bounds: *btn, border: Border { radius: 5.0.into(), ..Border::default() }, shadow: Shadow::default() },
                        bg,
                    );
                }
            }

            // ── icons (canvas paths) ───────────────────────────────────────
            let mut frame = Frame::new(&*renderer, bounds.size());

            for (i, btn) in btns.iter().enumerate() {
                let _cx = btn.x + BTN_SZ / 2.0;
                let cy = btn.y + BTN_SZ / 2.0;
                let hovered = state.hovered == Some(i);

                let active = match i {
                    IDX_SOLID  => self.line_style == LineStyle::Solid,
                    IDX_DASHED => self.line_style == LineStyle::Dashed,
                    IDX_DOTTED => self.line_style == LineStyle::Dotted,
                    IDX_ARROW_FROM => self.arrow_from,
                    IDX_ARROW_TO   => self.arrow_to,
                    _ => false,
                };
                let is_del = i == IDX_DELETE;
                let icon_col = if is_del && (active || hovered) {
                    self.danger_color
                } else if (active || hovered) && !is_del {
                    self.accent_color
                } else {
                    self.icon_color
                };

                match i {
                    IDX_SOLID => {
                        // Solid horizontal line
                        frame.stroke(
                            &Path::line(Point::new(btn.x + 5.0, cy), Point::new(btn.x + BTN_SZ - 5.0, cy)),
                            Stroke::default().with_color(icon_col).with_width(2.0),
                        );
                    }
                    IDX_DASHED => {
                        // Two dashes
                        let dash_w = (BTN_SZ - 10.0 - 3.0) / 2.0;
                        for d in 0..2u32 {
                            let x0 = btn.x + 5.0 + d as f32 * (dash_w + 3.0);
                            frame.stroke(
                                &Path::line(Point::new(x0, cy), Point::new(x0 + dash_w, cy)),
                                Stroke::default().with_color(icon_col).with_width(2.0),
                            );
                        }
                    }
                    IDX_DOTTED => {
                        // Three dots
                        for d in 0..3u32 {
                            let dot_x = btn.x + 6.0 + d as f32 * 6.5;
                            frame.fill(&Path::circle(Point::new(dot_x, cy), 2.0), icon_col);
                        }
                    }
                    IDX_ARROW_FROM => {
                        // Line with arrowhead on the LEFT (from-end arrow)
                        let x0 = btn.x + 5.0;
                        let x1 = btn.x + BTN_SZ - 5.0;
                        frame.stroke(
                            &Path::line(Point::new(x0, cy), Point::new(x1, cy)),
                            Stroke::default().with_color(icon_col).with_width(1.5),
                        );
                        // Arrowhead pointing left (tip at x0)
                        draw_arrowhead(&mut frame, Point::new(x0, cy), (-1.0, 0.0), 7.0, icon_col);
                    }
                    IDX_ARROW_TO => {
                        // Line with arrowhead on the RIGHT (to-end arrow)
                        let x0 = btn.x + 5.0;
                        let x1 = btn.x + BTN_SZ - 5.0;
                        frame.stroke(
                            &Path::line(Point::new(x0, cy), Point::new(x1, cy)),
                            Stroke::default().with_color(icon_col).with_width(1.5),
                        );
                        // Arrowhead pointing right (tip at x1)
                        draw_arrowhead(&mut frame, Point::new(x1, cy), (1.0, 0.0), 7.0, icon_col);
                    }
                    IDX_DELETE => {
                        // X shape
                        let m = 7.0;
                        frame.stroke(
                            &Path::line(Point::new(btn.x + m, btn.y + m), Point::new(btn.x + BTN_SZ - m, btn.y + BTN_SZ - m)),
                            Stroke::default().with_color(icon_col).with_width(2.0),
                        );
                        frame.stroke(
                            &Path::line(Point::new(btn.x + BTN_SZ - m, btn.y + m), Point::new(btn.x + m, btn.y + BTN_SZ - m)),
                            Stroke::default().with_color(icon_col).with_width(2.0),
                        );
                    }
                    _ => {}
                }
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
        _vp:        &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<ToolbarState>();
        let btns  = self.btn_rects();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new = cursor.position().and_then(|pos| btns.iter().position(|r| r.contains(pos)));
                if state.hovered != new { state.hovered = new; }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    for (i, btn) in btns.iter().enumerate() {
                        if btn.contains(pos) {
                            let msg = match i {
                                IDX_SOLID  => (self.on_set_style)(LineStyle::Solid),
                                IDX_DASHED => (self.on_set_style)(LineStyle::Dashed),
                                IDX_DOTTED => (self.on_set_style)(LineStyle::Dotted),
                                IDX_ARROW_FROM => (self.on_toggle_arrow_from)(),
                                IDX_ARROW_TO   => (self.on_toggle_arrow_to)(),
                                IDX_DELETE     => (self.on_delete)(),
                                _ => break,
                            };
                            shell.publish(msg);
                            return iced::event::Status::Captured;
                        }
                    }
                }
            }
            _ => {}
        }
        iced::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree:     &widget::Tree,
        _layout:   Layout<'_>,
        cursor:    mouse::Cursor,
        _vp:       &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position() {
            if self.btn_rects().iter().any(|r| r.contains(pos)) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<ConnectionToolbar<Message>> for Element<'a, Message> {
    fn from(t: ConnectionToolbar<Message>) -> Self { Element::new(t) }
}

// ── helpers ────────────────────────────────────────────────────────────────────

/// Draw an arrowhead (two wing lines) at `tip` pointing in `dir` (unit vector).
fn draw_arrowhead(frame: &mut Frame, tip: Point, dir: (f32, f32), size: f32, color: Color) {
    let angle = 35_f32.to_radians();
    let (cos_a, sin_a) = (angle.cos(), angle.sin());
    // Incoming direction (opposite of dir = where the "tail" is)
    let (ivx, ivy) = (-dir.0, -dir.1);
    let wing1 = Point::new(
        tip.x + (ivx * cos_a - ivy * sin_a) * size,
        tip.y + (ivx * sin_a + ivy * cos_a) * size,
    );
    let wing2 = Point::new(
        tip.x + (ivx * cos_a + ivy * sin_a) * size,
        tip.y + (-ivx * sin_a + ivy * cos_a) * size,
    );
    let s = Stroke::default().with_color(color).with_width(1.8);
    frame.stroke(&Path::line(tip, wing1), s.clone());
    frame.stroke(&Path::line(tip, wing2), s);
}
