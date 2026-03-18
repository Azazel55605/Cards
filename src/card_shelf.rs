/// CardShelf — a small floating toolbar pill at the bottom of the screen,
/// styled identically to CardToolbar.  When a drag is active the ghost card
/// silhouette is rendered by the same widget so the tree structure stays stable.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::widget::canvas::{gradient, Frame, Path};
use iced::widget::canvas::path::Builder;
use iced::advanced::svg::{Svg as SvgDrawable, Handle as SvgHandle};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector, mouse};
use crate::card::CardType;

// Pill geometry — mirrors card_toolbar.rs
const BTN_SIZE: f32 = 32.0;
const SPACING:  f32 =  4.0;
const PAD_H:    f32 =  8.0;
const PAD_V:    f32 =  6.0;
const CORNER_R: f32 = 10.0;
const ICON_SZ:  f32 = 18.0;
const MARGIN_B: f32 = 14.0;

pub const SHELF_HEIGHT: f32    = BTN_SIZE + PAD_V * 2.0 + MARGIN_B * 2.0;
pub const GHOST_CARD_W: f32    = 240.0;
pub const GHOST_TOP_BAR_H: f32 = 30.0;

const ICON_TYPE_TEXT:     &[u8] = include_bytes!("icons/type-text.svg");
const ICON_TYPE_MARKDOWN: &[u8] = include_bytes!("icons/type-markdown.svg");
const ICON_TYPE_IMAGE:    &[u8] = include_bytes!("icons/type-image.svg");

const TILES: &[CardType] = &[CardType::Text, CardType::Markdown, CardType::Image];

fn pill_w() -> f32 {
    let inner: f32 = (TILES.len() as f32) * BTN_SIZE + (TILES.len() - 1) as f32 * SPACING;
    inner + PAD_H * 2.0
}
fn pill_h() -> f32 { BTN_SIZE + PAD_V * 2.0 }

// ── CardShelf ─────────────────────────────────────────────────────────────────

pub struct CardShelf<Message: Clone> {
    window_width:    f32,
    window_height:   f32,
    on_drag_start:   Box<dyn Fn(CardType, Point) -> Message>,
    background:      Color,
    border_color:    Color,
    shadow_color:    Color,
    icon_color:      Color,
    hover_color:     Color,
    // Ghost card data — Some when a drag is active and cursor is above the shelf zone
    ghost:           Option<(CardType, Point)>,
    card_background: Color,
    card_border:     Color,
    accent_color:    Color,
}

impl<Message: Clone + 'static> CardShelf<Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window_width:  f32,
        window_height: f32,
        on_drag_start: impl Fn(CardType, Point) -> Message + 'static,
        background:    Color,
        border_color:  Color,
        shadow_color:  Color,
        icon_color:    Color,
        hover_color:   Color,
    ) -> Self {
        Self {
            window_width,
            window_height,
            on_drag_start: Box::new(on_drag_start),
            background,
            border_color,
            shadow_color,
            icon_color,
            hover_color,
            ghost: None,
            card_background: Color::TRANSPARENT,
            card_border:     Color::TRANSPARENT,
            accent_color:    Color::TRANSPARENT,
        }
    }

    /// Attach ghost card data so the shelf renders the drag silhouette too.
    pub fn with_ghost(
        mut self,
        card_type:       CardType,
        cursor:          Point,
        card_background: Color,
        card_border:     Color,
        accent_color:    Color,
    ) -> Self {
        self.ghost           = Some((card_type, cursor));
        self.card_background = card_background;
        self.card_border     = card_border;
        self.accent_color    = accent_color;
        self
    }

    fn pill_rect(&self) -> Rectangle {
        Rectangle {
            x: (self.window_width - pill_w()) / 2.0,
            y: self.window_height - pill_h() - MARGIN_B,
            width:  pill_w(),
            height: pill_h(),
        }
    }

    fn btn_rects(&self) -> Vec<Rectangle> {
        let pill = self.pill_rect();
        let mut out = Vec::new();
        let mut x = pill.x + PAD_H;
        let y = pill.y + PAD_V;
        for _ in TILES {
            out.push(Rectangle { x, y, width: BTN_SIZE, height: BTN_SIZE });
            x += BTN_SIZE + SPACING;
        }
        out
    }
}

