/// ICED theme and styling
use iced::widget::{button, container, text_input as text_input_widget};
use iced::{theme, Background, Color, Theme};

const BG_BASE: Color = Color::from_rgb(0.07, 0.08, 0.10);
const BG_SIDEBAR: Color = Color::from_rgb(0.08, 0.10, 0.15);
const BG_CARD: Color = Color::from_rgb(0.10, 0.13, 0.19);
const BG_ELEVATED: Color = Color::from_rgb(0.13, 0.17, 0.24);
const BG_HERO: Color = Color::from_rgb(0.06, 0.16, 0.22);
const ACCENT: Color = Color::from_rgb(0.00, 0.73, 0.93);
const ACCENT_ALT: Color = Color::from_rgb(0.08, 0.50, 1.00);
const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.97, 1.00);
const TEXT_MUTED: Color = Color::from_rgb(0.63, 0.72, 0.82);
const BORDER_SUBTLE: Color = Color::from_rgba(0.45, 0.63, 0.90, 0.35);
const BORDER_FOCUS: Color = Color::from_rgba(0.25, 0.76, 0.98, 0.92);

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
            radius: 20.0.into(),
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
            radius: 18.0.into(),
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
            radius: 16.0.into(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn hero_panel() -> impl Fn(&Theme) -> container::Appearance + Copy {
    |_| container::Appearance {
        background: Some(Background::Color(BG_HERO)),
        border: iced::Border {
            color: BORDER_FOCUS,
            width: 1.0,
            radius: 22.0.into(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn nav_button(active: bool) -> theme::Button {
    theme::Button::custom(NavButton { active })
}

pub fn primary_button() -> theme::Button {
    theme::Button::custom(PrimaryButton)
}

pub fn subtle_button() -> theme::Button {
    theme::Button::custom(SubtleButton)
}

pub fn danger_button() -> theme::Button {
    theme::Button::custom(DangerButton)
}

pub fn text_input() -> theme::TextInput {
    theme::TextInput::Custom(Box::new(NeonTextInput))
}

#[derive(Clone, Copy)]
struct NavButton {
    active: bool,
}

#[derive(Clone, Copy)]
struct PrimaryButton;

#[derive(Clone, Copy)]
struct SubtleButton;

#[derive(Clone, Copy)]
struct DangerButton;

#[derive(Clone, Copy)]
struct NeonTextInput;

impl button::StyleSheet for NavButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let bg = if self.active { ACCENT_ALT } else { BG_ELEVATED };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: if self.active {
                    BORDER_FOCUS
                } else {
                    BORDER_SUBTLE
                },
                width: if self.active { 1.2 } else { 1.0 },
                radius: 14.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 6.0,
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut a = self.active(style);
        a.background = Some(Background::Color(if self.active {
            ACCENT
        } else {
            Color::from_rgb(0.16, 0.21, 0.30)
        }));
        a
    }
}

impl button::StyleSheet for PrimaryButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ACCENT)),
            text_color: Color::from_rgb(0.02, 0.05, 0.10),
            border: iced::Border {
                color: BORDER_FOCUS,
                width: 1.0,
                radius: 14.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.72, 0.95, 0.30),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut a = self.active(style);
        a.background = Some(Background::Color(ACCENT_ALT));
        a
    }
}

impl button::StyleSheet for SubtleButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(BG_ELEVATED)),
            text_color: TEXT_PRIMARY,
            border: iced::Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut a = self.active(style);
        a.background = Some(Background::Color(Color::from_rgb(0.17, 0.22, 0.32)));
        a
    }
}

impl button::StyleSheet for DangerButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.36, 0.14, 0.22))),
            text_color: Color::from_rgb(1.0, 0.92, 0.96),
            border: iced::Border {
                color: Color::from_rgb(0.84, 0.25, 0.42),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut a = self.active(style);
        a.background = Some(Background::Color(Color::from_rgb(0.48, 0.16, 0.28)));
        a
    }
}

impl text_input_widget::StyleSheet for NeonTextInput {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> text_input_widget::Appearance {
        text_input_widget::Appearance {
            background: Background::Color(BG_ELEVATED),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: BORDER_SUBTLE,
            },
            icon_color: TEXT_MUTED,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input_widget::Appearance {
        let mut a = self.active(style);
        a.border.color = BORDER_FOCUS;
        a
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        TEXT_MUTED
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        TEXT_PRIMARY
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(TEXT_MUTED.r, TEXT_MUTED.g, TEXT_MUTED.b, 0.5)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(ACCENT.r, ACCENT.g, ACCENT.b, 0.35)
    }

    fn hovered(&self, style: &Self::Style) -> text_input_widget::Appearance {
        self.focused(style)
    }

    fn disabled(&self, style: &Self::Style) -> text_input_widget::Appearance {
        let mut a = self.active(style);
        a.background = Background::Color(Color::from_rgba(
            BG_ELEVATED.r,
            BG_ELEVATED.g,
            BG_ELEVATED.b,
            0.55,
        ));
        a
    }
}
