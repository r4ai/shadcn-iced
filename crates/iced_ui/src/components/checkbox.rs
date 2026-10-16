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
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Theme, mouse};

/// A checkbox. See the [module documentation](self).
pub struct Checkbox<'a, Message> {
    checked: bool,
    label: Option<String>,
    description: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    disabled: bool,
    invalid: bool,
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
    }
}

impl<'a, Message: Clone + 'a> Checkbox<'a, Message> {
    /// Sets the label, displayed on the right.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets a description, displayed below the label.
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

    /// Marks the checkbox as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Checkbox<'a, Message>> for Element<'a, Message> {
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        let checked = checkbox.checked;
        let disabled = checkbox.disabled || checkbox.on_toggle.is_none();
        let invalid = checkbox.invalid;

        let indicator = container(checked.then(|| lucide::CHECK.sized(14)))
            .center(16)
            .style(move |theme| indicator(theme, checked, disabled, invalid));

        let has_description = checkbox.description.is_some();

        let labels = (checkbox.label.is_some() || checkbox.description.is_some()).then(|| {
            Column::new()
                .push(checkbox.label.map(|label| {
                    text(label)
                        .size(style::text::SM)
                        .font(font::MEDIUM)
                        .line_height(iced::Pixels(16.0))
                        .style(move |theme: &Theme| text::Style {
                            color: Some(alpha(
                                tokens(theme).palette.foreground,
                                if disabled { style::DISABLED_OPACITY } else { 1.0 },
                            )),
                        })
                }))
                .push(checkbox.description.map(|description| {
                    text(description).size(style::text::SM).style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
                }))
                .spacing(8)
        });

        let on_press = checkbox.on_toggle.map(|f| f(!checked));

        Pressable::new(
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

/// The style of the checkbox square.
pub fn indicator(theme: &Theme, checked: bool, disabled: bool, invalid: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled { style::DISABLED_OPACITY } else { 1.0 };

    let (background, border) = if checked {
        (p.primary, p.primary)
    } else {
        (
            if t.is_dark() { alpha(p.input, 0.3) } else { Color::TRANSPARENT },
            if invalid { p.destructive } else { p.input },
        )
    };

    container::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        text_color: Some(alpha(p.primary_foreground, opacity)),
        border: style::border(alpha(border, opacity), 4.0),
        ..container::Style::default()
    }
}
