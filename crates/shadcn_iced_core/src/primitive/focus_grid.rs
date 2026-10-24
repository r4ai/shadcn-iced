//! The focused cell of a grid, like the days of a calendar.
//!
//! The cells of a grid (the days of react-day-picker) are not focusable
//! widgets of their own: [`FocusGrid`] knows where they are and which one
//! is focused. A pressed (or tapped) cell takes the focus, a press outside
//! of the grid blurs it, and when the focused cell is no longer displayed
//! (the months changed) the focus moves to a fallback cell, e.g. the
//! selected day. The focused cell shows a [`Ring`] outside of its bounds.
//!
//! Like the grids of the WAI-ARIA APG, the grid is a single <kbd>Tab</kbd>
//! stop (a roving `tabindex`): the focus operations focus the fallback
//! cell, and the keys move the focus between the cells
//! ([`FocusGrid::on_move`]) or activate the focused one
//! ([`FocusGrid::on_activate`]).
//!
//! Cells are identified by a key of yours, which survives the views.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::focus::Ring;
//! # use shadcn_iced_core::primitive::focus_grid::{Cell, Move, focus_grid};
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
//!     .on_move(|day, step| match step {
//!         Move::Left if *day > 0 => Some((day - 1, None)),
//!         Move::Right if *day < 6 => Some((day + 1, None)),
//!         _ => None,
//!     })
//!     .ring(|_theme| Ring { color: Color::BLACK, width: 3.0, radius: 6.0.into() })
//!     .into()
//! # }
//! ```
use super::focus::{self, Ring};
use super::pointer::{self, Gesture, Press};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{Focusable, focusable};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse, touch};

use std::cell::Cell as Flag;

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

/// A key moving the focus of a [`FocusGrid`], like in the date grid of the
/// WAI-ARIA APG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    /// <kbd>←</kbd>: the previous cell.
    Left,
    /// <kbd>→</kbd>: the next cell.
    Right,
    /// <kbd>↑</kbd>: the cell above.
    Up,
    /// <kbd>↓</kbd>: the cell below.
    Down,
    /// <kbd>Home</kbd>: the first cell of the row.
    Home,
    /// <kbd>End</kbd>: the last cell of the row.
    End,
    /// <kbd>PageUp</kbd>, or <kbd>Shift</kbd>+<kbd>PageUp</kbd> when `far`:
    /// the previous page (a month), or a further one (a year).
    PageUp {
        /// Whether <kbd>Shift</kbd> is held.
        far: bool,
    },
    /// <kbd>PageDown</kbd>, or <kbd>Shift</kbd>+<kbd>PageDown</kbd> when
    /// `far`: the next page, or a further one.
    PageDown {
        /// Whether <kbd>Shift</kbd> is held.
        far: bool,
    },
}

impl Move {
    /// The move of a key press, if any.
    fn of(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Self> {
        if modifiers.control() || modifiers.alt() || modifiers.logo() {
            return None;
        }

        let far = modifiers.shift();

        Some(match key {
            keyboard::Key::Named(Named::ArrowLeft) => Self::Left,
            keyboard::Key::Named(Named::ArrowRight) => Self::Right,
            keyboard::Key::Named(Named::ArrowUp) => Self::Up,
            keyboard::Key::Named(Named::ArrowDown) => Self::Down,
            keyboard::Key::Named(Named::Home) => Self::Home,
            keyboard::Key::Named(Named::End) => Self::End,
            keyboard::Key::Named(Named::PageUp) => Self::PageUp { far },
            keyboard::Key::Named(Named::PageDown) => Self::PageDown { far },
            _ => return None,
        })
    }
}

thread_local! {
    /// Whether the next grid shown with [`FocusGrid::focus_on_show`] takes
    /// the focus.
    static FOCUS_ON_SHOW: Flag<bool> = const { Flag::new(false) };
}

/// Asks the next [`FocusGrid`] built with
/// [`focus_on_show`](FocusGrid::focus_on_show) to take the focus as soon as
/// it is shown, e.g. once the key focusing the trigger of a popover opened
/// it (the `autoFocus` of react-day-picker).
pub fn focus_when_shown() {
    FOCUS_ON_SHOW.with(|flag| flag.set(true));
}

/// Takes back a request of [`focus_when_shown`], e.g. when the popover is
/// opened with the pointer instead.
pub fn cancel_focus_when_shown() {
    FOCUS_ON_SHOW.with(|flag| flag.set(false));
}

/// What a key moving the focus leads to: the cell to focus, which may not be
/// displayed yet, and a message, e.g. to display another month.
pub type Step<K, Message> = (K, Option<Message>);

/// A grid with a focused cell. See the [module documentation](self).
#[allow(clippy::type_complexity)]
pub struct FocusGrid<'a, K, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    cells: Vec<Cell<K>>,
    fallback: Option<K>,
    ring: Option<Box<dyn Fn(&Theme) -> Ring + 'a>>,
    on_move: Option<Box<dyn Fn(&K, Move) -> Option<Step<K, Message>> + 'a>>,
    on_activate: Option<Box<dyn Fn(&K) -> Option<Message> + 'a>>,
    focus_on_show: bool,
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
        on_move: None,
        on_activate: None,
        focus_on_show: false,
    }
}

