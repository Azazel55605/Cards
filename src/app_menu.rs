/// A custom dropdown/popup menu widget for the application menu.
///
/// Renders its own rounded background, border and shadow entirely with the
/// low-level renderer so there is never a transparent/square override from
/// an inner container.  Supports open/close scale+fade animations.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::advanced::text as text_trait;
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector};

// ─── Item definition ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AppMenuItem<Message> {
    /// A clickable text item.
    Button { label: String, message: Message },
    /// A thin separator line.
    Separator,
    /// A small dimmed section header (not clickable).
    Label(String),
    /// A submenu trigger row — fires on_hover when hovered, on_close when un-hovered.
    SubMenu { label: String, enabled: bool, on_hover: Option<Message>, on_close: Option<Message> },
}

// ─── State stored in the widget tree ─────────────────────────────────────────

#[derive(Default)]
struct MenuState {
    hovered_index: Option<usize>,
}

// ─── The widget itself ────────────────────────────────────────────────────────

pub struct AppMenu<Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    items: Vec<AppMenuItem<Message>>,
    position: Point,
    width: f32,
    background: Color,
    border_color: Color,
    text_color: Color,
    separator_color: Color,
    hover_color: Color,
    shadow_color: Color,
    on_close: Option<Message>,
    on_submenu_hover: Option<Message>,
    on_submenu_close: Option<Message>,
    animation_progress: f32,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<Message, Renderer> AppMenu<Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(items: Vec<AppMenuItem<Message>>, position: Point) -> Self {
        Self {
            items,
            position,
            width: 200.0,
            background: Color::from_rgb8(50, 50, 50),
            border_color: Color::from_rgb8(80, 80, 80),
            text_color: Color::WHITE,
            separator_color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
            hover_color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            on_close: None,
            on_submenu_hover: None,
            on_submenu_close: None,
            animation_progress: 1.0,
            _renderer: std::marker::PhantomData,
        }
    }

    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn background(mut self, c: Color) -> Self { self.background = c; self }
    pub fn border_color(mut self, c: Color) -> Self { self.border_color = c; self }
    pub fn text_color(mut self, c: Color) -> Self { self.text_color = c; self }
    pub fn separator_color(mut self, c: Color) -> Self { self.separator_color = c; self }
    pub fn hover_color(mut self, c: Color) -> Self { self.hover_color = c; self }
    pub fn shadow_color(mut self, c: Color) -> Self { self.shadow_color = c; self }
    pub fn on_close(mut self, msg: Message) -> Self { self.on_close = Some(msg); self }
    pub fn on_submenu_hover(mut self, msg: Message) -> Self { self.on_submenu_hover = Some(msg); self }
    pub fn on_submenu_close(mut self, msg: Message) -> Self { self.on_submenu_close = Some(msg); self }
    pub fn animation_progress(mut self, p: f32) -> Self {
        self.animation_progress = p.clamp(0.0, 1.0);
        self
    }

    // ── geometry helpers ──────────────────────────────────────────────────────

    fn item_height(item: &AppMenuItem<Message>) -> f32 {
        match item {
            AppMenuItem::Button { .. }  => 32.0,
            AppMenuItem::SubMenu { .. } => 32.0,
            AppMenuItem::Separator      => 9.0,
            AppMenuItem::Label(_)       => 26.0,
        }
    }

    fn total_content_height(&self) -> f32 {
        let inner: f32 = self.items.iter().map(Self::item_height).sum();
        inner + 12.0
    }

    fn menu_rect(&self, full_bounds: Rectangle) -> Rectangle {
        let h = self.total_content_height();
        let mut x = full_bounds.x + self.position.x;
        let mut y = full_bounds.y + self.position.y;
        if x + self.width > full_bounds.x + full_bounds.width - 8.0 {
            x = full_bounds.x + full_bounds.width - self.width - 8.0;
        }
        if y + h > full_bounds.y + full_bounds.height - 8.0 {
            y = full_bounds.y + full_bounds.height - h - 8.0;
        }
        x = x.max(full_bounds.x + 8.0);
        y = y.max(full_bounds.y + 8.0);
        Rectangle { x, y, width: self.width, height: h }
    }

    fn item_at_y(&self, local_y: f32) -> Option<usize> {
        let mut cy = 6.0;
        for (i, item) in self.items.iter().enumerate() {
            let h = Self::item_height(item);
            if local_y >= cy && local_y < cy + h { return Some(i); }
            cy += h;
        }
        None
    }
}

// ─── Easing ──────────────────────────────────────────────────────────────────

fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }

// ─── Widget implementation ────────────────────────────────────────────────────

