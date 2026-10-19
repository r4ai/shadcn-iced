//! A vertical list of selectable items with a highlighted item.
//!
//! [`Listbox`] is the headless building block behind menus, selects and
//! command palettes. It keeps track of the highlighted item, which follows
//! the mouse and can be moved with the arrow keys; <kbd>Enter</kbd> or a
//! click selects it. A finger selects an item with a tap, and a drag over
//! the items scrolls their scrollable instead, like on mobile browsers.
//!
//! Lists that own the keyboard, like menus and selects, can also be
//! [`typeahead`](Listbox::typeahead) lists: <kbd>Space</kbd> selects too,
//! and typing moves the highlight to the item whose
//! [`text_value`](Listbox::text_value) starts with what was typed.
//!
//! Items are plain elements: the listbox draws the highlight behind them and
//! switches their text color.
//!
//! A listbox in a scrollable is wrapped in a [`Region`], which scrolls the
//! highlighted item into view when it is moved with the keyboard, and can
//! hand the keyboard to the listbox only while the focus is within it.
use super::focus;
use super::pointer::{self, Gesture};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::scrollable::{AbsoluteOffset, Scrollable};
use iced::advanced::widget::{Id, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key};
use iced::time::{Duration, Instant};
use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector,
    mouse, touch,
};

std::thread_local! {
    /// Whether the item being drawn is highlighted, for [`ItemRow::trailing_color`].
    static HIGHLIGHTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    /// How far to scroll to reveal the item just highlighted with the
    /// keyboard, for the [`Region`] around the listbox.
    static REVEAL: std::cell::Cell<Option<Vector>> = const { std::cell::Cell::new(None) };
}

/// The look of the highlighted item of a [`Listbox`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The background of the highlighted item.
    pub background: Background,
    /// The border of the highlighted item.
    pub border: Border,
    /// The text color of the highlighted item.
    pub text_color: Color,
}

struct Child<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_select: Option<Message>,
    selectable: bool,
    style: Option<Box<dyn Fn(&Theme) -> Style + 'a>>,
    text: Option<String>,
}

/// A vertical list of selectable items.
pub struct Listbox<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    children: Vec<Child<'a, Message, Theme, Renderer>>,
    spacing: f32,
    padding: Padding,
    width: Length,
    min_width: f32,
    keyboard: bool,
    highlight_first: bool,
    initial: Option<usize>,
    clear_on_leave: bool,
    typeahead: bool,
    vim_bindings: bool,
    key: u64,
    style: Box<dyn Fn(&Theme) -> Style + 'a>,
}

