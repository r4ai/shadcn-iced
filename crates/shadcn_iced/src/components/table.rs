//! A responsive table component.
//!
//! Port of <https://ui.shadcn.com/docs/components/table>.
//!
//! ```no_run
//! # use shadcn_iced::table::table;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! table(["Invoice", "Status", "Amount"])
//!     .row(["INV001", "Paid", "$250.00"])
//!     .row(["INV002", "Pending", "$150.00"])
//!     .caption("A list of your recent invoices.")
//!     .into()
//! # }
//! ```
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::widget::{Column, container, space, text};
use iced::{
    Background, Color, Event, Length, Point, Rectangle, Size, Theme, Vector, alignment, mouse,
};

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
///
/// Like an HTML table (`w-full`, `table-layout: auto`), the columns are as
/// wide as their content by default and share the remaining width in
/// proportion to it. See [`Table::widths`].
pub fn table<'a, Message: Clone + 'a, T: Into<Cell<'a, Message>>>(
    headers: impl IntoIterator<Item = T>,
) -> Table<'a, Message> {
    let headers: Vec<Element<'a, Message>> = headers
        .into_iter()
        .map(|header| match header.into() {
            // `font-medium whitespace-nowrap`
            Cell::Text(label) => text(label)
                .size(style::text::SM)
                .line_height(style::text::leading(style::text::SM))
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
        widths: vec![Length::Shrink; columns],
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
        // `whitespace-nowrap`
        Cell::Text(value) => text(value)
            .size(style::text::SM)
            .line_height(style::text::leading(style::text::SM))
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
                    // `font-medium whitespace-nowrap`
                    Cell::Text(value) => text(value)
                        .size(style::text::SM)
                        .font(font::medium())
                        .line_height(style::text::leading(style::text::SM))
                        .wrapping(text::Wrapping::None)
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

    /// Sets the width of the columns:
    ///
    /// - [`Length::Shrink`] (the default) sizes a column to its content, like
    ///   `table-layout: auto`; when the table is wider than its content,
    ///   these columns share the extra width in proportion to their content.
    /// - [`Length::Fixed`] sets the width of a column, padding included
    ///   (e.g. `w-25` is `Length::Fixed(100.0)`).
    /// - [`Length::Fill`] and [`Length::FillPortion`] share the width left by
    ///   the other columns.
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

/// The kind of a row, which sets its height and its decorations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// `TableHead`: `h-10 px-2`.
    Header,
    /// `TableCell`: `p-2`, `data-[state=selected]:bg-muted`.
    Body { selected: bool },
    /// `TableFooter`: `bg-muted/50`.
    Footer,
}

/// The horizontal padding of the cells (`px-2`).
const PADDING_X: f32 = 8.0;

/// The vertical padding of the body and footer cells (`p-2`).
const PADDING_Y: f32 = 8.0;

/// The height of the header row (`h-10`).
const HEADER_HEIGHT: f32 = 40.0;

/// The width of the borders between rows (`border-b`).
const BORDER: f32 = 1.0;

/// Lays out the cells of a table in a grid, draws the backgrounds and the
/// borders of its rows, and highlights the hovered row (`hover:bg-muted/50`).
struct Grid<'a, Message> {
    /// The cells, row by row; every row has `columns` cells.
    cells: Vec<Element<'a, Message>>,
    rows: Vec<Kind>,
    columns: usize,
    widths: Vec<Length>,
    align: Vec<alignment::Horizontal>,
}

#[derive(Debug, Default)]
struct State {
    /// The vertical position and the height of every row.
    rows: Vec<(f32, f32)>,
    hovered: Option<usize>,
}

