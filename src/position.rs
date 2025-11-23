use iced::Rectangle;
use iced::advanced::widget;
use std::borrow::Cow;

/// The identifier of a widget that can track positions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(pub widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        let cow = id.into();
        Self(widget::Id::from(cow.into_owned()))
    }

    /// Creates a unique [`Id`].
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// The internal state for tracking widget positions.
pub trait Position {
    /// Store the position of a child widget by index.
    fn set(&mut self, index: usize, bounds: Rectangle);

    /// Get the position of a child widget by index.
    fn get(&self, index: usize) -> Option<Rectangle>;

    /// Clear all stored positions.
    fn clear(&mut self);
}

pub struct PositionState {
    position: Box<dyn Position>,
}

impl PositionState {
    pub fn new<T: Position + 'static>(position: T) -> Self {
        Self {
            position: Box::new(position),
        }
    }

    pub fn as_position(&self) -> &dyn Position {
        &*self.position
    }

    pub fn as_position_mut(&mut self) -> &mut dyn Position {
        &mut *self.position
    }
}
