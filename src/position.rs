use iced::advanced::widget::operation::{Operation, Outcome};
use iced::advanced::widget::{self, operate};
use iced::{Rectangle, Task};
use std::any::Any;
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

/// Create a [`Task`](iced::Task) to find the position of a specific child by index
/// within the widget identified by the given [`Id`].
pub fn find_position(target: Id, index: usize) -> Task<Option<Rectangle>> {
    struct FindPosition {
        target: Id,
        index: usize,
        found_bounds: Option<Rectangle>,
    }

    impl Operation<Option<Rectangle>> for FindPosition {
        fn traverse(
            &mut self,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Option<Rectangle>>),
        ) {
            operate_on_children(self);
        }

        fn container(&mut self, id: Option<&widget::Id>, _bounds: Rectangle) {
            if Some(&self.target.0) == id {
                return;
            }
        }

        fn custom(&mut self, id: Option<&widget::Id>, _bounds: Rectangle, state: &mut dyn Any) {
            if Some(&self.target.0) == id {
                if let Some(position_state) = state.downcast_mut::<PositionState>() {
                    self.found_bounds = position_state.as_position().get(self.index);
                }
            }
        }

        fn finish(&self) -> Outcome<Option<Rectangle>> {
            Outcome::Some(self.found_bounds)
        }
    }

    operate(FindPosition {
        target,
        index,
        found_bounds: None,
    })
}