impl<'a, K, Message, Theme, Renderer> FocusGrid<'a, K, Message, Theme, Renderer>
where
    K: PartialEq,
{
    /// Sets the cell that takes the focus when the focused cell is not
    /// displayed anymore, and when the grid is focused with
    /// <kbd>Tab</kbd> (react-day-picker's `calculateFocusTarget`).
    pub fn fallback(mut self, key: Option<K>) -> Self {
        self.fallback = key;
        self
    }

    /// Sets the ring drawn around the focused cell.
    pub fn ring(mut self, ring: impl Fn(&Theme) -> Ring + 'a) -> Self {
        self.ring = Some(Box::new(ring));
        self
    }

    /// Moves the focus with the arrow keys, <kbd>Home</kbd>, <kbd>End</kbd>,
    /// <kbd>PageUp</kbd> and <kbd>PageDown</kbd>: `f` receives the focused
    /// cell and the [`Move`], and returns the [`Step`] to take, if any.
    ///
    /// The keys are handled (and captured) while the grid has the focus and
    /// none of the controls of its content does.
    pub fn on_move(mut self, f: impl Fn(&K, Move) -> Option<Step<K, Message>> + 'a) -> Self {
        self.on_move = Some(Box::new(f));
        self
    }

    /// Sets the message produced when <kbd>Enter</kbd> or <kbd>Space</kbd>
    /// activates the focused cell, like a click on a focused `<button>`.
    pub fn on_activate(mut self, f: impl Fn(&K) -> Option<Message> + 'a) -> Self {
        self.on_activate = Some(Box::new(f));
        self
    }

    /// Takes the focus when shown after [`focus_when_shown`], e.g. in a
    /// popover opened with the keyboard.
    pub fn focus_on_show(mut self, focus: bool) -> Self {
        self.focus_on_show = focus;
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
    /// Whether the focus moved to a cell that the next view displays
    /// (a message changing the months was published), so that the keys
    /// pressed before it is built move on from there.
    moving: bool,
    /// The finger pressing the grid, focusing a cell once it taps.
    press: Option<Press>,
}

/// The focus of a grid, for focus operations: focusing the grid focuses the
/// fallback cell.
struct GridFocus<'a, K> {
    state: &'a mut State<K>,
    fallback: Option<&'a K>,
}

impl<K: Clone> Focusable for GridFocus<'_, K> {
    fn is_focused(&self) -> bool {
        self.state.focused.is_some()
    }

    fn focus(&mut self) {
        if self.state.focused.is_none() {
            self.state.focused = self.fallback.cloned();
        }
    }

    fn unfocus(&mut self) {
        self.state.focused = None;
        self.state.moving = false;
    }
}