impl<'a, Message, Theme, Renderer> Listbox<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates an empty [`Listbox`] with the given highlight style.
    pub fn new(style: impl Fn(&Theme) -> Style + 'a) -> Self {
        Self {
            children: Vec::new(),
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            min_width: 0.0,
            keyboard: true,
            highlight_first: false,
            initial: None,
            clear_on_leave: true,
            typeahead: false,
            vim_bindings: false,
            key: 0,
            style: Box::new(style),
        }
    }

    /// Appends a selectable item. Items without a message are disabled.
    pub fn item(
        mut self,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_select: Option<Message>,
    ) -> Self {
        let selectable = on_select.is_some();

        self.children.push(Child {
            content: content.into(),
            on_select,
            selectable,
            style: None,
            text: None,
        });
        self
    }

    /// Appends a selectable item highlighted with its own style, e.g. a
    /// destructive menu item.
    pub fn styled_item(
        mut self,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_select: Option<Message>,
        style: impl Fn(&Theme) -> Style + 'a,
    ) -> Self {
        let selectable = on_select.is_some();

        self.children.push(Child {
            content: content.into(),
            on_select,
            selectable,
            style: Some(Box::new(style)),
            text: None,
        });
        self
    }

    /// Appends an item that can be highlighted but does nothing when
    /// selected, e.g. a submenu trigger.
    pub fn passive(mut self, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.children.push(Child {
            content: content.into(),
            on_select: None,
            selectable: true,
            style: None,
            text: None,
        });
        self
    }

    /// Appends a non-selectable element (a label, a separator...).
    pub fn push(mut self, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.children.push(Child {
            content: content.into(),
            on_select: None,
            selectable: false,
            style: None,
            text: None,
        });
        self
    }

    /// Sets the text of the last appended item, which
    /// [typeahead](Self::typeahead) matches what was typed against (the
    /// `textValue` of Radix items). Items without one are skipped by
    /// typeahead.
    pub fn text_value(mut self, text: impl Into<String>) -> Self {
        if let Some(child) = self.children.last_mut() {
            child.text = Some(text.into());
        }
        self
    }

    /// Sets the space between children.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the minimum width of the items.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Sets whether the navigation keys (the arrows, <kbd>Home</kbd>,
    /// <kbd>End</kbd>, <kbd>PageUp</kbd>, <kbd>PageDown</kbd>) and
    /// <kbd>Enter</kbd> are handled.
    pub fn keyboard(mut self, keyboard: bool) -> Self {
        self.keyboard = keyboard;
        self
    }

    /// Highlights the first item until the user moves the highlight, like a
    /// command palette.
    pub fn highlight_first(mut self, highlight: bool) -> Self {
        self.highlight_first = highlight;
        self
    }

    /// Highlights the child at `index` (e.g. the selected option of a
    /// select) until the user moves the highlight.
    pub fn highlighted(mut self, index: Option<usize>) -> Self {
        self.initial = index;
        self
    }

    /// Sets whether the highlight is cleared when the cursor leaves the
    /// list. Defaults to `true`.
    pub fn clear_on_leave(mut self, clear: bool) -> Self {
        self.clear_on_leave = clear;
        self
    }

    /// Makes the list behave like the content of a Radix menu or select,
    /// which owns the keyboard: <kbd>Space</kbd> selects the highlighted
    /// item like <kbd>Enter</kbd>, and typed characters highlight the next
    /// item whose [`text_value`](Self::text_value) starts with them.
    /// Characters typed less than a second apart make up one search, and
    /// repeating a single character cycles through the items starting with
    /// it; <kbd>Space</kbd> is part of a search while typing one.
    ///
    /// Lists driven by a text input (a combobox, a command) leave it off
    /// (the default): there, <kbd>Space</kbd> and characters type into the
    /// input.
    pub fn typeahead(mut self, typeahead: bool) -> Self {
        self.typeahead = typeahead;
        self
    }

    /// Also moves the highlight with <kbd>Ctrl</kbd>+<kbd>N</kbd> or
    /// <kbd>Ctrl</kbd>+<kbd>J</kbd> (next) and <kbd>Ctrl</kbd>+<kbd>P</kbd>
    /// or <kbd>Ctrl</kbd>+<kbd>K</kbd> (previous), like the `vimBindings`
    /// of cmdk. Defaults to `false`.
    pub fn vim_bindings(mut self, vim_bindings: bool) -> Self {
        self.vim_bindings = vim_bindings;
        self
    }

    /// Identifies the items, e.g. by the query filtering them. When it
    /// changes, or when the items do (their number, or which ones are
    /// selectable), the highlight goes back to where it starts: the
    /// [`highlighted`](Self::highlighted) item, or the first one with
    /// [`highlight_first`](Self::highlight_first), like cmdk.
    pub fn key(mut self, key: impl std::hash::Hash) -> Self {
        use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};

        self.key = BuildHasherDefault::<DefaultHasher>::default().hash_one(key);
        self
    }

    /// Returns whether the listbox has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Default)]
struct State {
    active: Option<usize>,
    touched: bool,
    /// The item pressed with the mouse.
    pressed: Option<usize>,
    /// The item pressed with a finger.
    touch: Option<TouchPress>,
    typeahead: Typeahead,
    /// The [`Listbox::items`] the highlight is about.
    items: u64,
}

/// A finger pressing an item, which selects it when it lifts without
/// moving (a tap), and leaves it to the scrollable around the list when it
/// moves (a scroll).
#[derive(Debug, Clone, Copy)]
struct TouchPress {
    press: pointer::Press,
    item: usize,
    /// The highlight before the press, restored when the press turns into
    /// a scroll.
    before: (Option<usize>, bool),
}

/// How many items <kbd>PageUp</kbd> and <kbd>PageDown</kbd> move the
/// highlight by.
const PAGE: usize = 10;

