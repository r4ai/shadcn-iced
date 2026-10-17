//! Display additional information or actions to an input.
//!
//! Port of <https://ui.shadcn.com/docs/components/input-group>.
//!
//! ```no_run
//! # use shadcn_iced::input::input;
//! # use shadcn_iced::input_group::input_group;
//! # use shadcn_iced_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Search(String) }
//! # fn view(query: &str) -> iced::Element<'_, Message> {
//! input_group(input("Search...", query).on_input(Message::Search))
//!     .leading_icon(lucide::SEARCH)
//!     .trailing_text("12 results")
//!     .into()
//! # }
//! ```
use super::button::{self as button, Button};
use super::input::Input;

use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::focus::{self, FocusWithin};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme, border};

/// What an addon holds, which adjusts its padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Text and icons: `pl-2` / `pr-2`.
    Plain,
    /// `has-[>button]:ml-[-0.3rem]`
    Button,
    /// `has-[>kbd]:ml-[-0.15rem]`
    Kbd,
}

impl Kind {
    fn padding(self) -> f32 {
        match self {
            Kind::Plain => 8.0,
            Kind::Button => 8.0 - 4.8,
            Kind::Kbd => 8.0 - 2.4,
        }
    }
}

/// An input with addons. See the [module documentation](self).
pub struct InputGroup<'a, Message: Clone> {
    input: Input<'a, Message>,
    leading: Vec<Element<'a, Message>>,
    trailing: Vec<Element<'a, Message>>,
    leading_kind: Kind,
    trailing_kind: Kind,
    invalid: bool,
    disabled: bool,
    width: Length,
}

/// Creates an [`InputGroup`] around an [`Input`].
pub fn input_group<'a, Message: Clone + 'a>(input: Input<'a, Message>) -> InputGroup<'a, Message> {
    InputGroup {
        input,
        leading: Vec::new(),
        trailing: Vec::new(),
        leading_kind: Kind::Plain,
        trailing_kind: Kind::Plain,
        invalid: false,
        disabled: false,
        width: Length::Fill,
    }
}

/// `InputGroupText`: muted `text-sm`, `font-medium` from the addon.
fn muted_text<'a, Message: 'a>(content: String) -> Element<'a, Message> {
    text(content)
        .size(style::text::SM)
        .font(font::medium())
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
        .into()
}

/// A muted `size-4` icon.
fn muted_icon<'a, Message: 'a>(icon: Icon) -> Element<'a, Message> {
    icon.sized(16)
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
        .into()
}

/// Creates an `InputGroupButton`: a ghost `h-6 px-1.5 text-sm` button.
/// Change its variant with [`Button::variant`].
pub fn group_button<'a, Message: Clone + 'a>(label: impl Into<String>) -> Button<'a, Message> {
    button::button(label)
        .variant(button::Variant::Ghost)
        .size(button::Size::Xs)
        .padding_x(6.0)
        .text_size(style::text::SM)
}

/// Creates an icon-only `InputGroupButton` (`size="icon-xs"`).
pub fn group_icon_button<'a, Message: Clone + 'a>(icon: Icon) -> Button<'a, Message> {
    button::icon_button(icon)
        .variant(button::Variant::Ghost)
        .size(button::Size::IconXs)
}

impl<'a, Message: Clone + 'a> InputGroup<'a, Message> {
    /// Adds an element before the input (`align="inline-start"`).
    pub fn leading(mut self, addon: impl Into<Element<'a, Message>>) -> Self {
        self.leading.push(addon.into());
        self
    }

    /// Adds an element after the input (`align="inline-end"`).
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

    /// Adds a button before the input, pulled towards the edge
    /// (`has-[>button]:ml-[-0.3rem]`). See [`group_button`].
    pub fn leading_button(mut self, button: Button<'a, Message>) -> Self {
        self.leading_kind = Kind::Button;
        self.leading(button)
    }

    /// Adds a button after the input, pulled towards the edge
    /// (`has-[>button]:mr-[-0.3rem]`). See [`group_button`].
    pub fn trailing_button(mut self, button: Button<'a, Message>) -> Self {
        self.trailing_kind = Kind::Button;
        self.trailing(button)
    }

    /// Adds a key after the input (`has-[>kbd]:mr-[-0.15rem]`).
    pub fn trailing_kbd(mut self, label: impl Into<String>) -> Self {
        self.trailing_kind = Kind::Kbd;
        self.trailing(super::kbd::kbd(label.into()))
    }

    /// Marks the group as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Dims the group (`has-disabled:opacity-50 has-disabled:bg-input/50`).
    /// Leave the input without `on_input` to disable it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let disabled = group.disabled;
        let has_leading = !group.leading.is_empty();
        let has_trailing = !group.trailing.is_empty();

        // `px-2.5 py-1`, `pl-1.5` / `pr-1.5` next to an addon, in a `h-8`
        // group.
        let input = group
            .input
            .chromeless(true)
            .padding(Padding {
                top: 5.0,
                bottom: 5.0,
                left: if has_leading { 6.0 } else { 10.0 },
                right: if has_trailing { 6.0 } else { 10.0 },
            })
            .width(Length::Fill);

        // `gap-2 py-1.5`
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
            .push(addons(
                group.leading,
                Padding::ZERO.left(group.leading_kind.padding()),
            ))
            .push(input)
            .push(addons(
                group.trailing,
                Padding::ZERO.right(group.trailing_kind.padding()),
            ))
            .align_y(Alignment::Center)
            .height(30);

        FocusWithin::new(row, move |theme, status| {
            self::style(theme, status, invalid, disabled)
        })
        .padding(1)
        .width(group.width)
        .into()
    }
}

/// The style of an input group.
pub fn style(theme: &Theme, status: focus::Status, invalid: bool, disabled: bool) -> focus::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let radius = t.radius_lg();
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    // `dark:bg-input/30`, `has-disabled:bg-input/50 dark:has-disabled:bg-input/80`
    let background = match (disabled, dark) {
        (true, false) => alpha(p.input, 0.5),
        (true, true) => alpha(p.input, 0.8),
        (false, true) => alpha(p.input, 0.3),
        (false, false) => Color::TRANSPARENT,
    };

    focus::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        border: Border {
            color: alpha(
                if invalid {
                    p.destructive
                } else if status.focused {
                    p.ring
                } else {
                    p.input
                },
                opacity,
            ),
            width: 1.0,
            radius: border::radius(radius),
        },
        // `ring-3 ring-ring/50` when focused; `ring-destructive/20` when invalid.
        ring: match (status.focused, invalid) {
            (_, true) => Some(style::invalid_outset_ring(&t, radius)),
            (true, false) => Some(style::ring(&t, radius)),
            _ => None,
        },
        text_color: None,
    }
}
