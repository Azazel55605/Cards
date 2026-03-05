use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// A simple positioned widget that renders content at a specific screen position
pub struct Positioned<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    content: Element<'a, Message, iced::Theme, Renderer>,
    position: Point,
}

impl<'a, Message, Renderer> Positioned<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, Renderer>>,
        position: Point,
    ) -> Self {
        Self {
            content: content.into(),
            position,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for Positioned<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let content_layout = self.content.as_widget().layout(
            &mut tree.children[0],
            renderer,
            limits,
        );

        // Position at the specified coordinates
        layout::Node::with_children(
            content_layout.size(),
            vec![content_layout],
        )
        .translate(Vector::new(self.position.x, self.position.y))
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
        let child_layout = layout.children().next().unwrap();
        // Use the union of the actual viewport and the child's own bounds so that
        // widgets positioned off-screen (e.g. card icons on a panned canvas) are
        // never culled by the renderer's viewport scissor test.
        let expanded_viewport = viewport.union(&child_layout.bounds());
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child_layout,
            cursor,
            &expanded_viewport,
        );
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
    ) -> iced::advanced::graphics::core::event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Positioned<'a, Message, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(positioned: Positioned<'a, Message, Renderer>) -> Self {
        Element::new(positioned)
    }
}


