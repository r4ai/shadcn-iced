//! A small inline annotation in a conversation: a date, a status, an event.
//!
//! Port of <https://ui.shadcn.com/docs/components/marker>.
//!
//! ```no_run
//! # use shadcn_iced::marker::{marker, Variant};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! marker("Today").variant(Variant::Separator).into()
//! # }
//! ```
use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::{Element, style, tokens};

use iced::widget::{Column, Row, container, rule, text};
use iced::{Alignment, Length, Padding, Theme};

/// The visual style of a [`Marker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Muted text, optionally with an icon.
    #[default]
    Default,
    /// Centered text between two lines.
    Separator,
    /// Text above a bottom border (`border-b pb-2`).
    Border,
}

enum Leading {
    None,
    Icon(Icon),
    Spinner,
}

/// A marker. See the [module documentation](self).
pub struct Marker {
    label: String,
    leading: Leading,
    variant: Variant,
    vertical: bool,
}

/// Creates a [`Marker`] (`MarkerContent`).
pub fn marker(label: impl Into<String>) -> Marker {
    Marker {
        label: label.into(),
        leading: Leading::None,
        variant: Variant::Default,
        vertical: false,
    }
}

impl Marker {
    /// Sets the icon (`MarkerIcon`, `size-4`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.leading = Leading::Icon(icon);
        self
    }

    /// Shows a spinner as icon, for a status (`role="status"`).
    pub fn spinner(mut self) -> Self {
        self.leading = Leading::Spinner;
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Stacks the icon above the content (`flex-col`).
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// `h-px flex-1 bg-border`
fn line<'a, Message: 'a>() -> Element<'a, Message> {
    rule::horizontal(1)
        .style(|theme: &Theme| rule::Style {
            color: tokens(theme).palette.border,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

impl<'a, Message: 'a> From<Marker> for Element<'a, Message> {
    fn from(marker: Marker) -> Self {
        let separator = marker.variant == Variant::Separator;

        let icon: Option<Element<'a, Message>> = match marker.leading {
            Leading::None => None,
            Leading::Icon(icon) => Some(icon.sized(16).style(muted).into()),
            Leading::Spinner => Some(super::spinner::spinner().size(16.0).into()),
        };

        // `text-sm text-muted-foreground min-h-4`
        let label = text(marker.label).size(style::text::SM).style(muted);
        let label: Element<'a, Message> = if separator {
            // `flex-none text-center`
            label.wrapping(text::Wrapping::None).into()
        } else if marker.vertical {
            label.center().into()
        } else {
            label.width(Length::Fill).into()
        };

        if marker.vertical {
            return container(
                Column::new()
                    .push(icon)
                    .push(label)
                    .spacing(8)
                    .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .into();
        }

        // `gap-2`, and `before:mr-1 after:ml-1` around the lines.
        let row = Row::new()
            .push(separator.then(|| container(line()).padding(Padding::ZERO.right(4))))
            .push(icon)
            .push(label)
            .push(separator.then(|| container(line()).padding(Padding::ZERO.left(4))))
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        match marker.variant {
            Variant::Border => Column::new()
                .push(container(row).padding(Padding::ZERO.bottom(8)))
                .push(line())
                .width(Length::Fill)
                .into(),
            _ => row.into(),
        }
    }
}