impl<Message> Grid<'_, Message> {
    /// Whether a border follows the given row: the header has one
    /// (`[&_tr]:border-b`), the body rows too except the last one
    /// (`[&_tr:last-child]:border-0`), which the footer replaces with its
    /// own (`border-t`).
    fn has_border_after(&self, row: usize) -> bool {
        match self.rows[row] {
            Kind::Header => true,
            Kind::Body { .. } => self.rows.get(row + 1).is_some(),
            Kind::Footer => false,
        }
    }

    /// The widths of the columns, padding included, given the width of
    /// their content and the width of the table.
    fn column_widths(&self, content: &[f32], width: f32) -> Vec<f32> {
        let length = |column: usize| self.widths.get(column).copied().unwrap_or(Length::Shrink);
        let natural: Vec<f32> = content.iter().map(|w| w + 2.0 * PADDING_X).collect();
        let mut widths = vec![0.0; content.len()];
        let mut portions = 0.0;
        let mut auto = 0.0;

        for (column, width) in widths.iter_mut().enumerate() {
            match length(column) {
                Length::Fixed(fixed) => *width = fixed,
                Length::Shrink => {
                    *width = natural[column];
                    auto += natural[column];
                }
                fill => portions += f32::from(fill.fill_factor()),
            }
        }

        let remaining = if width.is_finite() {
            (width - widths.iter().sum::<f32>()).max(0.0)
        } else {
            0.0
        };

        for (column, width) in widths.iter_mut().enumerate() {
            match length(column) {
                Length::Fixed(_) => {}
                Length::Shrink => {
                    if portions == 0.0 && auto > 0.0 {
                        *width += remaining * natural[column] / auto;
                    }
                }
                fill => *width = remaining * f32::from(fill.fill_factor()) / portions,
            }
        }

        widths
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Grid<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let columns = self.columns.max(1);

        // The width of the content of every column (`whitespace-nowrap`).
        let unbounded = layout::Limits::new(Size::ZERO, Size::INFINITE);
        let mut content = vec![0.0_f32; columns];

        for (index, (cell, tree)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let width = cell
                .as_widget_mut()
                .layout(tree, renderer, &unbounded)
                .size()
                .width;

            if width.is_finite() {
                content[index % columns] = content[index % columns].max(width);
            }
        }

        let available = limits.max().width;
        let widths = self.column_widths(&content, available);
        let total: f32 = widths.iter().sum();
        let width = if available.is_finite() {
            available.max(total)
        } else {
            total
        };

        let borders: Vec<bool> = (0..self.rows.len())
            .map(|row| self.has_border_after(row))
            .collect();

        let mut nodes = Vec::with_capacity(self.cells.len());
        let mut rows = Vec::with_capacity(self.rows.len());
        let mut cells = self.cells.iter_mut().zip(&mut tree.children);
        let mut y = 0.0;

        for (row, kind) in self.rows.iter().enumerate() {
            let row_nodes: Vec<layout::Node> = widths
                .iter()
                .zip(cells.by_ref())
                .map(|(width, (cell, tree))| {
                    let inner = (width - 2.0 * PADDING_X).max(0.0);

                    cell.as_widget_mut().layout(
                        tree,
                        renderer,
                        &layout::Limits::new(Size::ZERO, Size::new(inner, f32::INFINITY)),
                    )
                })
                .collect();

            let content = row_nodes
                .iter()
                .map(|node| node.size().height)
                .fold(0.0, f32::max);

            let height = match kind {
                Kind::Header => content.max(HEADER_HEIGHT),
                Kind::Body { .. } | Kind::Footer => content + 2.0 * PADDING_Y,
            };

            let mut x = 0.0;

            for (column, node) in row_nodes.into_iter().enumerate() {
                let inner = (widths[column] - 2.0 * PADDING_X).max(0.0);
                let size = node.size();
                let offset = match self
                    .align
                    .get(column)
                    .copied()
                    .unwrap_or(alignment::Horizontal::Left)
                {
                    alignment::Horizontal::Left => 0.0,
                    alignment::Horizontal::Center => (inner - size.width) / 2.0,
                    alignment::Horizontal::Right => inner - size.width,
                };

                // `align-middle`
                nodes.push(node.move_to(Point::new(
                    x + PADDING_X + offset,
                    y + (height - size.height) / 2.0,
                )));

                x += widths[column];
            }

            rows.push((y, height));
            y += height;

            if borders[row] {
                y += BORDER;
            }
        }

        tree.state.downcast_mut::<State>().rows = rows;

        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((cell, tree), layout) in self
                .cells
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                cell.as_widget_mut()
                    .operate(tree, layout, renderer, operation);
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((cell, tree), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        let hovered = cursor.position_over(bounds).and_then(|position| {
            state.rows.iter().position(|(y, height)| {
                let top = bounds.y + y;
                position.y >= top && position.y < top + height
            })
        });

        if state.hovered != hovered {
            state.hovered = hovered;
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let p = tokens(theme).palette;
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let fill = |renderer: &mut iced::Renderer, y: f32, height: f32, color: Color| {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y + y,
                        width: bounds.width,
                        height,
                    },
                    ..Quad::default()
                },
                Background::Color(color),
            );
        };

        for (row, (kind, &(y, height))) in self.rows.iter().zip(&state.rows).enumerate() {
            match kind {
                // `data-[state=selected]:bg-muted`
                Kind::Body { selected: true } => fill(renderer, y, height, p.muted),
                // `bg-muted/50`
                Kind::Footer => fill(renderer, y, height, alpha(p.muted, 0.5)),
                Kind::Header | Kind::Body { selected: false } => {}
            }

            // `hover:bg-muted/50`
            if state.hovered == Some(row) && *kind != (Kind::Body { selected: true }) {
                fill(renderer, y, height, alpha(p.muted, 0.5));
            }

            if self.has_border_after(row) {
                fill(renderer, y + height, BORDER, p.border);
            }
        }

        let style = renderer::Style {
            text_color: p.foreground,
        };

        for ((cell, tree), layout) in self.cells.iter().zip(&tree.children).zip(layout.children()) {
            cell.as_widget()
                .draw(tree, renderer, theme, &style, layout, cursor, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((cell, tree), layout)| {
                cell.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        overlay::from_children(
            &mut self.cells,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<Table<'a, Message>> for Element<'a, Message> {
    fn from(table: Table<'a, Message>) -> Self {
        let columns = table.headers.len();
        let mut cells: Vec<Element<'a, Message>> = Vec::new();
        let mut rows = Vec::new();

        let mut push = |row: Vec<Element<'a, Message>>, kind: Kind| {
            let count = row.len();
            cells.extend(row.into_iter().take(columns));
            cells.extend((count..columns).map(|_| space().into()));
            rows.push(kind);
        };

        push(table.headers, Kind::Header);

        for (row, selected) in table.rows {
            push(row, Kind::Body { selected });
        }

        if let Some(footer) = table.footer {
            push(footer, Kind::Footer);
        }

        let grid = Grid {
            cells,
            rows,
            columns,
            widths: table.widths,
            align: table.align,
        };

        // `caption-bottom`, `mt-4 text-sm text-muted-foreground`
        let caption = table.caption.map(|caption| {
            container(
                text(caption)
                    .size(style::text::SM)
                    .line_height(style::text::leading(style::text::SM))
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            )
            .padding(iced::Padding::ZERO.top(16))
            .center_x(Length::Fill)
        });

        Column::new()
            .push(Element::new(grid))
            .push(caption)
            .width(Length::Fill)
            .into()
    }
}
