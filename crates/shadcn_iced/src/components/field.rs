//! Combine labels, controls, and help text to compose accessible form
//! fields.
//!
//! Port of <https://ui.shadcn.com/docs/components/field>.
//!
//! ```no_run
//! # use shadcn_iced::field::field;
//! # use shadcn_iced::input::input;
//! # #[derive(Clone)] enum Message { Name(String) }
//! # fn view(name: &str) -> iced::Element<'_, Message> {
//! field("Name", input("Evil Rabbit", name).on_input(Message::Name))
//!     .description("Choose a unique name for your account.")
//!     .into()
//! # }
//! ```
use shadcn_iced_core::primitive::{Labelled, Pressable};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Font, Length, Padding, Theme};

/// The layout of a [`Field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Label above the control (`flex-col`).
    #[default]
    Vertical,
    /// Control on the left of the label, e.g. for checkboxes and switches
    /// (`flex-row items-center`).
    Horizontal,
    /// Label and description on the left, control on the right, like the
    /// `responsive` orientation in a wide field group.
    Responsive,
}

/// A form field. See the [module documentation](self).
pub struct Field<'a, Message> {
    label: Option<String>,
    control: Element<'a, Message>,
    description: Option<String>,
    error: Option<String>,
    orientation: Orientation,
    disabled: bool,
    invalid: bool,
    font: Font,
}

/// Creates a [`Field`] with a label and a control.
pub fn field<'a, Message: 'a>(
    label: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
) -> Field<'a, Message> {
    Field {
        label: Some(label.into()),
        control: control.into(),
        description: None,
        error: None,
        orientation: Orientation::Vertical,
        disabled: false,
        invalid: false,
        font: font::medium(),
    }
}

