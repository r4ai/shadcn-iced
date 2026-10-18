//! The focused cell of a grid, like the days of a calendar.
//!
//! The cells of a grid (the days of react-day-picker) are not focusable
//! widgets of their own: [`FocusGrid`] knows where they are and which one
//! is focused. A pressed cell takes the focus, a press outside of the grid
//! blurs it, and when the focused cell is no longer displayed (the months
//! changed) the focus moves to a fallback cell, e.g. the selected day. The
//! focused cell shows a [`Ring`] outside of its bounds.
//!
//! Cells are identified by a key of yours, which survives the views.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::focus::Ring;
//! # use shadcn_iced_core::primitive::focus_grid::{Cell, focus_grid};
//! # use iced::{Color, Point, Rectangle, Size};
//! # fn view<'a, Message: 'a>(days: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! let cells = (0..7).map(|day| Cell {
//!     key: day,
//!     bounds: Rectangle::new(Point::new(day as f32 * 32.0, 0.0), Size::new(32.0, 32.0)),
//!     pressable: true,
//! });
//!
//! focus_grid(days, cells)
//!     .fallback(Some(0))
//!     .ring(|_theme| Ring { color: Color::BLACK, width: 3.0, radius: 6.0 })
//!     .into()
//! # }
//! ```
use super::focus::Ring;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, Vector, mouse};

/// A cell of a [`FocusGrid`].
#[derive(Debug, Clone, PartialEq)]
pub struct Cell<K> {
    /// Identifies the cell across views.
    pub key: K,
    /// Where the cell is, relative to the grid.
    pub bounds: Rectangle,
    /// Whether a press takes the focus (disabled cells do not).
    pub pressable: bool,
}

/// A grid with a focused cell. See the [module documentation](self).
pub struct FocusGrid<'a, K, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    cells: Vec<Cell<K>>,
    fallback: Option<K>,
    ring: Option<Box<dyn Fn(&Theme) -> Ring + 'a>>,
}

/// Creates a [`FocusGrid`] over `content`, whose cells are at `cells`.
pub fn focus_grid<'a, K, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    cells: impl IntoIterator<Item = Cell<K>>,
) -> FocusGrid<'a, K, Message, Theme, Renderer> {
    FocusGrid {
        content: content.into(),
        cells: cells.into_iter().collect(),
        fallback: None,
        ring: None,
    }
}

impl<'a, K, Message, Theme, Renderer> FocusGrid<'a, K, Message, Theme, Renderer>
where
    K: PartialEq,
{
    /// Sets the cell that takes the focus when the focused cell is not
    /// displayed anymore (react-day-picker's `calculateFocusTarget`).
    pub fn fallback(mut self, key: Option<K>) -> Self {
        self.fallback = key;
        self
    }

    /// Sets the ring drawn around the focused cell.
    pub fn ring(mut self, ring: impl Fn(&Theme) -> Ring + 'a) -> Self {
        self.ring = Some(Box::new(ring));
        self
    }

    /// The displayed cell with the given key.
    fn cell(&self, key: &K) -> Option<&Cell<K>> {
        self.cells.iter().find(|cell| cell.key == *key)
    }

    /// The focused cell, moved to the fallback when it is not displayed.
    fn focused(&self, state: &State<K>) -> Option<&Cell<K>> {
        let focused = state.focused.as_ref()?;

        self.cell(focused)
            .or_else(|| self.fallback.as_ref().and_then(|key| self.cell(key)))
    }
}

struct State<K> {
    focused: Option<K>,
}

impl<'a, K, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusGrid<'a, K, Message, Theme, Renderer>
where
    K: Clone + PartialEq + 'static,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<K>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<K> { focused: None })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            let state = tree.state.downcast_mut::<State<K>>();

            match cursor.position_in(layout.bounds()) {
                // Cells that cannot be pressed do not take the focus, and
                // the rest of the grid keeps it.
                Some(position) => {
                    if let Some(cell) = self
                        .cells
                        .iter()
                        .find(|cell| cell.pressable && cell.bounds.contains(position))
                        && state.focused.as_ref() != Some(&cell.key)
                    {
                        state.focused = Some(cell.key.clone());
                        shell.request_redraw();
                    }
                }
                None => {
                    if state.focused.take().is_some() {
                        shell.request_redraw();
                    }
                }
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        let Some(ring) = &self.ring else {
            return;
        };

        let Some(cell) = self.focused(tree.state.downcast_ref::<State<K>>()) else {
            return;
        };

        let ring = ring(theme);
        let bounds = cell.bounds + Vector::new(layout.bounds().x, layout.bounds().y);
        let outer = bounds.expand(ring.width);

        renderer.with_layer(outer, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: outer,
                    border: Border {
                        color: ring.color,
                        width: ring.width,
                        radius: (ring.radius + ring.width).into(),
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, K, Message, Theme, Renderer> From<FocusGrid<'a, K, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    K: Clone + PartialEq + 'static,
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(grid: FocusGrid<'a, K, Message, Theme, Renderer>) -> Self {
        Element::new(grid)
    }
}
