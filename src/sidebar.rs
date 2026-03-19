use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::gradient;
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Radians, Rectangle, Shadow, Size, Vector};

pub struct Sidebar<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    content: Element<'a, Message, iced::Theme, Renderer>,
    floating_button: Option<Element<'a, Message, iced::Theme, Renderer>>,
    width: f32,
    background: Color,
    accent: Color,
    shadow: Color,
    offset: f32,
    /// Border color for the floating button pill (optional — defaults to transparent)
    pill_border: Color,
}

impl<'a, Message, Renderer> Sidebar<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
        width: f32,
        background: Color,
        accent: Color,
        shadow: Color,
        offset: f32,
    ) -> Self {
        Self {
            content: content.into(),
            floating_button: None,
            width,
            background,
            accent,
            shadow,
            offset,
            pill_border: Color::TRANSPARENT,
        }
    }

    pub fn pill_border(mut self, color: Color) -> Self {
        self.pill_border = color;
        self
    }

    // ...existing code...

    pub fn floating_button(mut self, button: impl Into<Element<'a, Message, iced::Theme, Renderer>>) -> Self {
        self.floating_button = Some(button.into());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for Sidebar<'a, Message, Renderer>
where
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
            Size::new(self.width, max_size.height - 30.0),
        );

        let content_layout = self.content.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &content_limits,
        );

        let mut children = vec![content_layout];

        if self.floating_button.is_some() {
            let button_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(40.0, 40.0),
            );

            let button_layout = self.floating_button.as_ref().unwrap().as_widget().layout(
                &mut tree.children[1],
                renderer,
                &button_limits,
            );

            children.push(button_layout);
        }

        layout::Node::with_children(max_size, children)
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
        let mut children = layout.children();

        // Calculate sidebar position
        let sidebar_x = 15.0 + self.offset;
        let sidebar_y = 15.0;
        let sidebar_height = full_bounds.height - 30.0;

        let sidebar_bounds = Rectangle {
            x: sidebar_x,
            y: sidebar_y,
            width: self.width,
            height: sidebar_height,
        };

        // Only draw sidebar if it's visible
        if sidebar_x + self.width > 0.0 {
            // Use a layer to ensure sidebar renders on top of canvas (like settings modal)
            renderer.with_layer(full_bounds, |renderer| {
                // Build vertical gradient: background color at top → subtle accent tint at bottom
                let gradient = gradient::Linear::new(Radians(std::f32::consts::PI)) // top → bottom
                    .add_stop(0.0, Color {
                        r: self.background.r * (1.0 - self.accent.a) + self.accent.r * self.accent.a,
                        g: self.background.g * (1.0 - self.accent.a) + self.accent.g * self.accent.a,
                        b: self.background.b * (1.0 - self.accent.a) + self.accent.b * self.accent.a,
                        a: 1.0,
                    })
                    .add_stop(1.0, self.background);

                // Draw background with shadow and gradient
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: sidebar_bounds,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 12.0.into(),
                        },
                        shadow: Shadow {
                            color: self.shadow,
                            offset: Vector::new(4.0, 4.0),
                            blur_radius: 12.0,
                        },
                    },
                    iced::Background::Gradient(iced::Gradient::Linear(gradient)),
                );

                // Draw content
                if let Some(content_layout) = children.next() {
                    // Translate cursor for content
                    let translated_cursor = if let Some(pos) = cursor.position() {
                        if sidebar_bounds.contains(pos) {
                            mouse::Cursor::Available(Point::new(
                                pos.x - sidebar_x,
                                pos.y - sidebar_y,
                            ))
                        } else {
                            cursor
                        }
                    } else {
                        cursor
                    };

                    renderer.with_translation(
                        Vector::new(sidebar_x, sidebar_y),
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
                }
            });
        } else {
            // Skip content layout
            children.next();
        }

        // Draw floating button (pill) when sidebar is hidden
        if self.floating_button.is_some() && sidebar_x + self.width < 0.0 {
            if let Some(button_layout) = children.next() {
                let button_x = 15.0;
                let button_y = full_bounds.height - 40.0 - 14.0;

                let button_bounds = Rectangle {
                    x: button_x,
                    y: button_y,
                    width: 40.0,
                    height: 40.0,
                };

                // Draw pill background behind the button (matches card shelf / zoom bar style)
                renderer.with_layer(full_bounds, |renderer| {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: button_bounds,
                            border: Border {
                                color: self.pill_border,
                                width: 1.0,
                                radius: 10.0.into(),
                            },
                            shadow: Shadow {
                                color: self.shadow,
                                offset: Vector::new(0.0, 4.0),
                                blur_radius: 12.0,
                            },
                        },
                        self.background,
                    );

                    // Draw button content on top of pill
                    let translated_cursor = if let Some(pos) = cursor.position() {
                        if button_bounds.contains(pos) {
                            mouse::Cursor::Available(Point::new(
                                pos.x - button_x,
                                pos.y - button_y,
                            ))
                        } else {
                            cursor
                        }
                    } else {
                        cursor
                    };

                    renderer.with_translation(
                        Vector::new(button_x, button_y),
                        |renderer| {
                            self.floating_button.as_ref().unwrap().as_widget().draw(
                                &tree.children[1],
                                renderer,
                                theme,
                                style,
                                button_layout,
                                translated_cursor,
                                viewport,
                            );
                        },
                    );
                });
            }
        }
    }

    fn children(&self) -> Vec<widget::Tree> {
        let mut children = vec![widget::Tree::new(&self.content)];
        if let Some(ref button) = self.floating_button {
            children.push(widget::Tree::new(button));
        }
        children
    }

    fn diff(&self, tree: &mut widget::Tree) {
        if let Some(ref button) = self.floating_button {
            tree.diff_children(&[&self.content, button]);
        } else {
            tree.diff_children(&[&self.content]);
        }
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        let mut children = layout.children();

        if let Some(content_layout) = children.next() {
            self.content.as_widget().operate(
                &mut tree.children[0],
                content_layout,
                renderer,
                operation,
            );
        }

        if self.floating_button.is_some() {
            if let Some(button_layout) = children.next() {
                self.floating_button.as_ref().unwrap().as_widget().operate(
                    &mut tree.children[1],
                    button_layout,
                    renderer,
                    operation,
                );
            }
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
        let mut children = layout.children();

        // Calculate sidebar position
        let sidebar_x = 15.0 + self.offset;
        let sidebar_y = 15.0;
        let sidebar_height = full_bounds.height - 30.0;

        let sidebar_bounds = Rectangle {
            x: sidebar_x,
            y: sidebar_y,
            width: self.width,
            height: sidebar_height,
        };

        // Handle sidebar content events
        if sidebar_x + self.width > 0.0 {
            if let Some(content_layout) = children.next() {
                if let Some(pos) = cursor.position() {
                    if sidebar_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - sidebar_x,
                            pos.y - sidebar_y,
                        ));

                        let status = self.content.as_widget_mut().on_event(
                            &mut tree.children[0],
                            event.clone(),
                            content_layout,
                            translated_cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );

                        if status == iced::event::Status::Captured {
                            return status;
                        }
                    }
                }
            }
        } else {
            children.next();
        }

        // Handle floating button events
        if self.floating_button.is_some() && sidebar_x + self.width < 0.0 {
            if let Some(button_layout) = children.next() {
                let button_x = 15.0;
                let button_y = full_bounds.height - 40.0 - 14.0;

                let button_bounds = Rectangle {
                    x: button_x,
                    y: button_y,
                    width: 40.0,
                    height: 40.0,
                };

                if let Some(pos) = cursor.position() {
                    if button_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - button_x,
                            pos.y - button_y,
                        ));

                        let status = self.floating_button.as_mut().unwrap().as_widget_mut().on_event(
                            &mut tree.children[1],
                            event,
                            button_layout,
                            translated_cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );

                        if status == iced::event::Status::Captured {
                            return status;
                        }
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
        let mut children = layout.children();

        // Calculate sidebar position
        let sidebar_x = 15.0 + self.offset;
        let sidebar_y = 15.0;
        let sidebar_height = full_bounds.height - 30.0;

        let sidebar_bounds = Rectangle {
            x: sidebar_x,
            y: sidebar_y,
            width: self.width,
            height: sidebar_height,
        };

        // Check sidebar content
        if sidebar_x + self.width > 0.0 {
            if let Some(content_layout) = children.next() {
                if let Some(pos) = cursor.position() {
                    if sidebar_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - sidebar_x,
                            pos.y - sidebar_y,
                        ));

                        let interaction = self.content.as_widget().mouse_interaction(
                            &tree.children[0],
                            content_layout,
                            translated_cursor,
                            viewport,
                            renderer,
                        );

                        if interaction != mouse::Interaction::default() {
                            return interaction;
                        }
                    }
                }
            }
        } else {
            children.next();
        }

        // Check floating button
        if self.floating_button.is_some() && sidebar_x + self.width < 0.0 {
            if let Some(button_layout) = children.next() {
                let button_x = 15.0;
                let button_y = full_bounds.height - 40.0 - 14.0;

                let button_bounds = Rectangle {
                    x: button_x,
                    y: button_y,
                    width: 40.0,
                    height: 40.0,
                };

                if let Some(pos) = cursor.position() {
                    if button_bounds.contains(pos) {
                        let translated_cursor = mouse::Cursor::Available(Point::new(
                            pos.x - button_x,
                            pos.y - button_y,
                        ));

                        return self.floating_button.as_ref().unwrap().as_widget().mouse_interaction(
                            &tree.children[1],
                            button_layout,
                            translated_cursor,
                            viewport,
                            renderer,
                        );
                    }
                }
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message, Renderer> From<Sidebar<'a, Message, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(sidebar: Sidebar<'a, Message, Renderer>) -> Self {
        Element::new(sidebar)
    }
}
