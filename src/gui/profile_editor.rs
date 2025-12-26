/// Profile editor UI components
use iced::{Element, widget::{Column, Row, Text, Button, TextInput, Checkbox, Space, Container}, Length, Alignment};
use crate::gui::Message;
use crate::profile::Profile;

#[derive(Debug, Clone)]
pub struct EditingProfile {
    pub name: String,
    pub crosshair_x_offset: String,
    pub crosshair_y_offset: String,
    pub crosshair_image_path: Option<String>,
    pub overlay_enabled: bool,
}

impl Default for EditingProfile {
    fn default() -> Self {
        EditingProfile {
            name: String::new(),
            crosshair_x_offset: "0".to_string(),
            crosshair_y_offset: "0".to_string(),
            crosshair_image_path: None,
            overlay_enabled: false,
        }
    }
}

impl From<&Profile> for EditingProfile {
    fn from(profile: &Profile) -> Self {
        EditingProfile {
            name: profile.name.clone(),
            crosshair_x_offset: profile.crosshair_x_offset.to_string(),
            crosshair_y_offset: profile.crosshair_y_offset.to_string(),
            crosshair_image_path: profile.crosshair_image_path.clone(),
            overlay_enabled: profile.overlay_enabled,
        }
    }
}

pub fn render_editor(profile: &EditingProfile) -> Element<'static, Message> {
    let content = Column::new()
        .spacing(15)
        .padding(20)
        .push(
            Text::new("Profile Name")
                .size(18)
        )
        .push(
            TextInput::new("Enter profile name", &profile.name)
                .on_input(Message::ProfileNameChanged)
                .width(Length::Fill)
        )
        .push(Space::new(Length::Fill, Length::Fixed(10.0)))
        .push(
            Text::new("Crosshair Settings")
                .size(16)
        )
        .push(
            Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Column::new()
                        .spacing(10)
                        .push(Text::new("X Offset (pixels)"))
                        .push(
                            TextInput::new("-500 to 500", &profile.crosshair_x_offset)
                                .on_input(Message::CrosshairOffsetXChanged)
                                .width(Length::Fixed(150.0))
                        )
                )
                .push(
                    Column::new()
                        .spacing(10)
                        .push(Text::new("Y Offset (pixels)"))
                        .push(
                            TextInput::new("-500 to 500", &profile.crosshair_y_offset)
                                .on_input(Message::CrosshairOffsetYChanged)
                                .width(Length::Fixed(150.0))
                        )
                )
        )
        .push(Space::new(Length::Fill, Length::Fixed(10.0)))
        .push(
            Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Column::new()
                        .spacing(10)
                        .push(Text::new("Crosshair Image (100x100 PNG)"))
                        .push(
                            Button::new(Text::new("Select Image"))
                                .on_press(Message::SelectImage)
                                .padding(10)
                        )
                )
                .push(
                    if let Some(path) = &profile.crosshair_image_path {
                        Text::new(format!("✓ {}", path)).size(12)
                    } else {
                        Text::new("No image selected").size(12)
                    }
                )
        )
        .push(Space::new(Length::Fill, Length::Fixed(10.0)))
        .push(
            Checkbox::new("Enable crosshair overlay", profile.overlay_enabled)
                .on_toggle(Message::OverlayEnabledToggled)
        );

    Container::new(content)
        .width(Length::Fill)
        .into()
}

