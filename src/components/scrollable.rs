use iced::{Border, color, widget::scrollable};

pub fn scrollable_style() -> (
    impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style,
    scrollable::Direction,
) {
    let style_fn = |theme: &iced::Theme, status: scrollable::Status| {
        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                // TODO: where does this color come from? is it related to the theme?
                background: color!(0x8d8d8d).into(),
                border: Border::default().rounded(3.0),
            },
        };

        scrollable::Style {
            container: Default::default(),
            vertical_rail: rail,
            ..scrollable::default(theme, status)
        }
    };

    let scrollbar = scrollable::Scrollbar::new().width(7.0);
    let direction = scrollable::Direction::Vertical(scrollbar);

    (style_fn, direction)
}
