use iced::{Element, widget::text};

use crate::{components::types::DetailProps, message::Message};

pub fn render_detail(props: DetailProps) -> Element<'static, Message> {
    text(props.markdown).into()
}
