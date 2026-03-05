use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};

pub struct Overlay<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    base: Element<'a, Message, iced::Theme, Renderer>,
    overlay: Element<'a, Message, iced::Theme, Renderer>,
    modal: bool,
    /// Message fired when the user clicks the dimmed backdrop (outside the overlay content).
    /// Only used when `modal = true`.
    on_backdrop_press: Option<Message>,
}

impl<'a, Message, Renderer> Overlay<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        base: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
        overlay: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
    ) -> Self {
        Self {
            base: base.into(),
            overlay: overlay.into(),
            modal: false,
            on_backdrop_press: None,
        }
    }

    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// Set a message to emit when the user clicks the dim backdrop outside the modal panel.
    pub fn on_backdrop_press(mut self, msg: Message) -> Self {
        self.on_backdrop_press = Some(msg);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for Overlay<'a, Message, Renderer>
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
        let size = limits.max();

        let base_layout = self.base.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, size),
        );

        let overlay_layout = self.overlay.as_widget().layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::new(Size::ZERO, size),
        );

        layout::Node::with_children(size, vec![base_layout, overlay_layout])
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
        let mut children = layout.children();

        // Draw base layer (dot grid and previous overlays)
        if let Some(base_layout) = children.next() {
            self.base.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                base_layout,
                cursor,
                viewport,
            );
        }

        // Draw overlay layer on top.
        // For modal overlays use with_layer(viewport) so the content always
        // composites above every previously-drawn layer (sidebar, AppMenu etc.).
        // For plain positioned overlays (card icons, toolbars) draw normally so
        // the viewport is not clamped to a small/off-screen scissor rect.
        if let Some(overlay_layout) = children.next() {
            if self.modal {
                renderer.with_layer(*viewport, |renderer| {
                    self.overlay.as_widget().draw(
                        &tree.children[1],
                        renderer,
                        theme,
                        style,
                        overlay_layout,
                        cursor,
                        viewport,
                    );
                });
            } else {
                self.overlay.as_widget().draw(
                    &tree.children[1],
                    renderer,
                    theme,
                    style,
                    overlay_layout,
                    cursor,
                    viewport,
                );
            }
        }
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.base),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.base, &self.overlay]);
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        let mut children = layout.children();

        if let Some(base_layout) = children.next() {
            self.base
                .as_widget()
                .operate(&mut tree.children[0], base_layout, renderer, operation);
        }

        if let Some(overlay_layout) = children.next() {
            self.overlay
                .as_widget()
                .operate(&mut tree.children[1], overlay_layout, renderer, operation);
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
        let mut children = layout.children();
        let base_layout = children.next().unwrap();
        let overlay_layout = children.next().unwrap();

        // Forward keyboard events to overlay first (for text editing)
        if matches!(event, Event::Keyboard(_)) {
            let status = self.overlay.as_widget_mut().on_event(
                &mut tree.children[1],
                event.clone(),
                overlay_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if status == iced::event::Status::Captured {
                return status;
            }
        }

        // For mouse events, always let the overlay try first.
        if matches!(event, Event::Mouse(_)) {
            let overlay_status = self.overlay.as_widget_mut().on_event(
                &mut tree.children[1],
                event.clone(),
                overlay_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if overlay_status == iced::event::Status::Captured {
                return overlay_status;
            }

            if self.modal {
                // The overlay child didn't capture this event.
                // For a ButtonPressed: check if the cursor is outside the panel bounds.
                // If outside → backdrop click → fire on_backdrop_press.
                // Either way, swallow the event so it never reaches the canvas.
                if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
                    if let Some(pos) = cursor.position() {
                        // Find the actual panel bounds — the overlay child's layout
                        // has the full window size, but the panel itself is centered
                        // and fixed-size. Walk into the first child that is smaller
                        // than the full viewport to find the real panel rectangle.
                        let panel_bounds = find_panel_bounds(overlay_layout);
                        let outside = !panel_bounds.contains(pos);
                        if outside {
                            if let Some(msg) = self.on_backdrop_press.take() {
                                shell.publish(msg);
                            }
                        }
                    }
                }
                return iced::event::Status::Captured;
            }

            // Non-modal: pass to base
            return self.base.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                base_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        // For other events, forward to overlay first
        let overlay_status = self.overlay.as_widget_mut().on_event(
            &mut tree.children[1],
            event.clone(),
            overlay_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if overlay_status == iced::event::Status::Captured {
            return overlay_status;
        }

        // Then try base
        self.base.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            base_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let base_layout = children.next();
        let overlay_layout = children.next();

        // Check overlay first
        if let Some(overlay_layout) = overlay_layout {
            let interaction = self.overlay.as_widget().mouse_interaction(
                &tree.children[1],
                overlay_layout,
                cursor,
                viewport,
                renderer,
            );

            if interaction != mouse::Interaction::default() {
                return interaction;
            }
        }

        // Then check base
        if let Some(base_layout) = base_layout {
            return self.base.as_widget().mouse_interaction(
                &tree.children[0],
                base_layout,
                cursor,
                viewport,
                renderer,
            );
        }

        mouse::Interaction::default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, Renderer>> {
        let mut children = layout.children();
        let _base_layout = children.next();
        let overlay_layout = children.next();

        // Forward overlay requests from the overlay element (which contains SettingsModal)
        if let Some(overlay_layout) = overlay_layout {
            self.overlay.as_widget_mut().overlay(
                &mut tree.children[1],
                overlay_layout,
                renderer,
                translation,
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Renderer> From<Overlay<'a, Message, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(overlay: Overlay<'a, Message, Renderer>) -> Self {
        Element::new(overlay)
    }
}

/// Walk the layout tree to find the actual panel rectangle.
///
/// The overlay element is laid out with `Size::FILL` (full window), so its
/// root bounds equal the whole viewport.  The real modal card/panel is a
/// fixed-size child somewhere inside.  We do a breadth-first search for the
/// first child whose bounds are strictly smaller than the root bounds — that
/// is the panel.  If nothing smaller is found we fall back to the root bounds
/// (so clicks are never mis-identified as backdrop clicks).
fn find_panel_bounds(layout: Layout<'_>) -> Rectangle {
    let root = layout.bounds();
    let mut queue: Vec<Layout<'_>> = layout.children().collect();
    while !queue.is_empty() {
        let mut next = Vec::new();
        for child in &queue {
            let b = child.bounds();
            // "Strictly smaller" on both axes — the panel is never as wide or
            // as tall as the full window.
            if b.width < root.width - 1.0 && b.height < root.height - 1.0 {
                return b;
            }
            next.extend(child.children());
        }
        queue = next;
    }
    root
}

