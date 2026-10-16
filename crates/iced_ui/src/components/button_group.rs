//! A container that groups related buttons together with consistent styling.
//!
//! Port of <https://ui.shadcn.com/docs/components/button-group>.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::button_group::button_group;
//! # #[derive(Clone)] enum Message { Archive, Report }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! button_group([
//!     button("Archive").variant(Variant::Outline).on_press(Message::Archive),
//!     button("Report").variant(Variant::Outline).on_press(Message::Report),
//! ])
//! .into()
//! # }
//! ```
//!
//! Groups also hold text segments ([`ButtonGroup::text`]) and separators
//! ([`ButtonGroup::separator`]). Put several groups in a row with `gap-2`
//! to nest them.
use super::button::Button;

use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, space, text};
use iced::{Background, Length, Theme};

/// The direction of a [`ButtonGroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Side by side.
    #[default]
    Horizontal,
    /// Stacked.
    Vertical,
}

enum Item<'a, Message> {
    Button(Button<'a, Message>),
    Text(String),
    Separator,
}

/// A group of joined buttons. See the [module documentation](self).
pub struct ButtonGroup<'a, Message> {
    items: Vec<Item<'a, Message>>,
    orientation: Orientation,
}

/// Creates a [`ButtonGroup`].
pub fn button_group<'a, Message: Clone + 'a>(
    buttons: impl IntoIterator<Item = Button<'a, Message>>,
) -> ButtonGroup<'a, Message> {
    ButtonGroup {
        items: buttons.into_iter().map(Item::Button).collect(),
        orientation: Orientation::Horizontal,
    }
}

impl<'a, Message: Clone + 'a> ButtonGroup<'a, Message> {
    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Appends a button.
    pub fn push(mut self, button: Button<'a, Message>) -> Self {
        self.items.push(Item::Button(button));
        self
    }

    /// Appends a text segment (`ButtonGroupText`).
    pub fn text(mut self, label: impl Into<String>) -> Self {
        self.items.push(Item::Text(label.into()));
        self
    }

    /// Appends a separator (`ButtonGroupSeparator`), e.g. between two
    /// secondary buttons.
    pub fn separator(mut self) -> Self {
        self.items.push(Item::Separator);
        self
    }
}

impl<'a, Message: Clone + 'a> From<ButtonGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: ButtonGroup<'a, Message>) -> Self {
        let count = group.items.len();
        let vertical = group.orientation == Orientation::Vertical;
        // Adjacent borders overlap, like `border-l-0`, unless a separator
        // stands between the (borderless) buttons.
        let spacing = if group
            .items
            .iter()
            .any(|item| matches!(item, Item::Separator))
        {
            0.0
        } else {
            -1.0
        };

        let items = group.items.into_iter().enumerate().map(|(index, item)| {
            let position = style::Position::of(index, count);

            match item {
                Item::Button(button) => Element::from(button.position(position, vertical)),
                Item::Text(label) => segment(label, position, vertical),
                Item::Separator => separator(vertical),
            }
        });

        // `items-stretch`
        match group.orientation {
            Orientation::Horizontal => Row::with_children(items)
                .spacing(spacing)
                .height(Length::Shrink)
                .into(),
            Orientation::Vertical => Column::with_children(items)
                .spacing(spacing)
                .width(Length::Shrink)
                .into(),
        }
    }
}

/// A text segment of a button group (`ButtonGroupText`): `bg-muted gap-2
/// rounded-lg border px-2.5 text-sm font-medium`.
pub fn group_text<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    segment(label, style::Position::Alone, false)
}

fn segment<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    position: style::Position,
    vertical: bool,
) -> Element<'a, Message> {
    container(text(label).size(style::text::SM).font(font::medium()))
        .padding([0, 10])
        .center_y(Length::Fixed(32.0))
        .style(move |theme: &Theme| {
            let t = tokens(theme);
            let radius = t.radius_lg();
            let mut border = style::border(t.palette.border, radius);

            border.radius = if vertical {
                position.vertical_radius(radius)
            } else {
                position.radius(radius)
            };

            container::Style {
                background: Some(Background::Color(t.palette.muted)),
                border,
                ..container::Style::default()
            }
        })
        .into()
}

/// `ButtonGroupSeparator`: a `bg-input` line, inset by `my-px`.
fn separator<'a, Message: 'a>(vertical: bool) -> Element<'a, Message> {
    let line = container(space())
        .width(if vertical {
            Length::Fill
        } else {
            Length::Fixed(1.0)
        })
        .height(if vertical {
            Length::Fixed(1.0)
        } else {
            Length::Fill
        })
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).palette.input)),
            ..container::Style::default()
        });

    container(line)
        .padding(if vertical { [0, 1] } else { [1, 0] })
        .width(if vertical {
            Length::Fill
        } else {
            Length::Fixed(1.0)
        })
        .height(if vertical {
            Length::Fixed(1.0)
        } else {
            Length::Fill
        })
        .into()
}
