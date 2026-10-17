//! A vertical list of selectable items with a highlighted item.
//!
//! [`Listbox`] is the headless building block behind menus, selects and
//! command palettes. It keeps track of the highlighted item, which follows
//! the mouse and can be moved with the arrow keys; <kbd>Enter</kbd> or a
//! click selects it.
//!
//! Items are plain elements: the listbox draws the highlight behind them and
//! switches their text color.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key};
use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector,
    mouse, touch,
};

std::thread_local! {
    /// Whether the item being drawn is highlighted, for [`ItemRow::trailing_color`].
    static HIGHLIGHTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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
        });
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

    /// Sets whether the arrow keys and <kbd>Enter</kbd> are handled.
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

    /// Returns whether the listbox has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Default)]
struct State {
    active: Option<usize>,
    touched: bool,
    pressed: Option<usize>,
}

impl<Message, Theme, Renderer> Listbox<'_, Message, Theme, Renderer> {
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
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|child| Tree::new(&child.content))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
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
            return;
        }

        let hovered = cursor.position().and_then(|position| {
            layout
                .children()
                .position(|layout| layout.bounds().contains(position))
                .filter(|index| self.children[*index].selectable)
        });

        let active = self.active(tree.state.downcast_ref::<State>());
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(index) = hovered {
                    state.pressed = Some(index);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(pressed) = state.pressed.take()
                    && hovered == Some(pressed)
                    && let Some(message) = &self.children[pressed].on_select
                {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                ..
            }) if self.keyboard => match named {
                key::Named::ArrowDown | key::Named::ArrowUp => {
                    state.active = self.step(active, *named == key::Named::ArrowDown);
                    state.touched = true;
                    shell.request_redraw();
                    shell.capture_event();
                }
                key::Named::Home | key::Named::End => {
                    state.active = self.step(None, *named == key::Named::Home);
                    state.touched = true;
                    shell.request_redraw();
                    shell.capture_event();
                }
                key::Named::Enter => {
                    if let Some(message) =
                        active.and_then(|index| self.children[index].on_select.clone())
                    {
                        shell.publish(message);
                        shell.capture_event();
                    }
                }
                _ => {}
            },
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
