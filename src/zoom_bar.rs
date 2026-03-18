/// ZoomBar — a pill widget at the bottom-right of the screen showing zoom level
/// with Zoom In (+) and Zoom Out (-) buttons.  Styled identically to CardToolbar.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector, mouse};

const BTN_SIZE:  f32 = 28.0;
const LABEL_W:   f32 = 52.0;
const SPACING:   f32 =  4.0;
const PAD_H:     f32 =  8.0;
const PAD_V:     f32 =  6.0;
const CORNER_R:  f32 = 10.0;
const MARGIN_R:  f32 = 14.0;
const MARGIN_B:  f32 = 14.0;
const FONT_SZ:   f32 = 12.0;

// Layout: [-][  100%  ][+]
const NUM_BTNS: usize = 2;  // minus, plus
fn pill_w() -> f32 {
    NUM_BTNS as f32 * BTN_SIZE + LABEL_W + (NUM_BTNS as f32) * SPACING + PAD_H * 2.0
}
fn pill_h() -> f32 { BTN_SIZE + PAD_V * 2.0 }

pub struct ZoomBar<Message: Clone> {
    window_width:  f32,
    window_height: f32,
    zoom:          f32,
    on_zoom_in:    Box<dyn Fn() -> Message>,
    on_zoom_out:   Box<dyn Fn() -> Message>,
    on_zoom_reset: Box<dyn Fn() -> Message>,
    background:    Color,
    border_color:  Color,
    shadow_color:  Color,
    text_color:    Color,
    hover_color:   Color,
}

impl<Message: Clone + 'static> ZoomBar<Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window_width:  f32,
        window_height: f32,
        zoom:          f32,
        on_zoom_in:    impl Fn() -> Message + 'static,
        on_zoom_out:   impl Fn() -> Message + 'static,
        on_zoom_reset: impl Fn() -> Message + 'static,
        background:    Color,
        border_color:  Color,
        shadow_color:  Color,
        text_color:    Color,
        hover_color:   Color,
    ) -> Self {
        Self {
            window_width,
            window_height,
            zoom,
            on_zoom_in:    Box::new(on_zoom_in),
            on_zoom_out:   Box::new(on_zoom_out),
            on_zoom_reset: Box::new(on_zoom_reset),
            background,
            border_color,
            shadow_color,
            text_color,
            hover_color,
        }
    }

    fn pill_rect(&self) -> Rectangle {
        Rectangle {
            x: self.window_width - pill_w() - MARGIN_R,
            y: self.window_height - pill_h() - MARGIN_B,
            width:  pill_w(),
            height: pill_h(),
        }
    }

    /// Returns [minus_rect, label_rect, plus_rect]
    fn element_rects(&self) -> [Rectangle; 3] {
        let pill = self.pill_rect();
        let y = pill.y + PAD_V;
        let x0 = pill.x + PAD_H;
        let minus = Rectangle { x: x0,                                y, width: BTN_SIZE, height: BTN_SIZE };
        let label = Rectangle { x: x0 + BTN_SIZE + SPACING,           y, width: LABEL_W,  height: BTN_SIZE };
        let plus  = Rectangle { x: x0 + BTN_SIZE + SPACING + LABEL_W + SPACING, y, width: BTN_SIZE, height: BTN_SIZE };
        [minus, label, plus]
    }
}