/// Creates a [`Field`] without a label.
pub fn unlabeled<'a, Message: 'a>(control: impl Into<Element<'a, Message>>) -> Field<'a, Message> {
    Field {
        label: None,
        ..field(String::new(), control)
    }
}

impl<'a, Message: 'a> Field<'a, Message> {
    /// Sets the help text (`FieldDescription`).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets an error message (`FieldError`), which also colors the label.
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Sets an error message, if any.
    pub fn error_maybe(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Dims the label (`data-disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Colors the label (`data-invalid:text-destructive`), without an error
    /// message.
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

/// `text-muted-foreground`
fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// `FieldDescription`: `text-sm leading-normal text-muted-foreground`.
fn description<'a, Message: 'a>(content: String) -> Element<'a, Message> {
    text(content)
        .size(style::text::SM)
        .line_height(1.5)
        .style(muted)
        .into()
}

impl<'a, Message: 'a> From<Field<'a, Message>> for Element<'a, Message> {
    fn from(field: Field<'a, Message>) -> Self {
        let invalid = field.error.is_some() || field.invalid;
        let disabled = field.disabled;
        let has_description = field.description.is_some() || field.error.is_some();

        // `FieldLabel`: `text-sm font-medium leading-snug`
        let label = field.label.map(|label| {
            text(label)
                .size(style::text::SM)
                .font(field.font)
                .line_height(1.375)
                .style(move |theme: &Theme| {
                    let p = tokens(theme).palette;
                    text::Style {
                        color: Some(alpha(
                            if invalid { p.destructive } else { p.foreground },
                            if disabled {
                                style::DISABLED_OPACITY
                            } else {
                                1.0
                            },
                        )),
                    }
                })
        });

        let description = field.description.map(description);

        // `FieldError`: `text-destructive text-sm`
        let error = field.error.map(|error| {
            text(error)
                .size(style::text::SM)
                .line_height(1.5)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.destructive),
                })
        });

        // `FieldContent`: `flex-1 gap-0.5`
        let content = |label, description, error| {
            Column::new()
                .push(label)
                .push(description)
                .push(error)
                .spacing(2)
                .width(Length::Fill)
        };

        // `has-[>[data-slot=field-content]]:items-start` and
        // `[&>[role=checkbox],[role=radio]]:mt-px`.
        let control = |control| {
            container(control).padding(Padding {
                top: if has_description { 1.0 } else { 0.0 },
                ..Padding::ZERO
            })
        };
        let align = if has_description {
            Alignment::Start
        } else {
            Alignment::Center
        };

        // `gap-2`
        let field: Element<'a, Message> = match field.orientation {
            Orientation::Vertical => Column::new()
                .push(label)
                .push(field.control)
                .push(description)
                .push(error)
                .spacing(8)
                .width(Length::Fill)
                .into(),
            Orientation::Horizontal => Row::new()
                .push(control(field.control))
                .push(content(label, description, error))
                .spacing(8)
                .align_y(align)
                .into(),
            Orientation::Responsive => Row::new()
                .push(content(label, description, error))
                .push(control(field.control))
                .spacing(8)
                .align_y(align)
                .into(),
        };

        // The label focuses (or toggles) the control (`htmlFor`).
        Labelled::new(field).into()
    }
}

/// The look of a [`FieldSet`] legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegendVariant {
    /// `text-base`, followed closely by the description.
    #[default]
    Legend,
    /// `text-sm`, like a label.
    Label,
}

/// A group of fields with a legend (`FieldSet` + `FieldLegend` +
/// `FieldGroup`).
pub struct FieldSet<'a, Message> {
    legend: Option<String>,
    variant: LegendVariant,
    description: Option<String>,
    fields: Vec<Element<'a, Message>>,
    spacing: f32,
}

/// Creates a [`FieldSet`] with a legend.
pub fn field_set<'a, Message: 'a>(legend: impl Into<String>) -> FieldSet<'a, Message> {
    FieldSet {
        legend: Some(legend.into()),
        variant: LegendVariant::Legend,
        description: None,
        fields: Vec::new(),
        spacing: 20.0,
    }
}

impl<'a, Message: 'a> FieldSet<'a, Message> {
    /// Removes the legend.
    pub fn no_legend(mut self) -> Self {
        self.legend = None;
        self
    }

    /// Sets the look of the legend.
    pub fn legend_variant(mut self, variant: LegendVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the description shown under the legend.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends a field (or any element).
    pub fn push(mut self, field: impl Into<Element<'a, Message>>) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Sets the space between fields. Defaults to `gap-5`; checkbox and
    /// radio groups use `gap-3`.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a, Message: 'a> From<FieldSet<'a, Message>> for Element<'a, Message> {
    fn from(set: FieldSet<'a, Message>) -> Self {
        let variant = set.variant;

        // `mb-1.5 font-medium`; the description follows a `legend` with
        // `-mt-1.5`.
        let header = Column::new()
            .push(set.legend.map(|legend| {
                text(legend)
                    .size(match variant {
                        LegendVariant::Legend => style::text::BASE,
                        LegendVariant::Label => style::text::SM,
                    })
                    .font(font::medium())
            }))
            .push(set.description.map(description))
            .spacing(match variant {
                LegendVariant::Legend => 0,
                LegendVariant::Label => 6,
            });

        // `gap-4` (`gap-3` around checkbox and radio groups).
        Column::new()
            .push(header)
            .push(field_group(set.fields).spacing(set.spacing))
            .spacing(if set.spacing < 20.0 { 12 } else { 16 })
            .width(Length::Fill)
            .into()
    }
}

/// A stack of fields (`FieldGroup`: `flex-col gap-5`).
pub fn field_group<'a, Message: 'a>(
    fields: impl IntoIterator<Item = Element<'a, Message>>,
) -> Column<'a, Message> {
    Column::with_children(fields)
        .spacing(20)
        .width(Length::Fill)
}

/// A separator between fields, with optional centered text
/// (`FieldSeparator`: `h-5 -my-2`).
pub fn field_separator<'a, Message: 'a>(label: Option<&str>) -> Element<'a, Message> {
    // The negative margins eat into the gaps around the separator; the
    // 20px tall content overflows its 4px slot.
    let content: Element<'a, Message> = match label {
        None => super::separator::separator().into(),
        Some(label) => Row::new()
            .push(super::separator::separator())
            .push(
                // `text-muted-foreground px-2 bg-background`
                container(
                    text(label.to_owned())
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .style(muted),
                )
                .padding([0, 8])
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(tokens(theme).palette.background)),
                    ..container::Style::default()
                }),
            )
            .push(super::separator::separator())
            .align_y(Alignment::Center)
            .into(),
    };

    container(content)
        .center_y(Length::Fixed(4.0))
        .width(Length::Fill)
        .into()
}

/// A selectable card around a field (`FieldLabel` wrapping a horizontal
/// `Field`), e.g. a [`field`] with [`Orientation::Responsive`] or a
/// checkbox with a label and a description.
///
/// The whole card is pressable: pass the message of its control.
pub fn choice_card<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    checked: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    // `border` + `*:data-[slot=field]:p-2.5`
    Pressable::new(container(content).width(Length::Fill))
        .on_press_maybe(on_press)
        .width(Length::Fill)
        .padding(11)
        .style(move |theme, status| choice_card_style(theme, status, checked))
        .into()
}

/// The style of a [`choice_card`]: `rounded-lg border`,
/// `hover:bg-muted/50`, `has-data-checked:bg-primary/5
/// has-data-checked:border-primary/30`.
pub fn choice_card_style(theme: &Theme, status: Status, checked: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    let background = if checked {
        Some(alpha(p.primary, if dark { 0.1 } else { 0.05 }))
    } else if hovered && status != Status::Disabled {
        Some(alpha(p.muted, 0.5))
    } else {
        None
    };

    Style {
        background: background.map(Background::Color),
        text_color: p.foreground,
        border: style::border(
            if checked {
                alpha(p.primary, if dark { 0.2 } else { 0.3 })
            } else {
                p.border
            },
            t.radius_lg(),
        ),
        ..Style::default()
    }
}
