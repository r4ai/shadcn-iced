//! Combine labels, controls, and help text to compose accessible form
//! fields.
//!
//! Port of <https://ui.shadcn.com/docs/components/field>.
//!
//! ```no_run
//! # use iced_ui::field::field;
//! # use iced_ui::input::input;
//! # #[derive(Clone)] enum Message { Name(String) }
//! # fn view(name: &str) -> iced::Element<'_, Message> {
//! field("Name", input("Evil Rabbit", name).on_input(Message::Name))
//!     .description("Choose a unique name for your account.")
//!     .into()
//! # }
//! ```
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Theme};

/// The layout of a [`Field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Label above the control.
    #[default]
    Vertical,
    /// Control on the left of the label, e.g. for checkboxes and switches.
    Horizontal,
    /// Label and description on the left, control on the right.
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
    }
}

impl<'a, Message: 'a> Field<'a, Message> {
    /// Sets the help text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets an error message, which also colors the label.
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
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

impl<'a, Message: 'a> From<Field<'a, Message>> for Element<'a, Message> {
    fn from(field: Field<'a, Message>) -> Self {
        let invalid = field.error.is_some();
        let disabled = field.disabled;

        let label = field.label.map(|label| {
            text(label)
                .size(style::text::SM)
                .font(font::MEDIUM)
                .line_height(1.375)
                .style(move |theme: &Theme| {
                    let p = tokens(theme).palette;
                    text::Style {
                        color: Some(iced_ui_core::theme::alpha(
                            if invalid { p.destructive } else { p.foreground },
                            if disabled { style::DISABLED_OPACITY } else { 1.0 },
                        )),
                    }
                })
        });

        let description = field
            .description
            .map(|description| text(description).size(style::text::SM).line_height(1.5).style(muted));

        let error = field.error.map(|error| {
            text(error).size(style::text::SM).style(|theme: &Theme| text::Style {
                color: Some(tokens(theme).palette.destructive),
            })
        });

        match field.orientation {
            Orientation::Vertical => Column::new()
                .push(label)
                .push(field.control)
                .push(description)
                .push(error)
                .spacing(12)
                .width(Length::Fill)
                .into(),
            Orientation::Horizontal => Row::new()
                .push(field.control)
                .push(
                    Column::new()
                        .push(label)
                        .push(description)
                        .push(error)
                        .spacing(6)
                        .width(Length::Fill),
                )
                .spacing(12)
                .align_y(Alignment::Start)
                .into(),
            Orientation::Responsive => Row::new()
                .push(
                    Column::new()
                        .push(label)
                        .push(description)
                        .push(error)
                        .spacing(6)
                        .width(Length::Fill),
                )
                .push(field.control)
                .spacing(12)
                .align_y(Alignment::Center)
                .into(),
        }
    }
}

/// A group of fields with a legend (`FieldSet` + `FieldLegend`).
pub struct FieldSet<'a, Message> {
    legend: String,
    description: Option<String>,
    fields: Vec<Element<'a, Message>>,
    spacing: f32,
}

/// Creates a [`FieldSet`] with a legend.
pub fn field_set<'a, Message: 'a>(legend: impl Into<String>) -> FieldSet<'a, Message> {
    FieldSet {
        legend: legend.into(),
        description: None,
        fields: Vec::new(),
        spacing: 28.0,
    }
}

impl<'a, Message: 'a> FieldSet<'a, Message> {
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

    /// Sets the space between fields. Defaults to `gap-7`.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a, Message: 'a> From<FieldSet<'a, Message>> for Element<'a, Message> {
    fn from(set: FieldSet<'a, Message>) -> Self {
        let header = Column::new()
            .push(text(set.legend).size(style::text::BASE).font(font::MEDIUM))
            .push(
                set.description
                    .map(|description| text(description).size(style::text::SM).line_height(1.5).style(muted)),
            )
            .spacing(6);

        Column::new()
            .push(header)
            .push(Column::with_children(set.fields).spacing(set.spacing))
            .spacing(24)
            .width(Length::Fill)
            .into()
    }
}

/// A separator between fields, with optional centered text
/// (`FieldSeparator`).
pub fn field_separator<'a, Message: 'a>(label: Option<&str>) -> Element<'a, Message> {
    match label {
        None => container(super::separator::separator()).padding([10, 0]).into(),
        Some(label) => Row::new()
            .push(super::separator::separator())
            .push(
                container(text(label.to_owned()).size(style::text::SM).style(muted))
                    .padding([0, 8])
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(tokens(theme).palette.background)),
                        ..container::Style::default()
                    }),
            )
            .push(super::separator::separator())
            .align_y(Alignment::Center)
            .height(20)
            .into(),
    }
}
