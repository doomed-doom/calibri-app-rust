use iced::alignment::Horizontal;
use iced::widget::text::{LineHeight, Style as TextStyle};
use iced::widget::{button, container, text, Column};
use iced::{Background, Color, Element, Length, Theme, Vector};

pub fn hero_button<'a, Message: 'a>(
    label: impl Into<String>,
    text_size: f32,
    line_height: LineHeight,
) -> button::Button<'a, Message> {
    button(labeled_container(label.into(), text_size, line_height, false))
        .width(Length::Shrink)
        .padding([12, 22])
        .style(primary_button_style())
}

pub fn device_badge_button<'a, Message: 'a>(
    label: impl Into<String>,
    text_size: f32,
    line_height: LineHeight,
) -> button::Button<'a, Message> {
    button(labeled_container(label.into(), text_size, line_height, false))
        .style(device_badge_style())
        .padding([8, 24])
}

pub fn device_menu<'a, Message>(
    info_msg: Message,
    disconnect_msg: Message,
    text_size: f32,
    line_height: LineHeight,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let menu = Column::new()
        .spacing(8)
        .push(menu_button(label_info(), text_size, line_height).on_press(info_msg))
        .push(menu_button(label_disconnect(), text_size, line_height).on_press(disconnect_msg));

    container(menu.width(Length::Fixed(220.0)))
        .padding(12)
        .style(context_menu_style())
        .into()
}

pub fn device_list_entry<'a, Message: 'a>(
    label: String,
    active: bool,
    text_size: f32,
    line_height: LineHeight,
) -> button::Button<'a, Message> {
    let entry_label = container(
        text(label)
            .size(text_size)
            .line_height(line_height)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .style(device_entry_text_style(active)),
    )
    .width(Length::Fill)
    .center_x(Length::Fill);

    button(entry_label)
        .width(Length::Fill)
        .style(list_entry_button_style(active))
}

pub fn device_list_container<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content.into())
        .padding(12)
        .style(device_list_style())
}

fn labeled_container<'a, Message: 'a>(
    label: String,
    text_size: f32,
    line_height: LineHeight,
    fill_width: bool,
) -> container::Container<'a, Message> {
    let mut content = text(label)
        .size(text_size)
        .line_height(line_height)
        .align_x(Horizontal::Center);

    content = if fill_width {
        content.width(Length::Fill)
    } else {
        content.width(Length::Shrink)
    };

    let mut wrapper = container(content);

    if fill_width {
        wrapper = wrapper.width(Length::Fill).center_x(Length::Fill);
    } else {
        wrapper = wrapper.width(Length::Shrink).center_x(Length::Shrink);
    }

    wrapper
}

fn menu_button<'a, Message: 'a>(
    label: impl Into<String>,
    text_size: f32,
    line_height: LineHeight,
) -> button::Button<'a, Message> {
    button(labeled_container(label.into(), text_size, line_height, true))
        .style(menu_button_style())
}

fn label_info() -> &'static str {
    "Информация"
}

fn label_disconnect() -> &'static str {
    "Отключиться"
}

pub fn primary_button_style() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    |_, status| {
        let (color, alpha) = match status {
            button::Status::Hovered => (Color::from_rgb(0.18, 0.4, 0.95), 1.0),
            button::Status::Pressed => (Color::from_rgb(0.12, 0.3, 0.75), 1.0),
            button::Status::Disabled => (Color::from_rgba(0.3, 0.3, 0.35, 0.6), 0.5),
            button::Status::Active => (Color::from_rgb(0.14, 0.34, 0.85), 1.0),
        };

        let mut style = button::Style::default();
        style.background = Some(Background::Color(color));
        style.text_color = Color::from_rgba(1.0, 1.0, 1.0, alpha);
        style.border.radius = 999.0.into();
        style
    }
}