#[derive(Default)]
struct CardShelfState { hovered: Option<usize> }

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer> for CardShelf<Message> {
    fn size(&self) -> Size<Length> { Size { width: Length::Fill, height: Length::Fill } }

    fn layout(&self, _tree: &mut widget::Tree, _renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn tag(&self)   -> widget::tree::Tag   { widget::tree::Tag::of::<CardShelfState>() }
    fn state(&self) -> widget::tree::State { widget::tree::State::new(CardShelfState::default()) }
    fn children(&self) -> Vec<widget::Tree> { vec![] }
    fn diff(&self, _tree: &mut widget::Tree) {}

    fn draw(
        &self,
        tree:      &widget::Tree,
        renderer:  &mut iced::Renderer,
        _theme:    &iced::Theme,
        _style:    &renderer::Style,
        layout:    Layout<'_>,
        _cursor:   mouse::Cursor,
        viewport:  &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::graphics::geometry::Renderer as GeoRenderer;
        use iced::advanced::svg::Renderer as SvgRenderer;

        let state  = tree.state.downcast_ref::<CardShelfState>();
        let bounds = layout.bounds();
        let pill   = self.pill_rect();
        let vp     = viewport.union(&pill);

        // ── ghost card ──────────────────────────────────────────────────────
        if let Some((card_type, cursor)) = self.ghost {
            let top_left  = ghost_top_left(cursor);
            let card_rect = Rectangle { x: top_left.x, y: top_left.y, width: GHOST_CARD_W, height: 160.0 };

            renderer.with_layer(bounds, |renderer| {
                // card body
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: card_rect,
                        border: Border {
                            color:  Color { a: 0.55, ..self.card_border },
                            width:  1.5,
                            radius: 12.0.into(),
                        },
                        shadow: Shadow {
                            color:       Color { a: 0.25, ..Color::BLACK },
                            offset:      Vector::new(0.0, 6.0),
                            blur_radius: 16.0,
                        },
                    },
                    Color { a: 0.72, ..self.card_background },
                );

                // top bar gradient
                let bar = Rectangle {
                    x: card_rect.x, y: card_rect.y,
                    width: card_rect.width, height: GHOST_TOP_BAR_H,
                };
                let grad = gradient::Linear::new(
                    Point::new(bar.x, bar.y),
                    Point::new(bar.x + bar.width, bar.y),
                )
                .add_stop(0.0, Color { a: 0.15, ..self.card_border })
                .add_stop(1.0, Color { a: 0.30, ..self.accent_color });
                let mut frame = Frame::new(&*renderer, bounds.size());
                frame.fill(&rounded_rect_top(bar, 12.0), grad);
                GeoRenderer::draw_geometry(renderer, frame.into_geometry());

                // type icon
                let icon_data: &[u8] = match card_type {
                    CardType::Text     => ICON_TYPE_TEXT,
                    CardType::Markdown => ICON_TYPE_MARKDOWN,
                    CardType::Image    => ICON_TYPE_IMAGE,
                };
                let handle  = SvgHandle::from_memory(icon_data.to_vec());
                let icon_sz = 18.0_f32;
                renderer.draw_svg(
                    SvgDrawable { handle, color: Some(Color { a: 0.80, ..self.accent_color }), rotation: iced::Radians(0.0), opacity: 1.0 },
                    Rectangle {
                        x: card_rect.x + card_rect.width - icon_sz - 8.0,
                        y: card_rect.y + (GHOST_TOP_BAR_H - icon_sz) / 2.0,
                        width: icon_sz, height: icon_sz,
                    },
                );
            });
        }

        // ── pill ────────────────────────────────────────────────────────────
        renderer.with_layer(vp, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: pill,
                    border: Border { color: self.border_color, width: 1.0, radius: CORNER_R.into() },
                    shadow: Shadow {
                        color:       self.shadow_color,
                        offset:      Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                },
                self.background,
            );

            for (i, (card_type, btn)) in TILES.iter().zip(self.btn_rects().iter()).enumerate() {
                if state.hovered == Some(i) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: *btn,
                            border: Border { radius: 6.0.into(), ..Border::default() },
                            shadow: Shadow::default(),
                        },
                        self.hover_color,
                    );
                }

                let icon_data: &[u8] = match card_type {
                    CardType::Text     => ICON_TYPE_TEXT,
                    CardType::Markdown => ICON_TYPE_MARKDOWN,
                    CardType::Image    => ICON_TYPE_IMAGE,
                };
                let handle = SvgHandle::from_memory(icon_data.to_vec());
                renderer.draw_svg(
                    SvgDrawable { handle, color: Some(self.icon_color), rotation: iced::Radians(0.0), opacity: 1.0 },
                    Rectangle {
                        x: btn.x + (BTN_SIZE - ICON_SZ) / 2.0,
                        y: btn.y + (BTN_SIZE - ICON_SZ) / 2.0,
                        width: ICON_SZ, height: ICON_SZ,
                    },
                );
            }
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
        let state = tree.state.downcast_mut::<CardShelfState>();
        let btns  = self.btn_rects();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new = cursor.position().and_then(|pos| {
                    btns.iter().position(|r| r.contains(pos))
                });
                if state.hovered != new { state.hovered = new; }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    for (card_type, btn) in TILES.iter().zip(btns.iter()) {
                        if btn.contains(pos) {
                            shell.publish((self.on_drag_start)(*card_type, pos));
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
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position() {
            if self.btn_rects().iter().any(|r| r.contains(pos)) {
                return mouse::Interaction::Grab;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<CardShelf<Message>> for Element<'a, Message> {
    fn from(s: CardShelf<Message>) -> Self { Element::new(s) }
}

// ── helpers ───────────────────────────────────────────────────────────────────

pub fn ghost_top_left(cursor: Point) -> Point {
    Point::new(cursor.x - GHOST_CARD_W / 2.0, cursor.y - GHOST_TOP_BAR_H / 2.0)
}

fn rounded_rect_top(rect: Rectangle, radius: f32) -> Path {
    let mut b = Builder::new();
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let r = radius.min(w / 2.0).min(h / 2.0);
    b.move_to(Point::new(x + r, y));
    b.line_to(Point::new(x + w - r, y));
    b.arc_to(Point::new(x + w, y), Point::new(x + w, y + r), r);
    b.line_to(Point::new(x + w, y + h));
    b.line_to(Point::new(x, y + h));
    b.line_to(Point::new(x, y + r));
    b.arc_to(Point::new(x, y), Point::new(x + r, y), r);
    b.close();
    b.build()
}
