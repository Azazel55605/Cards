/// CardToolbar — a fully custom iced widget that draws the floating toolbar
/// above a selected card.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector};

const BTN_SIZE: f32 = 32.0;
const SPACING:  f32 =  4.0;
const PAD_H:    f32 =  8.0;
const PAD_V:    f32 =  6.0;
const SEP_W:    f32 =  1.0;
const CORNER_R: f32 = 10.0;

pub enum ToolbarItem<Message> {
    Icon  { handle: iced::widget::svg::Handle, message: Message },
    Separator,
}

impl<Message: Clone> Clone for ToolbarItem<Message> {
    fn clone(&self) -> Self {
        match self {
            Self::Icon  { handle, message } => Self::Icon  { handle: handle.clone(), message: message.clone() },
            Self::Separator                 => Self::Separator,
        }
    }
}

pub struct CardToolbar<Message> {
    items:        Vec<ToolbarItem<Message>>,
    background:   Color,
    border_color: Color,
    shadow_color: Color,
    icon_color:   Color,
    hover_color:  Color,
    text_color:   Color,
    /// Top-left position in screen coordinates
    position:     Point,
}

impl<Message: Clone> CardToolbar<Message> {
    pub fn new(
        items:        Vec<ToolbarItem<Message>>,
        position:     Point,
        background:   Color,
        border_color: Color,
        shadow_color: Color,
        icon_color:   Color,
        hover_color:  Color,
        text_color:   Color,
    ) -> Self {
        Self { items, position, background, border_color, shadow_color,
               icon_color, hover_color, text_color }
    }

    /// Exact pill width for this item list.
    pub fn measure_width(items: &[ToolbarItem<Message>]) -> f32 {
        if items.is_empty() { return 0.0; }
        let inner: f32 = items.iter().enumerate().map(|(i, item)| {
            let w = if matches!(item, ToolbarItem::Separator) { SEP_W } else { BTN_SIZE };
            if i == 0 { w } else { w + SPACING }
        }).sum();
        inner + PAD_H * 2.0
    }

    pub fn pill_height() -> f32 { BTN_SIZE + PAD_V * 2.0 }

    fn pill_rect(&self) -> Rectangle {
        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width:  Self::measure_width(&self.items),
            height: Self::pill_height(),
        }
    }

    /// Absolute screen rects for each item, computed from self.position directly.
    fn item_rects(&self) -> Vec<Rectangle> {
        let mut out = Vec::new();
        let mut x = self.position.x + PAD_H;
        let y = self.position.y + PAD_V;
        for item in &self.items {
            let w = if matches!(item, ToolbarItem::Separator) { SEP_W } else { BTN_SIZE };
            out.push(Rectangle { x, y, width: w, height: BTN_SIZE });
            x += w + SPACING;
        }
        out
    }
}

#[derive(Default)]
pub struct CardToolbarState { hovered: Option<usize> }

impl<Message: Clone + 'static> Widget<Message, iced::Theme, iced::Renderer>
    for CardToolbar<Message>
{
    fn tag(&self)   -> widget::tree::Tag   { widget::tree::Tag::of::<CardToolbarState>() }
    fn state(&self) -> widget::tree::State { widget::tree::State::new(CardToolbarState::default()) }
    fn children(&self) -> Vec<widget::Tree> { vec![] }
    fn diff(&self, _tree: &mut widget::Tree) {}

    // Fill the whole window so our layout node encompasses everything —
    // exactly like ContextMenu. We draw at self.position inside draw().
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
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::svg::Renderer as SvgRenderer;

        let state = tree.state.downcast_ref::<CardToolbarState>();
        let full   = layout.bounds();          // full window rect
        let pill   = self.pill_rect();         // our absolute pill rect
        let vp     = viewport.union(&pill);

        // Draw on its own layer so it always appears above the canvas
        renderer.with_layer(vp, |renderer| {
            // Background + border + shadow
            renderer.fill_quad(
                renderer::Quad {
                    bounds: pill,
                    border: Border { color: self.border_color, width: 1.0, radius: CORNER_R.into() },
                    shadow: Shadow {
                        color: self.shadow_color,
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                },
                self.background,
            );

            let rects = self.item_rects();
            for (i, (item, rect)) in self.items.iter().zip(rects.iter()).enumerate() {
                match item {
                    ToolbarItem::Separator => {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: rect.x,
                                    y: rect.y + 4.0,
                                    width: SEP_W,
                                    height: BTN_SIZE - 8.0,
                                },
                                border: Border::default(),
                                shadow: Shadow::default(),
                            },
                            Color { a: 0.35, ..self.border_color },
                        );
                    }
                    ToolbarItem::Icon { handle, .. } => {
                        if state.hovered == Some(i) {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: *rect,
                                    border: Border { radius: 6.0.into(), ..Border::default() },
                                    shadow: Shadow::default(),
                                },
                                self.hover_color,
                            );
                        }
                        let sz = 18.0_f32;
                        renderer.draw_svg(
                            iced::advanced::svg::Svg {
                                handle: handle.clone(),
                                color: Some(self.icon_color),
                                rotation: iced::Radians(0.0),
                                opacity: 1.0,
                            },
                            Rectangle {
                                x: rect.x + (BTN_SIZE - sz) / 2.0,
                                y: rect.y + (BTN_SIZE - sz) / 2.0,
                                width: sz,
                                height: sz,
                            },
                        );
                    }
                }
            }
        });

        let _ = full; // suppress unused warning
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        use iced::advanced::graphics::core::event::Status;

        let state = tree.state.downcast_mut::<CardToolbarState>();
        let rects = self.item_rects();  // absolute coords from self.position

        match &event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new_hov = cursor.position().and_then(|pos| {
                    rects.iter().zip(self.items.iter()).position(|(r, item)| {
                        r.contains(pos) && !matches!(item, ToolbarItem::Separator)
                    })
                });
                if state.hovered != new_hov { state.hovered = new_hov; }
                Status::Ignored
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    for (item, rect) in self.items.iter().zip(rects.iter()) {
                        if rect.contains(pos) {
                            let msg = match item {
                                ToolbarItem::Icon  { message, .. } => Some(message.clone()),
                                ToolbarItem::Separator             => None,
                            };
                            if let Some(m) = msg {
                                shell.publish(m);
                                return Status::Captured;
                            }
                        }
                    }
                }
                Status::Ignored
            }
            _ => Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<CardToolbarState>();
        let rects = self.item_rects();
        if let Some(pos) = cursor.position() {
            for (i, (rect, item)) in rects.iter().zip(self.items.iter()).enumerate() {
                if rect.contains(pos) && !matches!(item, ToolbarItem::Separator) {
                    return mouse::Interaction::Pointer;
                }
            }
        }
        if state.hovered.is_some() { return mouse::Interaction::Pointer; }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'static> From<CardToolbar<Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(t: CardToolbar<Message>) -> Self { Element::new(t) }
}