pub fn device_badge_style() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    |_, status| {
        let color = match status {
            button::Status::Hovered => Color::from_rgb(0.2, 0.45, 0.9),
            button::Status::Pressed => Color::from_rgb(0.15, 0.35, 0.8),
            button::Status::Disabled => Color::from_rgba(0.2, 0.35, 0.6, 0.6),
            button::Status::Active => Color::from_rgb(0.18, 0.4, 0.85),
        };
        let mut style = button::Style::default();
        style.background = Some(Background::Color(color));
        style.text_color = Color::WHITE;
        style.border.radius = 0.0.into();
        style
    }
}

pub fn menu_button_style() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    |theme, status| {
        let palette = theme.extended_palette();
        let base = palette.background.strong.color;
        let accent = palette.primary.base.color;

        let background = match status {
            button::Status::Hovered => Color {
                a: 0.25,
                ..base
            },
            button::Status::Pressed => Color {
                a: 0.35,
                ..base
            },
            button::Status::Disabled => Color {
                a: 0.15,
                ..base
            },
            button::Status::Active => Color {
                a: 0.18,
                ..base
            },
        };

        let mut text_color = Color::WHITE;
        if matches!(status, button::Status::Disabled) {
            text_color.a *= 0.7;
        }

        let mut style = button::Style::default();
        style.background = Some(Background::Color(background));
        style.text_color = if matches!(status, button::Status::Pressed) {
            accent
        } else {
            text_color
        };
        style.border.radius = 12.0.into();
        style
    }
}

pub fn device_list_style() -> impl Fn(&Theme) -> container::Style + Copy {
    |theme| {
        let mut style = container::Style::default();
        let palette = theme.extended_palette();
        let tint = palette.primary.weak.color;
        style.background = Some(Background::Color(Color {
            a: 0.12,
            ..tint
        }));
        style.border.radius = 16.0.into();
        style.border.width = 1.0;
        style.border.color = palette.primary.strong.color;
        style
    }
}

pub fn device_entry_text_style(active: bool) -> impl Fn(&Theme) -> TextStyle + Copy {
    move |theme| {
        let palette = theme.extended_palette();
        let color = if active {
            palette.primary.base.color
        } else {
            Color::WHITE
        };
        TextStyle { color: Some(color) }
    }
}

pub fn list_entry_button_style(
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme, status| {
        let palette = theme.extended_palette();
        let idle_bg = palette.background.strong.color;
        let highlight_bg = palette.primary.strong.color;
        let base_bg = if active { highlight_bg } else { idle_bg };
        let base_alpha: f32 = if active { 0.35 } else { 0.15 };

        let background = match status {
            button::Status::Hovered => Color {
                a: (base_alpha + 0.1).min(1.0),
                ..base_bg
            },
            button::Status::Pressed => Color {
                a: (base_alpha + 0.2).min(1.0),
                ..base_bg
            },
            button::Status::Disabled => Color {
                a: base_alpha * 0.4,
                ..base_bg
            },
            button::Status::Active => Color {
                a: base_alpha,
                ..base_bg
            },
        };

        let mut text_color = if active {
            palette.primary.base.color
        } else {
            Color::WHITE
        };
        if matches!(status, button::Status::Disabled) {
            text_color.a *= 0.6;
        }

        let mut style = button::Style::default();
        style.background = Some(Background::Color(background));
        style.text_color = text_color;
        style.border.radius = 12.0.into();
        style
    }
}

pub fn context_menu_style() -> impl Fn(&Theme) -> container::Style + Copy {
    |theme| {
        let mut style = container::Style::default();
        let palette = theme.extended_palette();
        style.background = Some(Background::Color(palette.background.weak.color));
        style.border.radius = 12.0.into();
        style.shadow.color = Color::from_rgba(0.0, 0.0, 0.0, 0.4);
        style.shadow.offset = Vector::new(0.0, 4.0);
        style.shadow.blur_radius = 20.0;
        style
    }
}
