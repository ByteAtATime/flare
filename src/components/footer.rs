use iced::{
    Alignment::Center,
    Color, Element, Length, Theme,
    advanced::graphics::core::SmolStr,
    keyboard::{Key, Modifiers, key::Named},
    widget::{button, column, container, row, rule, text},
};

use crate::{
    components::actions::{ActionHandler, ActionPanelItem},
    components::kbd::render_kbd,
    theme,
};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

#[derive(Debug, Clone)]
pub enum FooterMessage {
    OpenActionPanel,
    InvokeAction(ActionHandler),
}

pub fn render_footer<'a>(
    actions: &[ActionPanelItem],
    toast_message: &str,
    theme: &'a theme::Theme,
) -> Element<'a, FooterMessage> {
    let flattened_actions = actions.iter().flat_map(|item| match item {
        ActionPanelItem::Action(action) => std::slice::from_ref(action).iter(),
        ActionPanelItem::Section(section) => section.children.iter(),
    });
    let primary_action = flattened_actions.clone().next();

    let text_color = theme.colors.text;
    let bg_color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);

    let action_button = primary_action.map(|action| {
        let btn_text = text(action.title.clone())
            .font(INTER_FONT)
            .size(14)
            .line_height(1.5);
        let shortcut = render_kbd(theme, Key::Named(Named::Enter), Modifiers::empty());

        let mut btn = button(row![btn_text, shortcut].spacing(10).align_y(Center));

        if let Some(handler) = &action.handler {
            btn = btn.on_press(FooterMessage::InvokeAction(handler.clone()));
        }

        btn.style(move |_, _| button::Style {
            text_color,
            ..Default::default()
        })
    });

    let action_panel_button = if flattened_actions.count() > 1 {
        let btn_text = text("Actions")
            .font(INTER_FONT)
            .size(14)
            .line_height(1.5)
            .style(|_theme| text::Style {
                color: theme.colors.text_60.into(),
                ..Default::default()
            });
        let shortcut = render_kbd(
            theme,
            Key::Character(SmolStr::new_inline("k")),
            Modifiers::COMMAND,
        );

        Some(
            button(row![btn_text, shortcut].spacing(10).align_y(Center))
                .on_press(FooterMessage::OpenActionPanel)
                .style(move |_, _| button::Style {
                    text_color,
                    ..Default::default()
                }),
        )
    } else {
        None
    };

    let mut footer_content = row![
        text(toast_message.to_string())
            .width(Length::Fill)
            .color(text_color)
    ]
    .spacing(10)
    .height(Length::Fill)
    .align_y(iced::Alignment::Center)
    .padding([0, 8]);

    if let Some(action_button) = action_button {
        footer_content = footer_content.push(action_button);
    }

    if let Some(action_panel_button) = action_panel_button {
        footer_content = footer_content.push(action_panel_button);
    }

    let footer = column![
        rule::horizontal(1).style(|_theme| rule::Style {
            color: theme.colors.text_10,
            fill_mode: rule::FillMode::Full,
            radius: 0.into(),
            snap: true
        }),
        footer_content
    ];

    container(footer)
        .width(Length::Fill)
        .height(40)
        .style(move |_theme: &Theme| container::Style {
            background: Some(bg_color.into()),
            text_color: Some(text_color),
            ..Default::default()
        })
        .into()
}

pub fn map_footer_message(msg: FooterMessage) -> crate::Message {
    match msg {
        FooterMessage::OpenActionPanel => crate::Message::ToggleActionPanel(true),
        FooterMessage::InvokeAction(handler) => crate::Message::InvokeAction(handler),
    }
}
