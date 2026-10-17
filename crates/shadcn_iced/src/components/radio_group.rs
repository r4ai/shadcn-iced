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
use shadcn_iced_core::primitive::focus::{self, FocusWithin};
use shadcn_iced_core::primitive::roving::Keys;
use shadcn_iced_core::primitive::{Labelled, Pressable, Roving};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Color, Font, Padding, Theme, border};

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
        true,
    )
}

fn item<'a, Message: Clone + 'a>(
    selected: bool,
    label: String,
    description: Option<String>,
    on_select: Option<Message>,
    invalid: bool,
    font: Font,
    tab_stop: bool,
) -> Element<'a, Message> {
    let disabled = on_select.is_none();
    let has_description = description.is_some();
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    // `has-[>[data-slot=field-content]]:[&>[role=radio]]:mt-px`
    let indicator = container(control(selected, invalid, on_select, tab_stop)).padding(Padding {
        top: if has_description { 1.0 } else { 0.0 },
        ..Padding::ZERO
    });

    // `FieldLabel`: `text-sm leading-snug`
    let label = text(label)
        .size(style::text::SM)
        .font(font)
        .line_height(1.375)
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

    // A horizontal `Field` (`gap-2 items-center`, `items-start` with a
    // description), whose label selects the radio (`htmlFor`).
    Labelled::new(
        Row::new()
            .push(indicator)
            .push(labels)
            .spacing(8)
            .align_y(if has_description {
                Alignment::Start
            } else {
                Alignment::Center
            }),
    )
    .into()
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> From<RadioGroup<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(group: RadioGroup<'a, T, Message>) -> Self {
        let invalid = group.invalid;
        let font = group.font;
        let enabled = |disabled: bool| !disabled && !group.disabled;

        // Roving focus: only the selected radio, or else the first enabled
        // one, is in the tab order.
        let tab_stop = group
            .options
            .iter()
            .position(|(value, _, _, disabled)| {
                enabled(*disabled) && group.selected == Some(*value)
            })
            .or_else(|| {
                group
                    .options
                    .iter()
                    .position(|(_, _, _, disabled)| enabled(*disabled))
            });

        let items = group.options.into_iter().enumerate().map(
            |(index, (value, label, description, disabled))| {
                let on_select = enabled(disabled).then(|| (group.on_select)(value));

                item(
                    group.selected == Some(value),
                    label,
                    description,
                    on_select,
                    invalid,
                    font,
                    tab_stop == Some(index),
                )
            },
        );

        // `grid gap-2`; the arrow keys move the focus and select.
        let items: Element<'a, Message> = match group.orientation {
            Orientation::Vertical => Column::with_children(items).spacing(8).into(),
            Orientation::Horizontal => Row::with_children(items).spacing(16).into(),
        };

        Roving::new(items)
            .keys(Keys::Both)
            .activate_on_focus(true)
            .into()
    }
}

/// The radio circle alone (`size-4 rounded-full border`), with its dot when
/// selected and its `ring-3` when invalid.
pub fn indicator<'a, Message: 'a>(
    selected: bool,
    disabled: bool,
    invalid: bool,
) -> Element<'a, Message> {
    FocusWithin::new(dot(selected), move |theme, _status| {
        self::style(theme, selected, disabled, invalid)
    })
    .into()
}

/// The `size-4` content of the circle: its dot when selected
/// (`bg-primary-foreground size-2 rounded-full`).
fn dot<'a, Message: 'a>(selected: bool) -> Element<'a, Message> {
    let dot = selected.then(|| {
        container(space().width(8).height(8)).style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).palette.primary_foreground)),
            border: border::rounded(f32::MAX),
            ..container::Style::default()
        })
    });

    container(dot).center(16).into()
}

/// The radio circle as a control: focusable, selected by a click or
/// <kbd>Space</kbd>, with the `focus-visible:ring-3` of shadcn/ui. It
/// produces `on_select`; without it, the control is disabled. A `tab_stop`
/// control is reached with <kbd>Tab</kbd>; in a group, only one is.
pub fn control<'a, Message: Clone + 'a>(
    selected: bool,
    invalid: bool,
    on_select: Option<Message>,
    tab_stop: bool,
) -> Element<'a, Message> {
    let disabled = on_select.is_none();

    Pressable::new(dot(selected))
        .on_press_maybe(on_select)
        .disabled(disabled)
        .focusable(true)
        .tab_stop(tab_stop)
        .activate_on_enter(false)
        .focus_style(move |theme, _status, focused| {
            let style = focused_style(theme, selected, disabled, invalid, focused);

            iced::widget::button::Style {
                background: style.background,
                border: style.border,
                ..iced::widget::button::Style::default()
            }
        })
        .ring(move |theme, _status, focused| {
            focused_style(theme, selected, disabled, invalid, focused).ring
        })
        .into()
}

/// The style of the circle of a radio item, when its keyboard focus is
/// visible or not.
pub fn focused_style(
    theme: &Theme,
    selected: bool,
    disabled: bool,
    invalid: bool,
    focused: bool,
) -> focus::Style {
    let mut style = self::style(theme, selected, disabled, invalid);

    if focused {
        let t = tokens(theme);

        // `focus-visible:border-ring`, below `data-checked:border-primary`
        // and `aria-invalid:border-destructive`.
        if !selected && !invalid {
            style.border.color = t.palette.ring;
        }

        // `focus-visible:ring-3 focus-visible:ring-ring/50`, below
        // `aria-invalid:ring-destructive/20`.
        if style.ring.is_none() {
            style.ring = Some(style::ring(&t, 8.0));
        }
    }

    style
}

/// The style of the circle of a radio item.
///
/// Disabled radios look enabled: Base UI radios are not `<button>`s, so
/// their `disabled:opacity-50` never applies (only their labels fade).
pub fn style(theme: &Theme, selected: bool, _disabled: bool, invalid: bool) -> focus::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
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
        background: Some(Background::Color(background)),
        border: style::border(border, radius),
        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`
        ring: invalid.then(|| style::invalid_outset_ring(&t, radius)),
        text_color: None,
    }
}