#[derive(Default)]
struct ZoomBarState { hovered: Option<usize> } // 0=minus, 1=label, 2=plus

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer> for ZoomBar<Message> {
    fn size(&self) -> Size<Length> { Size { width: Length::Fill, height: Length::Fill } }

    fn layout(&self, _tree: &mut widget::Tree, _renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn tag(&self)   -> widget::tree::Tag   { widget::tree::Tag::of::<ZoomBarState>() }
    fn state(&self) -> widget::tree::State { widget::tree::State::new(ZoomBarState::default()) }
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
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::text as text_trait;
        use iced::advanced::text::Renderer as TextRenderer;

        let state = tree.state.downcast_ref::<ZoomBarState>();
        let bounds = layout.bounds();
        let pill   = self.pill_rect();
        let vp     = viewport.union(&pill);
        let [minus, label, plus] = self.element_rects();

        renderer.with_layer(vp, |renderer| {
            // Pill background
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

            // Hover highlight for minus (0) and plus (2)
            for (idx, rect) in [minus, plus].iter().enumerate() {
                let btn_idx = if idx == 0 { 0 } else { 2 };
                if state.hovered == Some(btn_idx) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: *rect,
                            border: Border { radius: 5.0.into(), ..Border::default() },
                            shadow: Shadow::default(),
                        },
                        self.hover_color,
                    );
                }
            }
            // Hover for label (reset, idx=1)
            if state.hovered == Some(1) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: label,
                        border: Border { radius: 5.0.into(), ..Border::default() },
                        shadow: Shadow::default(),
                    },
                    self.hover_color,
                );
            }

            // With Center alignment the position is the anchor (center of text), not top-left.
            let minus_cx = minus.x + minus.width  / 2.0;
            let minus_cy = minus.y + minus.height / 2.0;
            let label_cx = label.x + label.width  / 2.0;
            let label_cy = label.y + label.height / 2.0;
            let plus_cx  = plus.x  + plus.width   / 2.0;
            let plus_cy  = plus.y  + plus.height  / 2.0;

            // Minus button text
            renderer.fill_text(
                text_trait::Text {
                    content: "−".to_string(),
                    bounds:  Size::new(minus.width, minus.height),
                    size:    iced::Pixels(FONT_SZ + 2.0),
                    line_height: iced::widget::text::LineHeight::default(),
                    font:    iced::Font::default(),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment:   iced::alignment::Vertical::Center,
                    shaping: iced::widget::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::None,
                },
                Point::new(minus_cx, minus_cy),
                self.text_color,
                bounds,
            );

            // Zoom label
            let pct = format!("{}%", (self.zoom * 100.0).round() as u32);
            renderer.fill_text(
                text_trait::Text {
                    content: pct,
                    bounds:  Size::new(label.width, label.height),
                    size:    iced::Pixels(FONT_SZ),
                    line_height: iced::widget::text::LineHeight::default(),
                    font:    iced::Font::default(),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment:   iced::alignment::Vertical::Center,
                    shaping: iced::widget::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::None,
                },
                Point::new(label_cx, label_cy),
                self.text_color,
                bounds,
            );

            // Plus button text
            renderer.fill_text(
                text_trait::Text {
                    content: "+".to_string(),
                    bounds:  Size::new(plus.width, plus.height),
                    size:    iced::Pixels(FONT_SZ + 2.0),
                    line_height: iced::widget::text::LineHeight::default(),
                    font:    iced::Font::default(),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment:   iced::alignment::Vertical::Center,
                    shaping: iced::widget::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::None,
                },
                Point::new(plus_cx, plus_cy),
                self.text_color,
                bounds,
            );
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
        let state = tree.state.downcast_mut::<ZoomBarState>();
        let [minus, label, plus] = self.element_rects();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new = cursor.position().and_then(|pos| {
                    if minus.contains(pos) { Some(0) }
                    else if label.contains(pos) { Some(1) }
                    else if plus.contains(pos)  { Some(2) }
                    else { None }
                });
                if state.hovered != new { state.hovered = new; }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if minus.contains(pos) {
                        shell.publish((self.on_zoom_out)());
                        return iced::event::Status::Captured;
                    }
                    if label.contains(pos) {
                        shell.publish((self.on_zoom_reset)());
                        return iced::event::Status::Captured;
                    }
                    if plus.contains(pos) {
                        shell.publish((self.on_zoom_in)());
                        return iced::event::Status::Captured;
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
            let [minus, label, plus] = self.element_rects();
            if minus.contains(pos) || label.contains(pos) || plus.contains(pos) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<ZoomBar<Message>> for Element<'a, Message> {
    fn from(z: ZoomBar<Message>) -> Self { Element::new(z) }
}
