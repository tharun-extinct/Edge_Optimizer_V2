/// ICED theme and styling
use iced::widget::{button, container, text_input};
use iced::{theme, Background, Color, Theme};

const BG_BASE: Color = Color::from_rgb(0.07, 0.08, 0.10);
const BG_SIDEBAR: Color = Color::from_rgb(0.10, 0.11, 0.14);
const BG_CARD: Color = Color::from_rgb(0.13, 0.14, 0.18);
const BG_ELEVATED: Color = Color::from_rgb(0.16, 0.17, 0.22);
const ACCENT: Color = Color::from_rgb(0.00, 0.71, 0.84);
const ACCENT_HOVER: Color = Color::from_rgb(0.02, 0.60, 0.95);
const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.95, 0.98);
const TEXT_MUTED: Color = Color::from_rgb(0.66, 0.72, 0.79);
const BORDER_SUBTLE: Color = Color::from_rgba(0.60, 0.70, 0.82, 0.28);

#[allow(dead_code)]
pub fn theme() -> Theme {
    Theme::custom(
        "Edge Neon".to_string(),
        theme::Palette {
            background: BG_BASE,
            text: TEXT_PRIMARY,
            primary: ACCENT,
            success: Color::from_rgb(0.25, 0.85, 0.55),
            danger: Color::from_rgb(0.92, 0.28, 0.30),
        },
    )
}

pub fn root() -> impl Fn(&Theme) -> container::Appearance + Copy {
    |_| container::Appearance {
        background: Some(Background::Color(BG_BASE)),
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn sidebar() -> impl Fn(&Theme) -> container::Appearance + Copy {
    |_| container::Appearance {
        background: Some(Background::Color(BG_SIDEBAR)),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 14.0.into(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn panel_card() -> impl Fn(&Theme) -> container::Appearance + Copy {
    |_| container::Appearance {
        background: Some(Background::Color(BG_CARD)),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 14.0.into(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn status_bar() -> impl Fn(&Theme) -> container::Appearance + Copy {
    |_| container::Appearance {
        background: Some(Background::Color(BG_ELEVATED)),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn nav_button(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |_theme: &Theme, status: button::Status| {
        let base = if active { ACCENT } else { BG_ELEVATED };
        let hover = if active { ACCENT_HOVER } else { Color::from_rgb(0.21, 0.23, 0.29) };
        let bg = match status {
            button::Status::Hovered => hover,
            button::Status::Pressed => ACCENT_HOVER,
            button::Status::Disabled => Color::from_rgba(base.r, base.g, base.b, 0.35),
            button::Status::Active => base,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: if active { ACCENT_HOVER } else { BORDER_SUBTLE },
                width: if active { 1.5 } else { 1.0 },
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn primary_button() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => ACCENT_HOVER,
            button::Status::Pressed => Color::from_rgb(0.01, 0.52, 0.81),
            button::Status::Disabled => Color::from_rgba(ACCENT.r, ACCENT.g, ACCENT.b, 0.35),
            button::Status::Active => ACCENT,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: ACCENT_HOVER,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn subtle_button() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgb(0.21, 0.23, 0.29),
            button::Status::Pressed => Color::from_rgb(0.17, 0.18, 0.24),
            button::Status::Disabled => Color::from_rgba(BG_ELEVATED.r, BG_ELEVATED.g, BG_ELEVATED.b, 0.35),
            button::Status::Active => BG_ELEVATED,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn danger_button() -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |_theme: &Theme, status: button::Status| {
        let active = Color::from_rgb(0.60, 0.22, 0.24);
        let hover = Color::from_rgb(0.74, 0.24, 0.28);
        let bg = match status {
            button::Status::Hovered => hover,
            button::Status::Pressed => Color::from_rgb(0.52, 0.18, 0.20),
            button::Status::Disabled => Color::from_rgba(active.r, active.g, active.b, 0.35),
            button::Status::Active => active,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: Color::from_rgb(0.82, 0.26, 0.30),
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn text_input() -> impl Fn(&Theme, text_input::Status) -> text_input::Style + Copy {
    move |_theme: &Theme, status: text_input::Status| {
        let border_color = match status {
            text_input::Status::Active => BORDER_SUBTLE,
            text_input::Status::Hovered => Color::from_rgba(ACCENT.r, ACCENT.g, ACCENT.b, 0.7),
            text_input::Status::Focused => ACCENT_HOVER,
            text_input::Status::Disabled => Color::from_rgba(BORDER_SUBTLE.r, BORDER_SUBTLE.g, BORDER_SUBTLE.b, 0.45),
        };

        text_input::Style {
            background: Background::Color(BG_ELEVATED),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 10.0.into(),
            },
            icon: TEXT_MUTED,
            placeholder: TEXT_MUTED,
            value: TEXT_PRIMARY,
            selection: Color::from_rgba(ACCENT.r, ACCENT.g, ACCENT.b, 0.35),
        }
    }
}
