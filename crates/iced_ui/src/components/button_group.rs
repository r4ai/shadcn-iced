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
use super::button::Button;

use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
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

/// A group of joined buttons. See the [module documentation](self).
pub struct ButtonGroup<'a, Message> {
    buttons: Vec<Button<'a, Message>>,
    orientation: Orientation,
}

/// Creates a [`ButtonGroup`].
pub fn button_group<'a, Message: Clone + 'a>(
    buttons: impl IntoIterator<Item = Button<'a, Message>>,
) -> ButtonGroup<'a, Message> {
    ButtonGroup {
        buttons: buttons.into_iter().collect(),
        orientation: Orientation::Horizontal,
    }
}

impl<'a, Message: Clone + 'a> ButtonGroup<'a, Message> {
    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<'a, Message: Clone + 'a> From<ButtonGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: ButtonGroup<'a, Message>) -> Self {
        let count = group.buttons.len();
        let vertical = group.orientation == Orientation::Vertical;

        let buttons = group.buttons.into_iter().enumerate().map(|(index, button)| {
            Element::from(button.position(style::Position::of(index, count), vertical))
        });

        // Adjacent borders overlap, like `border-l-0`.
        match group.orientation {
            Orientation::Horizontal => Row::with_children(buttons).spacing(-1.0).into(),
            Orientation::Vertical => Column::with_children(buttons).spacing(-1.0).into(),
        }
    }
}

/// A text segment of a button group (`ButtonGroupText`).
pub fn group_text<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(label).size(style::text::SM).font(font::MEDIUM))
        .padding([0, 16])
        .height(36)
        .center_y(Length::Fixed(36.0))
        .style(|theme: &Theme| {
            let t = tokens(theme);
            container::Style {
                background: Some(Background::Color(t.palette.muted)),
                border: style::border(t.palette.border, t.radius_md()),
                ..container::Style::default()
            }
        })
        .into()
}
