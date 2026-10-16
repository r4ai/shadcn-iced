//! A control that allows the user to toggle between checked and not checked.
//!
//! Port of <https://ui.shadcn.com/docs/components/checkbox>.
//!
//! ```no_run
//! # use iced_ui::checkbox::checkbox;
//! # #[derive(Clone)] enum Message { Terms(bool) }
//! # fn view<'a>(accepted: bool) -> iced::Element<'a, Message> {
//! checkbox(accepted).label("Accept terms and conditions").on_toggle(Message::Terms).into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::primitive::focus::{self, FocusWithin};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Font, Padding, Theme, mouse};

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
        let indicator = container(indicator(checked, disabled, invalid)).padding(Padding {
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

        let on_press = checkbox.on_toggle.map(|f| f(!checked));

        // `Field orientation="horizontal"`: `gap-2 items-center`, or
        // `items-start` with a description.
        Pressable::new(Row::new().push(indicator).push(labels).spacing(8).align_y(
            if has_description {
                Alignment::Start
            } else {
                Alignment::Center
            },
        ))
        .on_press_maybe(if disabled { None } else { on_press })
        .disabled(disabled)
        .interaction(if disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Pointer
        })
        .style(|_, _| iced::widget::button::Style::default())
        .into()
    }
}

/// The checkbox square alone (`size-4 rounded-[4px] border`), with its
/// `ring-3` when invalid.
pub fn indicator<'a, Message: 'a>(
    checked: bool,
    disabled: bool,
    invalid: bool,
) -> Element<'a, Message> {
    // `[&>svg]:size-3.5`
    let check = container(checked.then(|| lucide::CHECK.sized(14))).center(16);

    FocusWithin::new(check, move |theme, _status| {
        self::style(theme, checked, disabled, invalid)
    })
    .into()
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

    focus::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        border: style::border(alpha(border, opacity), radius),
        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`
        ring: invalid.then(|| {
            let mut ring = style::invalid_outset_ring(&t, radius);
            ring.color = alpha(ring.color, opacity);
            ring
        }),
        text_color: Some(alpha(p.primary_foreground, opacity)),
    }
}