impl<Message, Theme, Renderer> Listbox<'_, Message, Theme, Renderer> {
    /// Identifies the items: their key, their number and which ones are
    /// selectable. The highlight, a child index, is only valid for them.
    fn items(&self) -> u64 {
        use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hasher};

        let mut hasher = BuildHasherDefault::<DefaultHasher>::default().build_hasher();
        hasher.write_u64(self.key);
        hasher.write_usize(self.children.len());

        for child in &self.children {
            hasher.write_u8(u8::from(child.selectable));
        }

        hasher.finish()
    }

    fn active(&self, state: &State) -> Option<usize> {
        let active = state.active.filter(|index| {
            self.children
                .get(*index)
                .is_some_and(|child| child.selectable)
        });

        if active.is_none() && !state.touched {
            if let Some(index) = self.initial.filter(|index| {
                self.children
                    .get(*index)
                    .is_some_and(|child| child.selectable)
            }) {
                return Some(index);
            }

            if self.highlight_first {
                return self.children.iter().position(|child| child.selectable);
            }
        }

        active
    }

    fn step(&self, from: Option<usize>, forward: bool) -> Option<usize> {
        let count = self.children.len();
        if count == 0 {
            return None;
        }

        let mut index = from;

        for _ in 0..count {
            let next = match (index, forward) {
                (None, true) => 0,
                (None, false) => count - 1,
                (Some(i), true) => (i + 1) % count,
                (Some(i), false) => (i + count - 1) % count,
            };

            if self.children[next].selectable {
                return Some(next);
            }

            index = Some(next);
        }

        None
    }

    /// The highlight moved by up to `count` selectable items, stopping at
    /// the ends rather than wrapping around.
    fn page(&self, from: Option<usize>, forward: bool, count: usize) -> Option<usize> {
        let Some(mut index) = from else {
            return self.step(None, forward);
        };

        let mut moved = 0;
        let mut next = index;

        while moved < count {
            next = match (forward, next) {
                (true, next) if next + 1 < self.children.len() => next + 1,
                (false, next) if next > 0 => next - 1,
                _ => break,
            };

            if self.children[next].selectable {
                index = next;
                moved += 1;
            }
        }

        Some(index)
    }

    /// The next item whose text starts with the `search` typed ahead, from
    /// the `active` one.
    fn find(&self, search: &str, active: Option<usize>) -> Option<usize> {
        next_match(
            search,
            self.children
                .iter()
                .enumerate()
                .filter(|(_, child)| child.selectable)
                .filter_map(|(index, child)| Some((index, child.text.as_deref()?))),
            active,
        )
    }

    /// Adds `text` to the search typed ahead, and returns the item to
    /// highlight: the next match, or the `active` item if nothing matches.
    fn type_ahead(
        &self,
        state: &mut State,
        text: &str,
        active: Option<usize>,
        now: Instant,
    ) -> Option<usize> {
        let search = state.typeahead.push(text, now);

        self.find(search, active).or(active)
    }
}

impl<Message: Clone, Theme, Renderer> Listbox<'_, Message, Theme, Renderer> {
    /// Selects the `active` item with the keyboard.
    fn select(&self, active: Option<usize>, shell: &mut Shell<'_, Message>) {
        if let Some(message) = active.and_then(|index| self.children[index].on_select.clone()) {
            shell.publish(message);
            shell.capture_event();
            super::anchored::report_selected();
        }
    }
}

/// How long the characters typed ahead make up a single search: like Radix,
/// the search starts over a second after the last key.
pub const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(1000);

/// The characters typed ahead in a list, to move to the item starting with
/// them (Radix's `useTypeaheadSearch`). Used by [`Listbox::typeahead`], and
/// by anything picking an item by typing its text, like the closed trigger
/// of a select.
#[derive(Debug, Clone, Default)]
pub struct Typeahead {
    search: String,
    last: Option<Instant>,
}

impl Typeahead {
    /// Whether a search is going on: something was typed less than
    /// [`TYPEAHEAD_TIMEOUT`] ago. <kbd>Space</kbd> is then part of the
    /// search rather than selecting.
    pub fn is_typing(&self, now: Instant) -> bool {
        !self.search.is_empty()
            && self
                .last
                .is_some_and(|last| now.saturating_duration_since(last) < TYPEAHEAD_TIMEOUT)
    }