impl<'a, K, Message, Theme, Renderer> FocusGrid<'a, K, Message, Theme, Renderer>
where
    K: Clone + PartialEq + 'static,
    Renderer: iced::advanced::Renderer,
{
    /// Focuses the cell at `position` (relative to the grid), or blurs the
    /// grid when `position` is outside of it.
    fn press(&self, state: &mut State<K>, position: Option<Point>, shell: &mut Shell<'_, Message>) {
        match position {
            // Cells that cannot be pressed do not take the focus, and the
            // rest of the grid keeps it.
            Some(position) => {
                if let Some(cell) = self
                    .cells
                    .iter()
                    .find(|cell| cell.pressable && cell.bounds.contains(position))
                    && state.focused.as_ref() != Some(&cell.key)
                {
                    state.focused = Some(cell.key.clone());
                    state.moving = false;
                    shell.request_redraw();
                }
            }
            None => {
                state.moving = false;

                if state.focused.take().is_some() {
                    shell.request_redraw();
                }
            }
        }
    }

    /// The cell the keys act on: the focused one, unless it is not displayed
    /// (the months changed), then the fallback, where the ring is.
    fn current(&self, state: &State<K>) -> Option<K> {
        let focused = state.focused.as_ref()?;

        if state.moving || self.cell(focused).is_some() {
            Some(focused.clone())
        } else {
            self.fallback.clone()
        }
    }

    /// Handles a key pressed while the grid has the focus.
    fn key(
        &mut self,
        tree: &mut Tree,
        key: &keyboard::Key,
        modifiers: keyboard::Modifiers,
        shell: &mut Shell<'_, Message>,
    ) {
        let state = tree.state.downcast_mut::<State<K>>();
        let Some(current) = self.current(state) else {
            return;
        };

        if let Some(on_move) = &self.on_move
            && let Some(step) = Move::of(key, modifiers)
        {
            // Like react-day-picker, the key is handled even when the focus
            // cannot move further (`preventDefault`).
            shell.capture_event();

            if let Some((key, message)) = on_move(&current, step) {
                state.moving = message.is_some();
                state.focused = Some(key);
                shell.request_redraw();

                if let Some(message) = message {
                    shell.publish(message);
                }
            }

            return;
        }

        let activates = matches!(key, keyboard::Key::Named(Named::Enter | Named::Space))
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo();

        if activates && let Some(on_activate) = &self.on_activate {
            state.focused = Some(current.clone());
            shell.capture_event();

            if let Some(message) = on_activate(&current) {
                shell.publish(message);
            }
        }
    }
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
        tree::State::new(State::<K> {
            focused: None,
            moving: false,
            press: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        // A new view displays the cells the keys moved to.
        tree.state.downcast_mut::<State<K>>().moving = false;
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

        // The grid comes after the controls of its content, like the days
        // of react-day-picker after its navigation. A grid without any cell
        // to focus is not a tab stop.
        let state = tree.state.downcast_mut::<State<K>>();

        if state.focused.is_some() || self.fallback.is_some() {
            let bounds = self
                .focused(state)
                .or_else(|| self.fallback.as_ref().and_then(|key| self.cell(key)))
                .map_or(layout.bounds(), |cell| {
                    cell.bounds + Vector::new(layout.bounds().x, layout.bounds().y)
                });

            operation.focusable(
                None,
                bounds,
                &mut GridFocus {
                    state,
                    fallback: self.fallback.as_ref(),
                },
            );
        }
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
        pointer::observe(event);

        let bounds = layout.bounds();
        let content = layout.children().next().unwrap();

        // Shown by a key (in a popover opened with the keyboard): the focus
        // moves to the fallback cell, away from the controls of the content.
        if self.focus_on_show && FOCUS_ON_SHOW.with(|flag| flag.replace(false)) {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content,
                renderer,
                &mut focusable::unfocus(),
            );

            let state = tree.state.downcast_mut::<State<K>>();
            state.focused = self.fallback.clone();
            state.moving = false;
            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State<K>>();
                self.press(state, cursor.position_in(bounds), shell);
            }
            // A finger focuses a cell once it taps, not when a scroll starts
            // on it, and blurs the grid with a tap elsewhere.
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = tree.state.downcast_mut::<State<K>>();

                if state.press.is_none() {
                    state.press = Press::start(event, cursor, |_| true);
                }
            }
            Event::Touch(_) => {
                let state = tree.state.downcast_mut::<State<K>>();

                if let Some(press) = &mut state.press {
                    match press.update(event, cursor) {
                        Some(Gesture::Tapped(position)) => {
                            state.press = None;

                            let position = bounds
                                .contains(position)
                                .then(|| position - Vector::new(bounds.x, bounds.y));
                            self.press(state, position, shell);
                        }
                        Some(Gesture::Released) => state.press = None,
                        _ => {}
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if tree.state.downcast_ref::<State<K>>().focused.is_some()
                    && !shell.is_event_captured()
                    // A focused control of the content (a navigation
                    // button) handles its keys.
                    && !focus::is_focus_within(
                        &mut self.content,
                        &mut tree.children[0],
                        content,
                        renderer,
                    ) =>
            {
                self.key(tree, key, *modifiers, shell);

                if shell.is_event_captured() {
                    return;
                }
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content,
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
        // A finger that lifted does not hover over the cell it tapped.
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            pointer::hover(cursor),
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

        // A layer does not clip to the one it is in: clipped to the viewport
        // too, the ring stays inside a scroll area.
        let Some(clip) = outer.intersection(viewport) else {
            return;
        };

        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: outer,
                    border: ring.border(),
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
