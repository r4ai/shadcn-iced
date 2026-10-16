//! An input where the user selects a value from within a given range.
//!
//! Port of <https://ui.shadcn.com/docs/components/slider>.
//!
//! ```no_run
//! # use iced_ui::slider::slider;
//! # #[derive(Clone)] enum Message { Volume(f32) }
//! # fn view<'a>(volume: f32) -> iced::Element<'a, Message> {
//! slider(0.0..=100.0, volume, Message::Volume).into()
//! # }
//! ```
use iced_ui_core::theme::alpha;
use iced_ui_core::tokens;

use iced::widget::slider::{Handle, HandleShape, Rail, Status, Style};
use iced::widget::{Slider, VerticalSlider, vertical_slider};
use iced::{Background, Color, Length, Theme, border};

use std::ops::RangeInclusive;

/// Creates a horizontal slider (`h-1.5` track, `size-4` thumb).
pub fn slider<'a, T, Message: Clone + 'a>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd + 'a,
    f64: From<T>,
{
    Slider::new(range, value, on_change)
        .height(16)
        .width(Length::Fill)
        .style(style)
}

/// Creates a vertical slider.
pub fn vertical<'a, T, Message: Clone + 'a>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> VerticalSlider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd + 'a,
    f64: From<T>,
{
    vertical_slider(range, value, on_change)
        .width(16)
        .height(176)
        .style(style)
}

/// The style of a slider.
pub fn style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let active = matches!(status, Status::Hovered | Status::Dragged);

    Style {
        rail: Rail {
            backgrounds: (Background::Color(p.primary), Background::Color(p.muted)),
            width: 6.0,
            border: border::rounded(f32::MAX),
        },
        handle: Handle {
            shape: HandleShape::Circle { radius: 8.0 },
            background: Background::Color(Color::WHITE),
            border_width: if active { 4.0 } else { 1.0 },
            border_color: if active { alpha(p.ring, 0.5) } else { p.primary },
        },
    }
}