    /// Adds `text` to the search, which starts over when the previous key is
    /// older than [`TYPEAHEAD_TIMEOUT`], and returns the search.
    pub fn push(&mut self, text: &str, now: Instant) -> &str {
        if !self.is_typing(now) {
            self.search.clear();
        }

        self.search.push_str(text);
        self.last = Some(now);

        &self.search
    }
}

/// The item to move to for a `search` typed ahead, among `items` (the
/// index and the text of each enabled item, in order) from the `current`
/// one, like Radix's `getNextMatch`: the first item after the current one
/// (wrapping around) whose text starts with the search, ignoring case.
///
/// A single character skips the current item, and a character repeated
/// (`"bbb"`) searches for that character alone: repeating it cycles
/// through the items starting with it. Returns `None` when nothing else
/// matches.
pub fn next_match<'t>(
    search: &str,
    items: impl IntoIterator<Item = (usize, &'t str)>,
    current: Option<usize>,
) -> Option<usize> {
    let items: Vec<_> = items.into_iter().collect();
    let first = search.chars().next()?;

    let repeated = search.chars().count() > 1 && search.chars().all(|c| c == first);
    let search = if repeated {
        first.to_lowercase().collect::<String>()
    } else {
        search.to_lowercase()
    };
    let exclude_current = search.chars().count() == 1;

    let start = current
        .and_then(|current| items.iter().position(|(index, _)| *index == current))
        .unwrap_or(0);

    items[start..]
        .iter()
        .chain(&items[..start])
        .filter(|(index, _)| !(exclude_current && Some(*index) == current))
        .find(|(_, text)| text.to_lowercase().starts_with(&search))
        .map(|(index, _)| *index)
        .filter(|index| Some(*index) != current)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Listbox<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            items: self.items(),
            ..State::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|child| Tree::new(&child.content))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        // Other items (a query typed in a command): the highlight starts
        // over rather than pointing at whatever took the place of the item.
        let items = self.items();
        let state = tree.state.downcast_mut::<State>();

        if state.items != items {
            *state = State {
                items,
                ..State::default()
            };
        }

        tree.diff_children(
            &self
                .children
                .iter()
                .map(|child| &child.content)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).shrink(self.padding);
        let max_width = limits.max().width;

        // First pass: measure the natural width of every child.
        let mut width: f32 = 0.0;

        for (child, tree) in self.children.iter_mut().zip(&mut tree.children) {
            // Children filling the width (separators...) have no natural
            // width; they simply follow the others.
            if child.content.as_widget().size_hint().width.fill_factor() > 0 {
                continue;
            }

            let node = child.content.as_widget_mut().layout(
                tree,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(max_width, f32::INFINITY)),
            );
            width = width.max(node.size().width);
        }

        let width = if self.width == Length::Shrink {
            width
                .max(limits.min().width)
                .max(self.min_width)
                .min(max_width)
        } else {
            max_width
        };

        // Second pass: lay out every child with the final width, so items
        // can fill it (and highlights span the whole list).
        let mut y = self.padding.top;
        let mut children = Vec::with_capacity(self.children.len());

        for (child, tree) in self.children.iter_mut().zip(&mut tree.children) {
            let node = child
                .content
                .as_widget_mut()
                .layout(
                    tree,
                    renderer,
                    &layout::Limits::new(Size::new(width, 0.0), Size::new(width, f32::INFINITY)),
                )
                .move_to(Point::new(self.padding.left, y));

            y += node.size().height + self.spacing;
            children.push(node);
        }

        if !children.is_empty() {
            y -= self.spacing;
        }

        layout::Node::with_children(
            Size::new(
                width + self.padding.left + self.padding.right,
                y + self.padding.bottom,
            ),
            children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((child, tree), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                child
                    .content
                    .as_widget_mut()
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        pointer::observe(event);

        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.content.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        if shell.is_event_captured() {
            if let Event::Touch(
                touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. },
            ) = event
            {
                tree.state.downcast_mut::<State>().touch = None;
            }

            return;
        }

        let item_at = |cursor: mouse::Cursor| {
            cursor.position().and_then(|position| {
                layout
                    .children()
                    .position(|layout| layout.bounds().contains(position))
                    .filter(|index| self.children[*index].selectable)
            })
        };

        // The item under the pointer, wherever it is (a finger that just
        // lifted is still where it was), and the item hovered: never by a
        // lifted finger.
        let pointed = item_at(cursor);
        let hovered = item_at(pointer::hover(cursor));

        let active = self.active(tree.state.downcast_ref::<State>());
        let state = tree.state.downcast_mut::<State>();

        match event {
            // The pointer moving to an open submenu leaves its trigger
            // highlighted.
            Event::Mouse(mouse::Event::CursorMoved { position })
                if super::anchored::is_in_pointer_grace(*position) => {}
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let next = match hovered {
                    Some(index) => Some(index),
                    None if self.clear_on_leave => None,
                    None => state.active,
                };

                if hovered.is_some() {
                    state.touched = true;
                }

                if next != state.active {
                    state.active = next;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) if self.clear_on_leave => {
                state.active = None;
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(index) = pointed {
                    state.pressed = Some(index);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(pressed) = state.pressed.take()
                    && pointed == Some(pressed)
                    && let Some(message) = &self.children[pressed].on_select
                {
                    shell.publish(message.clone());
                    shell.capture_event();
                    super::anchored::report_selected();
                }
            }
            // A finger highlights the item it presses (like `:active`), but
            // the press is not captured: the scrollable around the list
            // scrolls if the finger moves. Like Radix, which leaves
            // highlighting on `pointermove` to mice, a finger moving over
            // the items does not highlight them.
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if state.touch.is_none()
                    && let Some(item) = pointed
                    && let Some(press) = pointer::Press::start(event, cursor, |_| true)
                {
                    state.touch = Some(TouchPress {
                        press,
                        item,
                        before: (state.active, state.touched),
                    });
                    state.active = Some(item);
                    state.touched = true;
                    shell.request_redraw();
                }
            }
            Event::Touch(
                touch::Event::FingerMoved { .. }
                | touch::Event::FingerLifted { .. }
                | touch::Event::FingerLost { .. },
            ) => {
                if let Some(mut press) = state.touch {
                    match press.press.update(event, cursor) {
                        // A scroll: the item is not pressed anymore.
                        Some(Gesture::Cancelled) => {
                            (state.active, state.touched) = press.before;
                            state.touch = Some(press);
                            shell.request_redraw();
                        }
                        // Captured, so that a popup closes on the selection.
                        Some(Gesture::Tapped(_)) => {
                            state.touch = None;

                            if pointed == Some(press.item)
                                && let Some(message) = &self.children[press.item].on_select
                            {
                                shell.publish(message.clone());
                                shell.capture_event();
                                super::anchored::report_selected();
                            }
                        }
                        Some(Gesture::Released) => {
                            state.touch = None;

                            if !press.press.moved {
                                (state.active, state.touched) = press.before;
                                shell.request_redraw();
                            }
                        }
                        Some(Gesture::Pressed(_) | Gesture::Moved(_)) => {
                            state.touch = Some(press);
                        }
                        // Another finger.
                        None => {}
                    }
                }

                // The finger left the screen: the touch equivalent of the
                // cursor leaving the list.
                if let Event::Touch(
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. },
                ) = event
                    && state.touch.is_none()
                    && self.clear_on_leave
                    && state.active.is_some()
                {
                    state.active = None;
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if self.keyboard =>
            {
                let now = Instant::now();
                let typing = self.typeahead && state.typeahead.is_typing(now);
                // `Alt` with an arrow opens or closes a combobox, without
                // moving the highlight.
                let plain = !modifiers.alt() && !modifiers.control() && !modifiers.logo();

                let next =
                    match key.as_ref() {
                        keyboard::Key::Named(
                            named @ (key::Named::ArrowDown | key::Named::ArrowUp),
                        ) if plain => Some(self.step(active, named == key::Named::ArrowDown)),
                        keyboard::Key::Named(named @ (key::Named::Home | key::Named::End)) => {
                            Some(self.step(None, named == key::Named::Home))
                        }
                        keyboard::Key::Named(
                            named @ (key::Named::PageUp | key::Named::PageDown),
                        ) => Some(self.page(active, named == key::Named::PageDown, PAGE)),
                        keyboard::Key::Character(c)
                            if self.vim_bindings
                                && modifiers.control()
                                && !modifiers.alt()
                                && !modifiers.logo()
                                && matches!(c.to_lowercase().as_str(), "n" | "j" | "p" | "k") =>
                        {
                            Some(self.step(active, matches!(c.to_lowercase().as_str(), "n" | "j")))
                        }
                        // While typing ahead, `Space` is part of the search.
                        keyboard::Key::Named(key::Named::Space) if typing => {
                            Some(self.type_ahead(state, " ", active, now))
                        }
                        // Like any key of a menu, which owns the keyboard, the
                        // characters are captured even when nothing matches.
                        keyboard::Key::Character(c) if self.typeahead && plain => {
                            Some(self.type_ahead(state, c, active, now))
                        }
                        keyboard::Key::Named(key::Named::Enter) => {
                            self.select(active, shell);
                            None
                        }
                        keyboard::Key::Named(key::Named::Space) if self.typeahead => {
                            self.select(active, shell);
                            None
                        }
                        _ => None,
                    };

                if let Some(next) = next {
                    state.active = next;
                    state.touched = true;
                    reveal(layout, state.active, viewport);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }
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
        let active = self.active(tree.state.downcast_ref::<State>());
        let highlight = (self.style)(theme);

        for (index, ((child, tree), layout)) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            let is_active = active == Some(index);
            let highlight = match &child.style {
                Some(style) if is_active => style(theme),
                _ => highlight,
            };

            if is_active {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        border: highlight.border,
                        ..renderer::Quad::default()
                    },
                    highlight.background,
                );
            }

            let was_highlighted = HIGHLIGHTED.replace(is_active);

            child.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: if is_active {
                        highlight.text_color
                    } else {
                        style.text_color
                    },
                },
                layout,
                cursor,
                viewport,
            );

            HIGHLIGHTED.set(was_highlighted);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                let interaction = child
                    .content
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer);

                if interaction == mouse::Interaction::None && cursor.is_over(layout.bounds()) {
                    if child.on_select.is_some() {
                        mouse::Interaction::Pointer
                    } else {
                        mouse::Interaction::Idle
                    }
                } else {
                    interaction
                }
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let overlays: Vec<_> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, tree), layout)| {
                child
                    .content
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Listbox<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(listbox: Listbox<'a, Message, Theme, Renderer>) -> Self {
        Element::new(listbox)
    }
}

