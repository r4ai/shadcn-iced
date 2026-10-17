//! A set of checkable buttons, known as radio buttons, where no more than
//! one of the buttons can be checked at a time.
//!
//! Port of <https://ui.shadcn.com/docs/components/radio-group>.
//!
//! ```no_run
//! # use shadcn_iced::radio_group::radio_group;
//! # #[derive(Clone)] enum Message { Density(u8) }
//! # fn view<'a>(density: u8) -> iced::Element<'a, Message> {
//! radio_group(Some(density), Message::Density)
//!     .option(0, "Default")
//!     .option(1, "Comfortable")
//!     .option(2, "Compact")
//!     .into()
//! # }
//! ```
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::primitive::focus::{self, FocusWithin};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Color, Font, Padding, Theme, border, mouse};

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
    invalid: bool,
    font: Font,
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
        invalid: false,
        font: font::medium(),
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

    /// Marks every option as invalid (`aria-invalid`), which also colors
    /// the labels (`data-invalid` on the fields).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the font of the labels. Defaults to `font-medium`; use
    /// [`font::sans()`] for `font-normal`.
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
}

/// Creates a single radio item, for custom layouts.
pub fn radio_item<'a, Message: Clone + 'a>(
    selected: bool,
    label: impl Into<String>,
    on_select: Option<Message>,
) -> Element<'a, Message> {
    item(
        selected,
        label.into(),
        None,
        on_select,
        false,
        font::medium(),
    )
}

fn item<'a, Message: Clone + 'a>(
    selected: bool,
    label: String,
    description: Option<String>,
    on_select: Option<Message>,
    invalid: bool,
    font: Font,
) -> Element<'a, Message> {
    let disabled = on_select.is_none();
    let has_description = description.is_some();
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    // `has-[>[data-slot=field-content]]:[&>[role=radio]]:mt-px`
    let indicator = container(indicator(selected, disabled, invalid)).padding(Padding {
        top: if has_description { 1.0 } else { 0.0 },
        ..Padding::ZERO
    });

    let label = text(label)
        .size(style::text::SM)
        .font(font)
        // `Label`: `leading-none`; `FieldLabel`: `leading-snug`
        .line_height(if has_description { 1.375 } else { 1.0 })
        .style(move |theme: &Theme| {
            let p = tokens(theme).palette;

            text::Style {
                color: Some(alpha(
                    if invalid { p.destructive } else { p.foreground },
                    opacity,
                )),
            }
        });

    // `FieldContent`: `gap-0.5`
    let labels = Column::new()
        .push(label)
        .push(description.map(|description| {
            text(description)
                .size(style::text::SM)
                .line_height(1.5)
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(tokens(theme).palette.muted_foreground, opacity)),
                })
        }))
        .spacing(2);

    // `flex items-center gap-3` around a `Label`, or a horizontal `Field`
    // (`gap-2 items-start`) with a description.
    Pressable::new(
        Row::new()
            .push(indicator)
            .push(labels)
            .spacing(if has_description { 8 } else { 12 })
            .align_y(if has_description {
                Alignment::Start
            } else {
                Alignment::Center
            }),
    )
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
        let invalid = group.invalid;
        let font = group.font;

        let items = group
            .options
            .into_iter()
            .map(|(value, label, description, disabled)| {
                let on_select = (!disabled && !group.disabled).then(|| (group.on_select)(value));

                item(
                    group.selected == Some(value),
                    label,
                    description,
                    on_select,
                    invalid,
                    font,
                )
            });

        // `grid gap-2`
        match group.orientation {
            Orientation::Vertical => Column::with_children(items).spacing(8).into(),
            Orientation::Horizontal => Row::with_children(items).spacing(16).into(),
        }
    }
}

/// The radio circle alone (`size-4 rounded-full border`), with its dot when
/// selected and its `ring-3` when invalid.
pub fn indicator<'a, Message: 'a>(
    selected: bool,
    disabled: bool,
    invalid: bool,
) -> Element<'a, Message> {
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    // `bg-primary-foreground size-2 rounded-full`
    let dot = selected.then(|| {
        container(space().width(8).height(8)).style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(alpha(
                tokens(theme).palette.primary_foreground,
                opacity,
            ))),
            border: border::rounded(f32::MAX),
            ..container::Style::default()
        })
    });

    FocusWithin::new(container(dot).center(16), move |theme, _status| {
        self::style(theme, selected, disabled, invalid)
    })
    .into()
}

/// The style of the circle of a radio item.
pub fn style(theme: &Theme, selected: bool, disabled: bool, invalid: bool) -> focus::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };
    let radius = 8.0;

    let (background, border) = if selected {
        // `data-checked:bg-primary data-checked:border-primary`
        (p.primary, p.primary)
    } else {
        (
            // `dark:bg-input/30`
            if dark {
                alpha(p.input, 0.3)
            } else {
                Color::TRANSPARENT
            },
            // `aria-invalid:border-destructive dark:aria-invalid:border-destructive/50`
            match (invalid, dark) {
                (true, false) => p.destructive,
                (true, true) => alpha(p.destructive, 0.5),
                (false, _) => p.input,
            },
        )
    };

    focus::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        border: style::border(alpha(border, opacity), radius),
        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`
        ring: invalid.then(|| {
            let mut ring = style::invalid_outset_ring(&t, radius);
            ring.color = alpha(ring.color, opacity);
            ring
        }),
        text_color: None,
    }
}
