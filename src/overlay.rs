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
    overlay: Element<'a, Message, iced::Theme, Renderer>
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
            overlay: overlay.into()
        }
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

        // Draw overlay layer on top
        if let Some(overlay_layout) = children.next() {
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

        // For mouse events, forward to overlay first, then to base if not captured
        if matches!(event, Event::Mouse(_)) {
            if let Some(cursor_position) = cursor.position() {
                if overlay_layout.bounds().contains(cursor_position) {
                    // Cursor is over overlay bounds, let overlay try to handle it
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
                    // If overlay ignored it, fall through to send to base
                }
                // Cursor is NOT over overlay bounds OR overlay ignored the event
                // Send to base (canvas) to handle
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