/// Asks the [`Region`] around the listbox to scroll the item at `index`
/// into the `viewport`, by as little as possible (`block: "nearest"`).
fn reveal(layout: Layout<'_>, index: Option<usize>, viewport: &Rectangle) {
    let Some(item) = index.and_then(|index| layout.children().nth(index)) else {
        return;
    };

    let item = item.bounds();
    let top = item.y - viewport.y;
    let bottom = item.y + item.height - (viewport.y + viewport.height);

    // The top of an item taller than the viewport.
    let delta = if top < 0.0 || item.height > viewport.height {
        top
    } else if bottom > 0.0 {
        bottom
    } else {
        return;
    };

    if delta != 0.0 {
        REVEAL.set(Some(Vector::new(0.0, delta)));
    }
}

/// The region of a [`Listbox`]: scrolls the item highlighted with the
/// keyboard into view, in the first scrollable within it, and can hand the
/// navigation keys to the listbox only while the focus is within it.
pub struct Region<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    focus_within: bool,
}

/// Creates a [`Region`] around `content`, a scrollable [`Listbox`] or
/// anything containing it (e.g. a command with its search field).
pub fn region<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Region<'a, Message, Theme, Renderer> {
    Region {
        content: content.into(),
        focus_within: false,
    }
}

impl<Message, Theme, Renderer> Region<'_, Message, Theme, Renderer> {
    /// Hands the navigation keys (<kbd>↑</kbd>, <kbd>↓</kbd>,
    /// <kbd>Home</kbd>, <kbd>End</kbd>, <kbd>PageUp</kbd>,
    /// <kbd>PageDown</kbd>, the [`vim_bindings`](Listbox::vim_bindings) and
    /// <kbd>Enter</kbd>) to the listbox
    /// only while a control of the region (its search field) is focused,
    /// like the root of cmdk; otherwise, the listbox reacts to them
    /// anywhere in the window. Defaults to `false`, for listboxes in
    /// popups, which own the keyboard while they are open.
    pub fn focus_within(mut self, focus_within: bool) -> Self {
        self.focus_within = focus_within;
        self
    }
}