impl<Message, Renderer> Widget<Message, iced::Theme, Renderer> for AppMenu<Message, Renderer>
where
    Message: Clone + 'static,
    Renderer: iced::advanced::Renderer + text_trait::Renderer<Font = iced::Font>,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<MenuState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(MenuState::default())
    }

    fn size(&self) -> Size<Length> {
        Size { width: Length::Fill, height: Length::Fill }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuState>();
        let full_bounds = layout.bounds();
        let base_rect = self.menu_rect(full_bounds);

        let t = ease_out_cubic(self.animation_progress);
        let scale = 0.85 + t * 0.15;
        let alpha = t;

        let rect = Rectangle {
            x: base_rect.x,
            y: base_rect.y,
            width: base_rect.width * scale,
            height: base_rect.height * scale,
        };

        if rect.width < 2.0 || rect.height < 2.0 { return; }

        let radius = 10.0_f32;

        // Collect state values needed inside the closure
        let hovered_index = state.hovered_index;
        let items = &self.items;
        let background = self.background;
        let border_color = self.border_color;
        let shadow_color = self.shadow_color;
        let separator_color = self.separator_color;
        let hover_color = self.hover_color;
        let text_color = self.text_color;

        // Wrap everything in with_layer so this composites on top of the
        // Sidebar's own with_layer, regardless of widget tree draw order.
        renderer.with_layer(full_bounds, |renderer| {
            let bg       = Color { a: background.a * alpha,    ..background };
            let border_c = Color { a: border_color.a * alpha,  ..border_color };
            let shadow_c = Color { a: shadow_color.a * alpha,  ..shadow_color };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    border: Border { color: border_c, width: 1.0, radius: radius.into() },
                    shadow: Shadow { color: shadow_c, offset: Vector::new(0.0, 4.0), blur_radius: 16.0 },
                },
                bg,
            );

            let mut item_y = rect.y + 6.0 * scale;

            for (i, item) in items.iter().enumerate() {
                let item_h = Self::item_height(item) * scale;

                match item {
                    AppMenuItem::Separator => {
                        let sep_c = Color { a: separator_color.a * alpha, ..separator_color };
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: rect.x + 8.0, y: item_y + item_h * 0.5 - 0.5,
                                    width: rect.width - 16.0, height: 1.0,
                                },
                                border: Border::default(),
                                shadow: Shadow::default(),
                            },
                            sep_c,
                        );
                    }

                    AppMenuItem::Label(label) => {
                        let label_c = Color { a: text_color.a * alpha * 0.5, ..text_color };
                        renderer.fill_text(
                            text_trait::Text {
                                content: label.clone(),
                                bounds: Size::new(rect.width - 24.0, item_h),
                                size: iced::Pixels(11.0 * scale),
                                line_height: iced::widget::text::LineHeight::default(),
                                font: iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() },
                                horizontal_alignment: iced::alignment::Horizontal::Left,
                                vertical_alignment: iced::alignment::Vertical::Center,
                                shaping: iced::widget::text::Shaping::Advanced,
                                wrapping: iced::widget::text::Wrapping::None,
                            },
                            Point::new(rect.x + 12.0, item_y + item_h * 0.5),
                            label_c,
                            rect,
                        );
                    }

                    AppMenuItem::Button { label, .. } | AppMenuItem::SubMenu { label, .. } => {
                        let is_submenu = matches!(item, AppMenuItem::SubMenu { .. });
                        let enabled = match item {
                            AppMenuItem::SubMenu { enabled, .. } => *enabled,
                            _ => true,
                        };
                        let item_rect = Rectangle {
                            x: rect.x + 4.0, y: item_y,
                            width: rect.width - 8.0, height: item_h,
                        };

                        if hovered_index == Some(i) && enabled {
                            let hover_c = Color { a: hover_color.a * alpha, ..hover_color };
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: item_rect,
                                    border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 6.0.into() },
                                    shadow: Shadow::default(),
                                },
                                hover_c,
                            );
                        }

                        let label_alpha = if enabled { alpha } else { alpha * 0.35 };
                        let text_c = Color { a: text_color.a * label_alpha, ..text_color };
                        renderer.fill_text(
                            text_trait::Text {
                                content: label.clone(),
                                bounds: Size::new(item_rect.width - 16.0, item_h),
                                size: iced::Pixels(13.0 * scale),
                                line_height: iced::widget::text::LineHeight::default(),
                                font: iced::Font::default(),
                                horizontal_alignment: iced::alignment::Horizontal::Left,
                                vertical_alignment: iced::alignment::Vertical::Center,
                                shaping: iced::widget::text::Shaping::Advanced,
                                wrapping: iced::widget::text::Wrapping::None,
                            },
                            Point::new(item_rect.x + 8.0, item_y + item_h * 0.5),
                            text_c,
                            item_rect,
                        );

                        // Draw ▶ arrow for SubMenu
                        if is_submenu {
                            renderer.fill_text(
                                text_trait::Text {
                                    content: "▶".to_string(),
                                    bounds: Size::new(16.0, item_h),
                                    size: iced::Pixels(10.0 * scale),
                                    line_height: iced::widget::text::LineHeight::default(),
                                    font: iced::Font::default(),
                                    horizontal_alignment: iced::alignment::Horizontal::Right,
                                    vertical_alignment: iced::alignment::Vertical::Center,
                                    shaping: iced::widget::text::Shaping::Advanced,
                                    wrapping: iced::widget::text::Wrapping::None,
                                },
                                Point::new(item_rect.x + item_rect.width - 8.0, item_y + item_h * 0.5),
                                text_c,
                                item_rect,
                            );
                        }
                    }
                }

                item_y += item_h;
            }
        });
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<MenuState>();
        let full_bounds = layout.bounds();
        let base_rect = self.menu_rect(full_bounds);
        let t = ease_out_cubic(self.animation_progress);
        let scale = 0.85 + t * 0.15;
        let rect = Rectangle {
            x: base_rect.x, y: base_rect.y,
            width: base_rect.width * scale, height: base_rect.height * scale,
        };

        match &event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    if rect.contains(pos) {
                        let local_y = (pos.y - rect.y) / scale;
                        let new_hovered = self.item_at_y(local_y).and_then(|i| {
                            match &self.items[i] {
                                AppMenuItem::Button { .. } => Some(i),
                                AppMenuItem::SubMenu { enabled, .. } if *enabled => Some(i),
                                _ => None,
                            }
                        });
                        if state.hovered_index != new_hovered {
                            let now_on_submenu = new_hovered
                                .map(|i| matches!(self.items[i], AppMenuItem::SubMenu { .. }))
                                .unwrap_or(false);
                            let was_submenu_idx = state.hovered_index
                                .filter(|&i| matches!(self.items[i], AppMenuItem::SubMenu { .. }));

                            // Fire close on the old submenu if we're leaving it
                            if let Some(idx) = was_submenu_idx {
                                if let AppMenuItem::SubMenu { on_close: Some(ref msg), .. } = self.items[idx] {
                                    shell.publish(msg.clone());
                                } else if let Some(ref msg) = self.on_submenu_close {
                                    shell.publish(msg.clone());
                                }
                            }

                            state.hovered_index = new_hovered;

                            if now_on_submenu {
                                if let Some(idx) = new_hovered {
                                    if let AppMenuItem::SubMenu { on_hover: Some(ref msg), .. } = self.items[idx] {
                                        shell.publish(msg.clone());
                                    } else if let Some(ref msg) = self.on_submenu_hover {
                                        shell.publish(msg.clone());
                                    }
                                }
                            }
                            return iced::event::Status::Captured;
                        }
                    } else if state.hovered_index.is_some() {
                        state.hovered_index = None;
                        return iced::event::Status::Captured;
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if rect.contains(pos) {
                        let local_y = (pos.y - rect.y) / scale;
                        if let Some(i) = self.item_at_y(local_y) {
                            match &self.items[i] {
                                AppMenuItem::Button { message, .. } => {
                                    // Close all submenus when clicking a button
                                    if let Some(ref msg) = self.on_submenu_close {
                                        shell.publish(msg.clone());
                                    }
                                    shell.publish(message.clone());
                                }
                                AppMenuItem::SubMenu { enabled, on_hover, .. } if *enabled => {
                                    if let Some(ref msg) = on_hover {
                                        shell.publish(msg.clone());
                                    } else if let Some(ref msg) = self.on_submenu_hover {
                                        shell.publish(msg.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                        return iced::event::Status::Captured;
                    } else {
                        if let Some(ref msg) = self.on_close {
                            shell.publish(msg.clone());
                        }
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
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let full_bounds = layout.bounds();
        let base_rect = self.menu_rect(full_bounds);
        let t = ease_out_cubic(self.animation_progress);
        let scale = 0.85 + t * 0.15;
        let rect = Rectangle {
            x: base_rect.x, y: base_rect.y,
            width: base_rect.width * scale, height: base_rect.height * scale,
        };

        if let Some(pos) = cursor.position() {
            if rect.contains(pos) {
                let local_y = (pos.y - rect.y) / scale;
                if let Some(i) = self.item_at_y(local_y) {
                    match &self.items[i] {
                        AppMenuItem::Button { .. } => return mouse::Interaction::Pointer,
                        AppMenuItem::SubMenu { enabled, .. } if *enabled => return mouse::Interaction::Pointer,
                        _ => {}
                    }
                }
                return mouse::Interaction::default();
            }
        }
        mouse::Interaction::default()
    }
}

// ─── Into<Element> ───────────────────────────────────────────────────────────

impl<'a, Message, Renderer> From<AppMenu<Message, Renderer>>
    for Element<'a, Message, iced::Theme, Renderer>
where
    Message: Clone + 'static,
    Renderer: iced::advanced::Renderer + text_trait::Renderer<Font = iced::Font> + 'a,
{
    fn from(menu: AppMenu<Message, Renderer>) -> Self {
        Element::new(menu)
    }
}





