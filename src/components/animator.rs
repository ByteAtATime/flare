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
}

impl<'a, Message> From<Scaler<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    fn from(scaler: Scaler<'a, Message>) -> Self {
        Self::new(scaler)
    }
}