/// Whether a key moves the highlight of a listbox or selects its item, for
/// [`Region::focus_within`].
fn is_navigation(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
    match key.as_ref() {
        keyboard::Key::Named(named) => matches!(
            named,
            key::Named::ArrowDown
                | key::Named::ArrowUp
                | key::Named::Home
                | key::Named::End
                | key::Named::PageUp
                | key::Named::PageDown
                | key::Named::Enter
        ),
        // The `vimBindings` of cmdk.
        keyboard::Key::Character(c) => {
            modifiers.control() && matches!(c.to_lowercase().as_str(), "n" | "j" | "p" | "k")
        }
        keyboard::Key::Unidentified => false,
    }
}

/// Scrolls the first scrollable it meets by a delta.
struct ScrollBy {
    delta: Vector,
    done: bool,
}

impl Operation for ScrollBy {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        if !self.done {
            operate(self);
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if !self.done {
            state.scroll_by(
                AbsoluteOffset {
                    x: self.delta.x,
                    y: self.delta.y,
                },
                bounds,
                content_bounds,
            );
            self.done = true;
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Region<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if self.focus_within
            && let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && is_navigation(key, *modifiers)
            && !focus::is_focus_within(&mut self.content, &mut tree.children[0], layout, renderer)
        {
            return;
        }

        // A listbox outside of the region may still be waiting for its own.
        let outer = REVEAL.take();

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if let Some(delta) = REVEAL.take() {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut ScrollBy { delta, done: false },
            );
            shell.request_redraw();
        }

