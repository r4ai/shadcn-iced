//! A control that allows the user to toggle between checked and not checked.
//!
//! Port of <https://ui.shadcn.com/docs/components/checkbox>.
//!
//! ```no_run
//! # use shadcn_iced::checkbox::checkbox;
//! # #[derive(Clone)] enum Message { Terms(bool) }
//! # fn view<'a>(accepted: bool) -> iced::Element<'a, Message> {
//! checkbox(accepted).label("Accept terms and conditions").on_toggle(Message::Terms).into()
//! # }
//! ```
use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::focus::{self, FocusWithin};
use shadcn_iced_core::primitive::{Labelled, Pressable};
use shadcn_iced_core::theme::{alpha, mix};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Font, Padding, Theme};

/// A checkbox. See the [module documentation](self).
pub struct Checkbox<'a, Message> {
    checked: bool,
    label: Option<String>,
    description: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    disabled: bool,
    invalid: bool,
    font: Font,
}

/// Creates a [`Checkbox`].
pub fn checkbox<'a, Message: Clone + 'a>(checked: bool) -> Checkbox<'a, Message> {
    Checkbox {
        checked,
        label: None,
        description: None,
        on_toggle: None,
        disabled: false,
        invalid: false,
        font: font::medium(),
    }
}

impl<'a, Message: Clone + 'a> Checkbox<'a, Message> {
    /// Sets the label, displayed on the right (`FieldLabel`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets a description, displayed below the label (`FieldDescription`).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the message produced when the checkbox is toggled.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Disables the checkbox.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the checkbox as invalid (`aria-invalid`), which also colors the
    /// label (`data-invalid` on the field).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the font of the label. Defaults to `font-medium`; use
    /// [`font::sans()`] for `font-normal`.
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Checkbox<'a, Message>> for Element<'a, Message> {
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        let checked = checkbox.checked;
        let disabled = checkbox.disabled || checkbox.on_toggle.is_none();
        let invalid = checkbox.invalid;
        let has_description = checkbox.description.is_some();

        let opacity = if disabled {
            style::DISABLED_OPACITY
        } else {
            1.0
        };

        // `has-[>[data-slot=field-content]]:[&>[role=checkbox]]:mt-px`
        let on_toggle = checkbox
            .on_toggle
            .filter(|_| !disabled)
            .map(|f| f(!checked));

        let indicator = container(control(checked, invalid, on_toggle)).padding(Padding {
            top: if has_description { 1.0 } else { 0.0 },
            ..Padding::ZERO
        });

        let labels = (checkbox.label.is_some() || has_description).then(|| {
            let font = checkbox.font;

            // `FieldContent`: `gap-0.5`
            Column::new()
                .push(checkbox.label.map(|label| {
                    // `text-sm leading-snug font-medium`
                    text(label)
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
                        })
                }))
                .push(checkbox.description.map(|description| {
                    // `text-muted-foreground text-sm leading-normal`
                    text(description)
                        .size(style::text::SM)
                        .line_height(1.5)
                        .style(move |theme: &Theme| text::Style {
                            color: Some(alpha(tokens(theme).palette.muted_foreground, opacity)),
                        })
                }))
                .spacing(2)
        });

        // `Field orientation="horizontal"`: `gap-2 items-center`, or
        // `items-start` with a description. The labels toggle the checkbox
        // (`htmlFor`).
        Labelled::new(Row::new().push(indicator).push(labels).spacing(8).align_y(
            if has_description {
                Alignment::Start
            } else {
                Alignment::Center
            },
        ))
        .into()
    }
}

/// The checkbox square alone (`size-4 rounded-[4px] border`), with its
/// `ring-3` when invalid. It is not interactive; see [`control`].
pub fn indicator<'a, Message: 'a>(
    checked: bool,
    disabled: bool,
    invalid: bool,
) -> Element<'a, Message> {
    FocusWithin::new(check(checked), move |theme, _status| {
        self::style(theme, checked, disabled, invalid)
    })
    .into()
}

/// The checkbox square as a control: focusable, pressed by a click or
/// <kbd>Space</kbd>, with the `focus-visible:ring-3` of shadcn/ui. It
/// produces `on_toggle`; without it, the control is disabled.
///
/// Wrap it with its label in a [`Labelled`] for the label to toggle it.
pub fn control<'a, Message: Clone + 'a>(
    checked: bool,
    invalid: bool,
    on_toggle: Option<Message>,
) -> Element<'a, Message> {
    let disabled = on_toggle.is_none();

    Pressable::new(check(checked))
        .on_press_maybe(on_toggle)
        .disabled(disabled)
        .focusable(true)
        .activate_on_enter(false)
        .focus_style(move |theme, _status, focused| {
            let style = focused_style(theme, checked, disabled, invalid, focused);

            iced::widget::button::Style {
                background: style.background,
                text_color: style.text_color.unwrap_or(Color::TRANSPARENT),
                border: style.border,
                ..iced::widget::button::Style::default()
            }
        })
        .ring(move |theme, _status, focused| {
            focused_style(theme, checked, disabled, invalid, focused).ring
        })
        .into()
}

/// `[&>svg]:size-3.5`
fn check<'a, Message: 'a>(checked: bool) -> Element<'a, Message> {
    container(checked.then(|| lucide::CHECK.sized(14)))
        .center(16)
        .into()
}

/// The style of the checkbox square, when its keyboard focus is visible or
/// not.
pub fn focused_style(
    theme: &Theme,
    checked: bool,
    disabled: bool,
    invalid: bool,
    focused: bool,
) -> focus::Style {
    let mut style = self::style(theme, checked, disabled, invalid);

    if focused {
        let t = tokens(theme);

        // `focus-visible:border-ring`, below `data-checked:border-primary`
        // and `aria-invalid:border-destructive`.
        if !checked && !invalid {
            style.border.color = t.palette.ring;
        }

        // `focus-visible:ring-3 focus-visible:ring-ring/50`, below
        // `aria-invalid:ring-destructive/20`.
        if style.ring.is_none() {
            style.ring = Some(style::ring(&t, 4.0));
        }
    }

    style
}

/// The style of the checkbox square.
pub fn style(theme: &Theme, checked: bool, disabled: bool, invalid: bool) -> focus::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };
    let radius = 4.0;

    let (background, border) = if checked {
        // `data-checked:bg-primary data-checked:border-primary`, also when
        // invalid (`aria-invalid:aria-checked:border-primary`).
        (p.primary, p.primary)
    } else {
        (
            // `dark:bg-input/30`
            if dark {
                alpha(p.input, 0.3)
            } else {
                Color::TRANSPARENT
            },
            match (invalid, dark) {
                // `aria-invalid:border-destructive dark:aria-invalid:border-destructive/50`
                (true, false) => p.destructive,
                (true, true) => alpha(p.destructive, 0.5),
                (false, _) => p.input,
            },
        )
    };

    // `disabled:opacity-50` fades the checked square as a whole: its
    // border and check are opaque over its background.
    let fade = |color: Color| {
        if checked {
            mix(p.background, color, opacity)
        } else {
            alpha(color, opacity)
        }
    };

    focus::Style {
        background: Some(Background::Color(fade(background))),
        border: style::border(fade(border), radius),
        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`
        ring: invalid.then(|| {
            let mut ring = style::invalid_outset_ring(&t, radius);
            ring.color = alpha(ring.color, opacity);
            ring
        }),
        text_color: Some(fade(p.primary_foreground)),
    }
}
