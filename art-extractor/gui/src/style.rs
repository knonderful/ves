#![allow(unused)]

use iced::{button, container, Background, Color, Vector, Text};
use iced::container::Style;

pub const FONT_SIZE: u16 = 16;

const SURFACE: Color = Color::from_rgb(
    0xF2 as f32 / 255.0,
    0xF3 as f32 / 255.0,
    0xF5 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);

const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

pub struct AppBackgroundStyle;

impl container::StyleSheet for AppBackgroundStyle {
    fn style(&self) -> Style {
        container::Style {
            background: Some(Color::from_rgb8(0x10, 0x10, 0x10).into()),
            ..Default::default()
        }
    }
}

pub enum PaneTitleBarStyle {
    Unfocused,
    Focused,
}

impl container::StyleSheet for PaneTitleBarStyle {
    fn style(&self) -> container::Style {
        let pane = match self {
            Self::Unfocused => PaneStyle::Unfocused,
            Self::Focused => PaneStyle::Focused,
        }
            .style();

        container::Style {
            text_color: Some(Color::from_rgb8(0xb2, 0xb2, 0xb2)),
            background: Some(pane.border_color.into()),
            ..Default::default()
        }
    }
}

pub enum PaneStyle {
    Unfocused,
    Focused,
}

impl container::StyleSheet for PaneStyle {
    fn style(&self) -> container::Style {
        let background = match self {
            PaneStyle::Unfocused => Color::from_rgb8(0x3c, 0x3f, 0x41),
            PaneStyle::Focused => Color::from_rgb8(0x4e, 0x52, 0x54),
        };

        container::Style {
            background: Some(Color::from_rgb8(0x31, 0x33, 0x35).into()),
            border_width: 1.0,
            border_color: background,
            ..Default::default()
        }
    }
}

pub enum ButtonStyle {
    Primary,
    Destructive,
    Control,
    Pin,
}

impl button::StyleSheet for ButtonStyle {
    fn active(&self) -> button::Style {
        let (background, text_color) = match self {
            ButtonStyle::Primary => (Some(ACTIVE), Color::WHITE),
            ButtonStyle::Destructive => {
                (None, Color::from_rgb8(0xFF, 0x47, 0x47))
            }
            ButtonStyle::Control => (Some(PANE_ID_COLOR_FOCUSED), Color::WHITE),
            ButtonStyle::Pin => (Some(ACTIVE), Color::WHITE),
        };

        button::Style {
            text_color,
            background: background.map(Background::Color),
            border_radius: 5.0,
            shadow_offset: Vector::new(0.0, 0.0),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        let background = match self {
            ButtonStyle::Primary => Some(HOVERED),
            ButtonStyle::Destructive => Some(Color {
                a: 0.2,
                ..active.text_color
            }),
            ButtonStyle::Control => Some(PANE_ID_COLOR_FOCUSED),
            ButtonStyle::Pin => Some(HOVERED),
        };

        button::Style {
            background: background.map(Background::Color),
            ..active
        }
    }
}

pub fn form_label(text: impl Into<String>) -> Text {
    Text::new(text)
        .size(FONT_SIZE)
        .color(Color::from_rgb8(0x80, 0x80, 0x80))
}