        REVEAL.set(outer);
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
            layout,
            cursor,
            viewport,
        );
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
            layout,
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
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Region<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(region: Region<'a, Message, Theme, Renderer>) -> Self {
        Element::new(region)
    }
}

/// An item laid out as `content` on the left and `trailing` on the right
/// (a shortcut, a check mark, a chevron...).
///
/// iced has no intrinsic width for elements that fill their parent, so a
/// [`Listbox`] could not size itself to its widest item if items used
/// [`Length::Fill`] to push their trailing part to the right. An [`ItemRow`]
/// shrinks to fit its parts when measured, and stretches when given a
/// minimum width, which is exactly what [`Listbox`] does.
pub struct ItemRow<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    trailing: Option<Element<'a, Message, Theme, Renderer>>,
    trailing_color: Option<Box<dyn Fn(&Theme, bool) -> Color + 'a>>,
    spacing: f32,
    padding: Padding,
}

/// Creates an [`ItemRow`].
pub fn row<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    trailing: Option<Element<'a, Message, Theme, Renderer>>,
) -> ItemRow<'a, Message, Theme, Renderer> {
    ItemRow {
        content: content.into(),
        trailing,
        trailing_color: None,
        spacing: 16.0,
        padding: Padding::ZERO,
    }
}

impl<'a, Message, Theme, Renderer> ItemRow<'a, Message, Theme, Renderer> {
    /// Sets the minimum space between the content and the trailing part.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text color of the trailing part, given whether the item is
    /// highlighted in its [`Listbox`], e.g. a shortcut that is muted
    /// unless highlighted (`text-muted-foreground
    /// group-focus:text-accent-foreground`). By default, it inherits the
    /// color of the item.
    pub fn trailing_color(mut self, f: impl Fn(&Theme, bool) -> Color + 'a) -> Self {
        self.trailing_color = Some(Box::new(f));
        self
    }

    /// Sets the padding around both parts.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ItemRow<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.trailing.as_ref().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let mut children = vec![self.content.as_widget()];
        children.extend(self.trailing.as_ref().map(Element::as_widget));
        tree.diff_children(&children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.shrink(self.padding);
        let loose = layout::Limits::new(Size::ZERO, limits.max());

        let mut trailing = self.trailing.as_mut().map(|trailing| {
            trailing
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &loose)
        });

        let trailing_width = trailing
            .as_ref()
            .map_or(0.0, |node| node.size().width + self.spacing);

