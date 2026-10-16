//! A responsive table component.
//!
//! Port of <https://ui.shadcn.com/docs/components/table>.
//!
//! ```no_run
//! # use iced_ui::table::table;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! table(["Invoice", "Status", "Amount"])
//!     .row(["INV001", "Paid", "$250.00"])
//!     .row(["INV002", "Pending", "$150.00"])
//!     .caption("A list of your recent invoices.")
//!     .into()
//! # }
//! ```
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Theme, alignment, mouse};

/// A table. See the [module documentation](self).
pub struct Table<'a, Message> {
    headers: Vec<Element<'a, Message>>,
    rows: Vec<(Vec<Element<'a, Message>>, bool)>,
    footer: Option<Vec<Element<'a, Message>>>,
    caption: Option<String>,
    widths: Vec<Length>,
    align: Vec<alignment::Horizontal>,
}

/// Creates a [`Table`] with the given column headers.
pub fn table<'a, Message: Clone + 'a, T: Into<Cell<'a, Message>>>(
    headers: impl IntoIterator<Item = T>,
) -> Table<'a, Message> {
    let headers: Vec<Element<'a, Message>> = headers
        .into_iter()
        .map(|header| match header.into() {
            Cell::Text(label) => text(label)
                .size(style::text::SM)
                .font(font::medium())
                .wrapping(text::Wrapping::None)
                .into(),
            Cell::Element(element) => element,
        })
        .collect();

    let columns = headers.len();

    Table {
        headers,
        rows: Vec::new(),
        footer: None,
        caption: None,
        widths: vec![Length::Fill; columns],
        align: vec![alignment::Horizontal::Left; columns],
    }
}

/// The content of a cell: either text or any element.
pub enum Cell<'a, Message> {
    /// A text cell.
    Text(String),
    /// A custom cell.
    Element(Element<'a, Message>),
}

impl<Message> From<&str> for Cell<'_, Message> {
    fn from(value: &str) -> Self {
        Cell::Text(value.to_owned())
    }
}

impl<Message> From<String> for Cell<'_, Message> {
    fn from(value: String) -> Self {
        Cell::Text(value)
    }
}

impl<'a, Message> From<Element<'a, Message>> for Cell<'a, Message> {
    fn from(value: Element<'a, Message>) -> Self {
        Cell::Element(value)
    }
}

fn cell<'a, Message: 'a>(cell: Cell<'a, Message>) -> Element<'a, Message> {
    match cell {
        // `p-2 align-middle whitespace-nowrap`
        Cell::Text(value) => text(value)
            .size(style::text::SM)
            .line_height(iced::Pixels(20.0))
            .wrapping(text::Wrapping::None)
            .into(),
        Cell::Element(element) => element,
    }
}

impl<'a, Message: Clone + 'a> Table<'a, Message> {
    /// Appends a row.
    pub fn row<T: Into<Cell<'a, Message>>>(self, cells: impl IntoIterator<Item = T>) -> Self {
        self.push_row(cells, false)
    }

    /// Appends a selected row (`data-state=selected`).
    pub fn selected_row<T: Into<Cell<'a, Message>>>(
        self,
        cells: impl IntoIterator<Item = T>,
    ) -> Self {
        self.push_row(cells, true)
    }

    fn push_row<T: Into<Cell<'a, Message>>>(
        mut self,
        cells: impl IntoIterator<Item = T>,
        selected: bool,
    ) -> Self {
        let cells = cells.into_iter().map(|c| cell(c.into())).collect();
        self.rows.push((cells, selected));
        self
    }

    /// Sets the footer row.
    pub fn footer<T: Into<Cell<'a, Message>>>(
        mut self,
        cells: impl IntoIterator<Item = T>,
    ) -> Self {
        self.footer = Some(
            cells
                .into_iter()
                .map(|c| match c.into() {
                    Cell::Text(value) => text(value)
                        .size(style::text::SM)
                        .font(font::medium())
                        .line_height(iced::Pixels(20.0))
                        .into(),
                    Cell::Element(element) => element,
                })
                .collect(),
        );
        self
    }

    /// Sets the caption displayed below the table.
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the width of the columns.
    pub fn widths(mut self, widths: impl IntoIterator<Item = Length>) -> Self {
        self.widths = widths.into_iter().collect();
        self
    }

    /// Sets the horizontal alignment of a column.
    pub fn align(mut self, column: usize, align: alignment::Horizontal) -> Self {
        if let Some(slot) = self.align.get_mut(column) {
            *slot = align;
        }
        self
    }
}

fn layout_row<'a, Message: 'a>(
    cells: Vec<Element<'a, Message>>,
    widths: &[Length],
    align: &[alignment::Horizontal],
    height: Option<f32>,
    padding: [f32; 2],
) -> Row<'a, Message> {
    Row::with_children(cells.into_iter().enumerate().map(|(index, content)| {
        let mut cell = container(content)
            .padding(padding)
            .width(widths.get(index).copied().unwrap_or(Length::Fill))
            .align_x(
                align
                    .get(index)
                    .copied()
                    .unwrap_or(alignment::Horizontal::Left),
            )
            .align_y(alignment::Vertical::Center);

        if let Some(height) = height {
            cell = cell.height(height);
        }

        cell.into()
    }))
    .align_y(Alignment::Center)
}

fn row_style(theme: &Theme, status: Status, selected: bool) -> Style {
    let p = tokens(theme).palette;
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: if selected {
            Some(Background::Color(p.muted))
        } else {
            hovered.then(|| Background::Color(alpha(p.muted, 0.5)))
        },
        text_color: p.foreground,
        ..Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Table<'a, Message>> for Element<'a, Message> {
    fn from(table: Table<'a, Message>) -> Self {
        let mut column = Column::new();

        column = column.push(layout_row(
            table.headers,
            &table.widths,
            &table.align,
            Some(40.0),
            [0.0, 8.0],
        ));
        column = column.push(super::separator::separator());

        let count = table.rows.len();

        for (index, (cells, selected)) in table.rows.into_iter().enumerate() {
            let row = layout_row(cells, &table.widths, &table.align, None, [8.0, 8.0]);

            column = column.push(
                Pressable::new(row)
                    .width(Length::Fill)
                    .interaction(mouse::Interaction::Idle)
                    .style(move |theme, status| row_style(theme, status, selected)),
            );

            if index + 1 < count || table.footer.is_some() {
                column = column.push(super::separator::separator());
            }
        }

        if let Some(footer) = table.footer {
            column = column.push(
                container(layout_row(
                    footer,
                    &table.widths,
                    &table.align,
                    None,
                    [8.0, 8.0],
                ))
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(alpha(tokens(theme).palette.muted, 0.5))),
                    ..container::Style::default()
                }),
            );
        }

        if let Some(caption) = table.caption {
            column =
                column.push(
                    container(text(caption).size(style::text::SM).style(|theme: &Theme| {
                        text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }
                    }))
                    .padding(iced::Padding::ZERO.top(16))
                    .center_x(Length::Fill),
                );
        }

        column.width(Length::Fill).into()
    }
}
