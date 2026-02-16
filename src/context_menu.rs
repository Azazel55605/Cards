use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector};

pub struct ContextMenu<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    content: Element<'a, Message, iced::Theme, Renderer>,
    position: Point,
    background: Color,
    border_color: Color,
    shadow_color: Color,
    width: f32,
    on_close: Option<Message>,
}

impl<'a, Message, Renderer> ContextMenu<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
        position: Point,
        background: Color,
        border_color: Color,
        shadow_color: Color,
    ) -> Self {
        Self {
            content: content.into(),
            position,
            background,
            border_color,
            shadow_color,
            width: 180.0,
            on_close: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    fn calculate_menu_bounds(&self, full_bounds: Rectangle, content_height: f32) -> Rectangle {
        let mut menu_x = full_bounds.x + self.position.x;
        let mut menu_y = full_bounds.y + self.position.y;

        // Adjust if menu would go off screen
        if menu_x + self.width > full_bounds.x + full_bounds.width {
            menu_x = full_bounds.x + full_bounds.width - self.width - 10.0;
        }
        if menu_y + content_height > full_bounds.y + full_bounds.height {
            menu_y = full_bounds.y + full_bounds.height - content_height - 10.0;
        }

        // Ensure menu stays within bounds
        menu_x = menu_x.max(full_bounds.x + 10.0);
        menu_y = menu_y.max(full_bounds.y + 10.0);

        Rectangle {
            x: menu_x,
            y: menu_y,
            width: self.width,
            height: content_height,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for ContextMenu<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let max_size = limits.max();

        let content_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(self.width, max_size.height),
        );

        let content_layout = self.content.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &content_limits,
        );

        layout::Node::with_children(max_size, vec![content_layout])
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let full_bounds = layout.bounds();

        if let Some(content_layout) = layout.children().next() {
            let content_size = content_layout.bounds().size();
            let menu_bounds = self.calculate_menu_bounds(full_bounds, content_size.height);

            // Draw menu background with shadow
            renderer.fill_quad(
                renderer::Quad {
                    bounds: menu_bounds,
                    border: Border {
                        color: self.border_color,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: self.shadow_color,
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                },
                self.background,
            );

            // Translate cursor for content
            let translated_cursor = if let Some(pos) = cursor.position() {
                if menu_bounds.contains(pos) {
                    mouse::Cursor::Available(Point::new(
                        pos.x - menu_bounds.x,
                        pos.y - menu_bounds.y,
                    ))
                } else {
                    mouse::Cursor::Unavailable
                }
            } else {
                mouse::Cursor::Unavailable
            };

            // Draw content with clipping to menu bounds (inset to respect rounded corners)
            let border_radius = 8.0;
            let content_clip_bounds = Rectangle {
                x: menu_bounds.x + border_radius * 0.5,
                y: menu_bounds.y + border_radius * 0.5,
                width: menu_bounds.width - border_radius,
                height: menu_bounds.height - border_radius,
            };

            renderer.with_layer(content_clip_bounds, |renderer| {
                renderer.with_translation(
                    Vector::new(menu_bounds.x - full_bounds.x, menu_bounds.y - full_bounds.y),
                    |renderer| {
                        self.content.as_widget().draw(
                            &tree.children[0],
                            renderer,
                            theme,
                            style,
                            content_layout,
                            translated_cursor,
                            viewport,
                        );
                    },
                );
            });
        }
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        if let Some(content_layout) = layout.children().next() {
            self.content.as_widget().operate(
                &mut tree.children[0],
                content_layout,
                renderer,
                operation,
            );
        }
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> iced::event::Status {
        let full_bounds = layout.bounds();

        if let Some(content_layout) = layout.children().next() {
            let content_size = content_layout.bounds().size();
            let menu_bounds = self.calculate_menu_bounds(full_bounds, content_size.height);

            // Handle mouse button press
            if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = &event {
                if let Some(pos) = cursor.position() {
                    if menu_bounds.contains(pos) {
                        // Click inside menu - forward to content
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - menu_bounds.x,
                            pos.y - menu_bounds.y,
                        ));

                        let status = self.content.as_widget_mut().on_event(
                            &mut tree.children[0],
                            event,
                            content_layout,
                            translated_cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );

                        return status;
                    } else {
                        // Click outside menu - close it
                        if let Some(ref on_close) = self.on_close {
                            shell.publish(on_close.clone());
                        }
                        return iced::event::Status::Captured;
                    }
                }
            }

            // Handle mouse button release inside menu
            if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = &event {
                if let Some(pos) = cursor.position() {
                    if menu_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - menu_bounds.x,
                            pos.y - menu_bounds.y,
                        ));

                        return self.content.as_widget_mut().on_event(
                            &mut tree.children[0],
                            event,
                            content_layout,
                            translated_cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }
            }

            // For cursor moved events, always forward if in bounds (for hover effects)
            if let Event::Mouse(mouse::Event::CursorMoved { .. }) = &event {
                if let Some(pos) = cursor.position() {
                    if menu_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - menu_bounds.x,
                            pos.y - menu_bounds.y,
                        ));

                        return self.content.as_widget_mut().on_event(
                            &mut tree.children[0],
                            event,
                            content_layout,
                            translated_cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }
            }
        }

        iced::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let full_bounds = layout.bounds();

        if let Some(content_layout) = layout.children().next() {
            let content_size = content_layout.bounds().size();
            let menu_bounds = self.calculate_menu_bounds(full_bounds, content_size.height);

            if let Some(pos) = cursor.position() {
                if menu_bounds.contains(pos) {
                    let translated_cursor = mouse::Cursor::Available(Point::new(
                        pos.x - menu_bounds.x,
                        pos.y - menu_bounds.y,
                    ));

                    return self.content.as_widget().mouse_interaction(
                        &tree.children[0],
                        content_layout,
                        translated_cursor,
                        viewport,
                        renderer,
                    );
                }
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message, Renderer> From<ContextMenu<'a, Message, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(menu: ContextMenu<'a, Message, Renderer>) -> Self {
        Element::new(menu)
    }
}