        let content = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                Size::new(
                    (limits.max().width - trailing_width).max(0.0),
                    limits.max().height,
                ),
            ),
        );

        let natural = content.size().width + trailing_width;
        let width = natural.max(limits.min().width);
        let height = content
            .size()
            .height
            .max(trailing.as_ref().map_or(0.0, |node| node.size().height));

        let content_height = content.size().height;
        let content = content.move_to(Point::new(
            self.padding.left,
            self.padding.top + (height - content_height) / 2.0,
        ));

        let mut children = vec![content];

        if let Some(node) = trailing.take() {
            let size = node.size();
            children.push(node.move_to(Point::new(
                self.padding.left + width - size.width,
                self.padding.top + (height - size.height) / 2.0,
            )));
        }

        layout::Node::with_children(
            Size::new(
                width + self.padding.left + self.padding.right,
                height + self.padding.top + self.padding.bottom,
            ),
            children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut layouts = layout.children();
        let content = layouts.next().unwrap();

        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], content, renderer, operation);

        if let (Some(trailing), Some(layout)) = (self.trailing.as_mut(), layouts.next()) {
            trailing
                .as_widget_mut()
                .operate(&mut tree.children[1], layout, renderer, operation);
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
        let mut layouts = layout.children();
        let content = layouts.next().unwrap();

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

        if let (Some(trailing), Some(layout)) = (self.trailing.as_mut(), layouts.next()) {
            trailing.as_widget_mut().update(
                &mut tree.children[1],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
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
        let mut layouts = layout.children();
        let content = layouts.next().unwrap();

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content,
            cursor,
            viewport,
        );

        if let (Some(trailing), Some(layout)) = (self.trailing.as_ref(), layouts.next()) {
            let style = match &self.trailing_color {
                Some(f) => &renderer::Style {
                    text_color: f(theme, HIGHLIGHTED.get()),
                },
                None => style,
            };

            trailing.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut layouts = layout.children();
        let content = layouts.next().unwrap();

        let interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content,
            cursor,
            viewport,
            renderer,
        );

        match (self.trailing.as_ref(), layouts.next()) {
            (Some(trailing), Some(layout)) => {
                interaction.max(trailing.as_widget().mouse_interaction(
                    &tree.children[1],
                    layout,
                    cursor,
                    viewport,
                    renderer,
                ))
            }
            _ => interaction,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut layouts = layout.children();
        let content = layouts.next().unwrap();
        let (first, rest) = tree.children.split_at_mut(1);

        let mut overlays: Vec<_> = self
            .content
            .as_widget_mut()
            .overlay(&mut first[0], content, renderer, viewport, translation)
            .into_iter()
            .collect();

        if let (Some(trailing), Some(layout)) = (self.trailing.as_mut(), layouts.next()) {
            overlays.extend(trailing.as_widget_mut().overlay(
                &mut rest[0],
                layout,
                renderer,
                viewport,
                translation,
            ));
        }

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<ItemRow<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(row: ItemRow<'a, Message, Theme, Renderer>) -> Self {
        Element::new(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRUITS: [(usize, &str); 4] =
        [(0, "Apple"), (1, "Banana"), (2, "Blueberry"), (4, "Cherry")];

    #[test]
    fn typing_moves_to_the_item_starting_with_the_search() {
        assert_eq!(next_match("b", FRUITS, None), Some(1));
        assert_eq!(next_match("BLU", FRUITS, None), Some(2));
        assert_eq!(next_match("ch", FRUITS, Some(1)), Some(4));
        assert_eq!(next_match("x", FRUITS, None), None);
    }

    #[test]
    fn repeating_a_character_cycles_through_its_items() {
        assert_eq!(next_match("b", FRUITS, Some(1)), Some(2));
        assert_eq!(next_match("bb", FRUITS, Some(2)), Some(1));
        // The only match is the current item: nothing to move to.
        assert_eq!(next_match("a", FRUITS, Some(0)), None);
        // A longer search keeps the current item when it still matches.
        assert_eq!(next_match("ba", FRUITS, Some(1)), None);
    }

    #[test]
    fn the_search_starts_over_after_a_second() {
        let now = Instant::now();
        let mut typeahead = Typeahead::default();

        assert_eq!(typeahead.push("b", now), "b");
        assert_eq!(typeahead.push("l", now + Duration::from_millis(500)), "bl");
        assert!(typeahead.is_typing(now + Duration::from_millis(900)));
        assert!(!typeahead.is_typing(now + Duration::from_millis(1600)));
        assert_eq!(typeahead.push("c", now + Duration::from_millis(1600)), "c");
    }
}
