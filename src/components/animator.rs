use iced::advanced::Renderer as _; // using for with_layer trait impl only
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::{Element, Length, Rectangle, Renderer, Size, Theme, Transformation, mouse};

pub struct Scaler<'a, Message> {
    scale: f32,
    content: Element<'a, Message>,
}

impl<'a, Message> Scaler<'a, Message> {
    pub fn new(scale: f32, content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            scale,
            content: content.into(),
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Scaler<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.with_layer(bounds, |renderer| {
            renderer.with_transformation(
                Transformation::translate(bounds.x + bounds.width, bounds.y + bounds.height)
                    * Transformation::scale(self.scale)
                    * Transformation::translate(
                        -bounds.x - bounds.width,
                        -bounds.y - bounds.height,
                    ),
                |renderer| {
                    self.content
                        .as_widget()
                        .draw(tree, renderer, theme, style, layout, cursor, viewport);
                },
            );
        });
    }

    fn mouse_interaction(
        &self,
        state: &widget::Tree,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'a, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(state, layout, renderer, viewport, translation)
    }

    fn operate(
        &mut self,
        state: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(state, layout, renderer, operation);
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn update(
        &mut self,
        state: &mut widget::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            state, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }
}

impl<'a, Message> From<Scaler<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    fn from(scaler: Scaler<'a, Message>) -> Self {
        Self::new(scaler)
    }
}
