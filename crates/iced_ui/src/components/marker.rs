//! A small inline annotation in a conversation: a date, a status, an event.
//!
//! Port of <https://ui.shadcn.com/docs/components/marker>.
//!
//! ```no_run
//! # use iced_ui::marker::{marker, Variant};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! marker("Today").variant(Variant::Separator).into()
//! # }
//! ```
use iced_ui_core::icon::Icon;
use iced_ui_core::{Element, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Length, Theme};

/// The visual style of a [`Marker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Muted text, optionally with an icon.
    #[default]
    Default,
    /// Centered text between two lines.
    Separator,
    /// Text above a bottom border.
    Border,
}

/// A marker. See the [module documentation](self).
pub struct Marker {
    label: String,
    icon: Option<Icon>,
    variant: Variant,
}

/// Creates a [`Marker`].
pub fn marker(label: impl Into<String>) -> Marker {
    Marker {
        label: label.into(),
        icon: None,
        variant: Variant::Default,
    }
}

impl Marker {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
}

impl<'a, Message: 'a> From<Marker> for Element<'a, Message> {
    fn from(marker: Marker) -> Self {
        let muted = |theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        };

        let content = Row::new()
            .push(marker.icon.map(|icon| icon.sized(16).style(muted)))
            .push(text(marker.label).size(style::text::SM).style(muted))
            .spacing(8)
            .align_y(Alignment::Center);

        match marker.variant {
            Variant::Default => container(content).width(Length::Fill).into(),
            Variant::Separator => Row::new()
                .push(super::separator::separator())
                .push(container(content).padding([0, 4]))
                .push(super::separator::separator())
                .spacing(4)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .into(),
            Variant::Border => Column::new()
                .push(container(content).padding(iced::Padding::ZERO.bottom(8)))
                .push(super::separator::separator())
                .width(Length::Fill)
                .into(),
        }
    }
}
