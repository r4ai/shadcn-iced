//! Display additional information or actions to an input.
//!
//! Port of <https://ui.shadcn.com/docs/components/input-group>.
//!
//! ```no_run
//! # use iced_ui::input::input;
//! # use iced_ui::input_group::input_group;
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Search(String) }
//! # fn view(query: &str) -> iced::Element<'_, Message> {
//! input_group(input("Search...", query).on_input(Message::Search))
//!     .leading_icon(lucide::SEARCH)
//!     .trailing_text("12 results")
//!     .into()
//! # }
//! ```
use super::input::Input;

use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::focus::{self, FocusWithin};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme, border};

/// An input with addons. See the [module documentation](self).
pub struct InputGroup<'a, Message: Clone> {
    input: Input<'a, Message>,
    leading: Vec<Element<'a, Message>>,
    trailing: Vec<Element<'a, Message>>,
    invalid: bool,
    width: Length,
}

/// Creates an [`InputGroup`] around an [`Input`].
pub fn input_group<'a, Message: Clone + 'a>(input: Input<'a, Message>) -> InputGroup<'a, Message> {
    InputGroup {
        input,
        leading: Vec::new(),
        trailing: Vec::new(),
        invalid: false,
        width: Length::Fill,
    }
}

fn muted_text<'a, Message: 'a>(content: String) -> Element<'a, Message> {
    text(content)
        .size(style::text::SM)
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
        .into()
}

fn muted_icon<'a, Message: 'a>(icon: Icon) -> Element<'a, Message> {
    icon.sized(16)
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
        .into()
}

impl<'a, Message: Clone + 'a> InputGroup<'a, Message> {
    /// Adds an element before the input.
    pub fn leading(mut self, addon: impl Into<Element<'a, Message>>) -> Self {
        self.leading.push(addon.into());
        self
    }

    /// Adds an element after the input.
    pub fn trailing(mut self, addon: impl Into<Element<'a, Message>>) -> Self {
        self.trailing.push(addon.into());
        self
    }

    /// Adds a muted icon before the input.
    pub fn leading_icon(self, icon: Icon) -> Self {
        self.leading(muted_icon(icon))
    }

    /// Adds a muted icon after the input.
    pub fn trailing_icon(self, icon: Icon) -> Self {
        self.trailing(muted_icon(icon))
    }

    /// Adds muted text before the input.
    pub fn leading_text(self, content: impl Into<String>) -> Self {
        self.leading(muted_text(content.into()))
    }

    /// Adds muted text after the input.
    pub fn trailing_text(self, content: impl Into<String>) -> Self {
        self.trailing(muted_text(content.into()))
    }

    /// Marks the group as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: Clone + 'a> From<InputGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: InputGroup<'a, Message>) -> Self {
        let invalid = group.invalid;
        let has_leading = !group.leading.is_empty();
        let has_trailing = !group.trailing.is_empty();

        let input = group
            .input
            .chromeless(true)
            .padding(Padding {
                top: 7.0,
                bottom: 7.0,
                left: if has_leading { 8.0 } else { 12.0 },
                right: if has_trailing { 8.0 } else { 12.0 },
            })
            .width(Length::Fill);

        let addons = |items: Vec<Element<'a, Message>>, padding: Padding| {
            (!items.is_empty()).then(|| {
                container(
                    Row::with_children(items)
                        .spacing(8)
                        .align_y(Alignment::Center),
                )
                .padding(padding)
                .center_y(Length::Fill)
            })
        };

        let row = Row::new()
            .push(addons(group.leading, Padding::ZERO.left(12)))
            .push(input)
            .push(addons(group.trailing, Padding::ZERO.right(12)))
            .align_y(Alignment::Center)
            .height(36);

        FocusWithin::new(row, move |theme, status| {
            self::style(theme, status, invalid)
        })
        .width(group.width)
        .into()
    }
}

/// The style of an input group.
pub fn style(theme: &Theme, status: focus::Status, invalid: bool) -> focus::Style {
    let t = tokens(theme);
    let p = &t.palette;

    focus::Style {
        background: Some(Background::Color(if t.is_dark() {
            alpha(p.input, 0.3)
        } else {
            Color::TRANSPARENT
        })),
        border: Border {
            color: if invalid {
                p.destructive
            } else if status.focused {
                p.ring
            } else {
                p.input
            },
            width: 1.0,
            radius: border::radius(t.radius_md()),
        },
        ring: match (status.focused, invalid) {
            (true, false) => Some(style::ring(&t, t.radius_md())),
            (true, true) => Some(style::invalid_outset_ring(&t, t.radius_md())),
            _ => None,
        },
        text_color: None,
    }
}
