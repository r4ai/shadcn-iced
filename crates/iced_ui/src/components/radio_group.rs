//! A set of checkable buttons, known as radio buttons, where no more than
//! one of the buttons can be checked at a time.
//!
//! Port of <https://ui.shadcn.com/docs/components/radio-group>.
//!
//! ```no_run
//! # use iced_ui::radio_group::radio_group;
//! # #[derive(Clone)] enum Message { Density(u8) }
//! # fn view<'a>(density: u8) -> iced::Element<'a, Message> {
//! radio_group(Some(density), Message::Density)
//!     .option(0, "Default")
//!     .option(1, "Comfortable")
//!     .option(2, "Compact")
//!     .into()
//! # }
//! ```
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Color, Theme, border, mouse};

/// The direction in which options are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// One option per line.
    #[default]
    Vertical,
    /// Options side by side.
    Horizontal,
}

/// A radio group. See the [module documentation](self).
pub struct RadioGroup<'a, T, Message> {
    selected: Option<T>,
    options: Vec<(T, String, Option<String>, bool)>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    orientation: Orientation,
    disabled: bool,
}

/// Creates a [`RadioGroup`] with the given selected value.
pub fn radio_group<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a>(
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> RadioGroup<'a, T, Message> {
    RadioGroup {
        selected,
        options: Vec::new(),
        on_select: Box::new(on_select),
        orientation: Orientation::Vertical,
        disabled: false,
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> RadioGroup<'a, T, Message> {
    /// Appends an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push((value, label.into(), None, false));
        self
    }

    /// Appends an option with a description.
    pub fn option_with_description(
        mut self,
        value: T,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.options
            .push((value, label.into(), Some(description.into()), false));
        self
    }

    /// Appends a disabled option.
    pub fn disabled_option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push((value, label.into(), None, true));
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables every option.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Creates a single radio item, for custom layouts.
pub fn radio_item<'a, Message: Clone + 'a>(
    selected: bool,
    label: impl Into<String>,
    on_select: Option<Message>,
) -> Element<'a, Message> {
    item(selected, label.into(), None, on_select)
}

fn item<'a, Message: Clone + 'a>(
    selected: bool,
    label: String,
    description: Option<String>,
    on_select: Option<Message>,
) -> Element<'a, Message> {
    let disabled = on_select.is_none();

    let dot = selected.then(|| {
        container(space().width(8).height(8)).style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(alpha(
                tokens(theme).palette.primary,
                if disabled {
                    style::DISABLED_OPACITY
                } else {
                    1.0
                },
            ))),
            border: border::rounded(f32::MAX),
            ..container::Style::default()
        })
    });

    let indicator = container(dot)
        .center(16)
        .style(move |theme| self::indicator(theme, disabled));

    let has_description = description.is_some();

    let labels = Column::new()
        .push(
            text(label)
                .size(style::text::SM)
                .font(font::MEDIUM)
                .line_height(iced::Pixels(16.0))
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(
                        tokens(theme).palette.foreground,
                        if disabled {
                            style::DISABLED_OPACITY
                        } else {
                            1.0
                        },
                    )),
                }),
        )
        .push(description.map(|description| {
            text(description)
                .size(style::text::SM)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
        }))
        .spacing(6);

    Pressable::new(Row::new().push(indicator).push(labels).spacing(12).align_y(
        if has_description {
            Alignment::Start
        } else {
            Alignment::Center
        },
    ))
    .on_press_maybe(on_select)
    .disabled(disabled)
    .interaction(if disabled {
        mouse::Interaction::NotAllowed
    } else {
        mouse::Interaction::Pointer
    })
    .style(|_, _| iced::widget::button::Style::default())
    .into()
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> From<RadioGroup<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(group: RadioGroup<'a, T, Message>) -> Self {
        let items = group
            .options
            .into_iter()
            .map(|(value, label, description, disabled)| {
                let on_select = (!disabled && !group.disabled).then(|| (group.on_select)(value));

                item(group.selected == Some(value), label, description, on_select)
            });

        match group.orientation {
            Orientation::Vertical => Column::with_children(items).spacing(12).into(),
            Orientation::Horizontal => Row::with_children(items).spacing(16).into(),
        }
    }
}

/// The style of the circle of a radio item.
pub fn indicator(theme: &Theme, disabled: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    container::Style {
        background: Some(Background::Color(if t.is_dark() {
            alpha(p.input, 0.3 * opacity)
        } else {
            Color::TRANSPARENT
        })),
        border: iced::Border {
            color: alpha(p.input, opacity),
            width: 1.0,
            radius: border::radius(f32::MAX),
        },
        ..container::Style::default()
    }
}
