use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::gradient;
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Radians, Rectangle, Shadow, Size, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsCategory {
    #[default]
    General,
    Appearance,
    Shortcuts,
    About,
}

impl SettingsCategory {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Shortcuts => "Shortcuts",
            SettingsCategory::About => "About",
        }
    }

    pub fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::General,
            SettingsCategory::Appearance,
            SettingsCategory::Shortcuts,
            SettingsCategory::About,
        ]
    }
}

pub struct SettingsModal<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    content: Element<'a, Message, iced::Theme, Renderer>,
    width: f32,
    height: f32,
    background: Color,
    accent: Color,
    border_radius: f32,
    shadow_color: Color,
    overlay_color: Color,
    on_close: Option<Box<dyn Fn() -> Message + 'a>>,
    scale: f32,
}

impl<'a, Message, Renderer> SettingsModal<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
        background: Color,
        accent: Color,
        shadow_color: Color,
    ) -> Self {
        Self {
            content: content.into(),
            width: 600.0,
            height: 400.0,
            background,
            accent,
            border_radius: 12.0,
            shadow_color,
            overlay_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            on_close: None,
            scale: 1.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Message + 'a,
    {
        self.on_close = Some(Box::new(f));
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for SettingsModal<'a, Message, Renderer>
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

        // Modal content area
        let content_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(self.width, self.height),
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
        let bounds = layout.bounds();

        // Apply scale to dimensions
        let scaled_width = self.width * self.scale;
        let scaled_height = self.height * self.scale;

        // Calculate centered modal position (accounting for scale)
        let modal_x = (bounds.width - scaled_width) / 2.0;
        let modal_y = (bounds.height - scaled_height) / 2.0;

        let modal_bounds = Rectangle {
            x: modal_x,
            y: modal_y,
            width: scaled_width,
            height: scaled_height,
        };

        // Adjust overlay opacity based on scale (fade in/out with animation)
        let overlay_alpha = self.overlay_color.a * self.scale;
        let animated_overlay_color = Color {
            a: overlay_alpha,
            ..self.overlay_color
        };

        // Use a layer to ensure modal renders on top of canvas
        renderer.with_layer(bounds, |renderer| {
            // Draw semi-transparent overlay over entire screen
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default(),
                    shadow: Shadow::default(),
                },
                animated_overlay_color,
            );

            // Draw modal background with shadow and diagonal gradient
            let blended = Color {
                r: self.background.r * (1.0 - self.accent.a) + self.accent.r * self.accent.a,
                g: self.background.g * (1.0 - self.accent.a) + self.accent.g * self.accent.a,
                b: self.background.b * (1.0 - self.accent.a) + self.accent.b * self.accent.a,
                a: 1.0,
            };
            // Diagonal gradient: 135° = top-left → bottom-right
            let gradient = gradient::Linear::new(Radians(std::f32::consts::PI * 0.75))
                .add_stop(0.0, self.background)
                .add_stop(1.0, blended);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: modal_bounds,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: self.border_radius.into(),
                    },
                    shadow: Shadow {
                        color: Color { a: self.shadow_color.a * self.scale, ..self.shadow_color },
                        offset: Vector::new(0.0, 8.0 * self.scale),
                        blur_radius: 24.0 * self.scale,
                    },
                },
                iced::Background::Gradient(iced::Gradient::Linear(gradient)),
            );

            // Translate cursor for content
            let translated_cursor = if let Some(pos) = cursor.position() {
                if modal_bounds.contains(pos) {
                    mouse::Cursor::Available(iced::Point::new(
                        pos.x - modal_x,
                        pos.y - modal_y,
                    ))
                } else {
                    mouse::Cursor::Unavailable
                }
            } else {
                mouse::Cursor::Unavailable
            };

            // Draw content
            if let Some(content_layout) = layout.children().next() {
                renderer.with_translation(
                    Vector::new(modal_x, modal_y),
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

        // Calculate modal position (centered)
        let modal_x = (full_bounds.width - self.width) / 2.0;
        let modal_y = (full_bounds.height - self.height) / 2.0;

        let modal_bounds = Rectangle {
            x: full_bounds.x + modal_x,
            y: full_bounds.y + modal_y,
            width: self.width,
            height: self.height,
        };

        // Check if cursor is within modal
        if let Some(pos) = cursor.position() {
            if modal_bounds.contains(pos) {
                let translated_cursor = mouse::Cursor::Available(iced::Point::new(
                    pos.x - full_bounds.x - modal_x,
                    pos.y - full_bounds.y - modal_y,
                ));

                if let Some(content_layout) = layout.children().next() {
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

                    // Return the status from the content widget
                    // This allows pick_list and other interactive widgets to work
                    return status;
                }
            } else {
                // Click is outside the modal
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
                    // Emit close message if configured
                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close());
                    }
                    return iced::event::Status::Captured;
                }
            }
        }

        // Only capture mouse events outside the modal to prevent background interaction
        match event {
            Event::Mouse(_) => iced::event::Status::Captured,
            _ => iced::event::Status::Ignored,
        }
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

        let modal_x = (full_bounds.width - self.width) / 2.0;
        let modal_y = (full_bounds.height - self.height) / 2.0;

        let modal_bounds = Rectangle {
            x: full_bounds.x + modal_x,
            y: full_bounds.y + modal_y,
            width: self.width,
            height: self.height,
        };

        if let Some(pos) = cursor.position() {
            if modal_bounds.contains(pos) {
                let translated_cursor = mouse::Cursor::Available(iced::Point::new(
                    pos.x - full_bounds.x - modal_x,
                    pos.y - full_bounds.y - modal_y,
                ));

                if let Some(content_layout) = layout.children().next() {
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, Renderer>> {
        let full_bounds = layout.bounds();
        let modal_x = (full_bounds.width - self.width) / 2.0;
        let modal_y = (full_bounds.height - self.height) / 2.0;

        if let Some(content_layout) = layout.children().next() {
            // Calculate the translation for the modal content
            let modal_translation = Vector::new(
                translation.x + modal_x,
                translation.y + modal_y,
            );

            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                content_layout,
                renderer,
                modal_translation,
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Renderer> From<SettingsModal<'a, Message, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(modal: SettingsModal<'a, Message, Renderer>) -> Self {
        Element::new(modal)
    }
}
