//! A styled native-like select, backed by iced's [`PickList`].
//!
//! Port of <https://ui.shadcn.com/docs/components/native-select>. For the
//! fully custom select with groups and separators, see
//! [`select`](super::select).
//!
//! Like a native `<select>`, it can be focused (with <kbd>Tab</kbd>), shows
//! a focus ring when focused with the keyboard, and while it is focused:
//!
//! - <kbd>↑</kbd> / <kbd>↓</kbd> pick the previous or next option, and
//!   <kbd>Home</kbd> / <kbd>End</kbd> the first or the last one;
//! - typing picks the next option starting with what was typed (characters
//!   typed less than a second apart make up one search);
//! - <kbd>Alt</kbd>+<kbd>↓</kbd>, <kbd>F4</kbd>, <kbd>Space</kbd> and
//!   <kbd>Enter</kbd> open the list, and close it again, like
//!   <kbd>Escape</kbd>.
//!
//! The list of [`PickList`] has no keyboard navigation of its own: while it
//! is open, the keys above still pick an option, and the list follows.
//!
//! ```no_run
//! # use shadcn_iced::native_select::native_select;
//! # #[derive(Clone)] enum Message { Fruit(&'static str) }
//! # fn view<'a>(fruit: Option<&'static str>) -> iced::Element<'a, Message> {
//! native_select(&["Apple", "Banana", "Blueberry"][..], fruit, Message::Fruit)
//!     .placeholder("Select a fruit")
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::listbox;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, operation, tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay, renderer};
use iced::keyboard::{self, key};
use iced::overlay::menu;
use iced::time::Instant;
use iced::widget::pick_list::{Handle, Icon, PickList, Status, Style};
use iced::widget::text;
use iced::{
    Background, Border, Color, Event, Length, Padding, Pixels, Rectangle, Size as Dimensions,
    Theme, Vector, border, mouse, touch,
};

use std::borrow::Borrow;
use std::rc::Rc;

/// The size of a [`NativeSelect`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `size="sm"`: `h-7 py-0.5 rounded-[min(var(--radius-md),10px)]`.
    Sm,
    /// `h-8 py-1 rounded-lg`.
    #[default]
    Default,
}

/// A native select. See the [module documentation](self).
pub struct NativeSelect<'a, T, Message> {
    options: Vec<T>,
    selected: Option<T>,
    on_select: Rc<dyn Fn(T) -> Message + 'a>,
    placeholder: Option<String>,
    width: Length,
    size: Size,
    disabled: bool,
    invalid: bool,
}

/// Creates a select from a list of options.
pub fn native_select<'a, T, L, V, Message: Clone + 'a>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> NativeSelect<'a, T, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    NativeSelect {
        options: options.borrow().to_vec(),
        selected: selected.map(|selected| selected.borrow().clone()),
        on_select: Rc::new(on_select),
        placeholder: None,
        width: Length::Shrink,
        size: Size::Default,
        disabled: false,
        invalid: false,
    }
}

impl<'a, T, Message> NativeSelect<'a, T, Message> {
    /// Sets the text shown when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the width. Defaults to the width of the longest option.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Disables the select (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select as invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-3`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

/// Makes a select small (`size="sm"`: `h-7 py-0.5
/// rounded-[min(var(--radius-md),10px)]`).
pub fn small<T, Message>(select: NativeSelect<'_, T, Message>) -> NativeSelect<'_, T, Message> {
    select.size(Size::Sm)
}

/// Marks a select as invalid (`aria-invalid:border-destructive`).
pub fn invalid<T, Message>(select: NativeSelect<'_, T, Message>) -> NativeSelect<'_, T, Message> {
    select.invalid(true)
}

/// The radius of a select of the given size.
fn radius(theme: &Theme, size: Size) -> f32 {
    let t = tokens(theme);

    match size {
        Size::Default => t.radius_lg(),
        Size::Sm => t.radius_md().min(10.0),
    }
}

/// The style of the select field: `rounded-lg border border-input
/// bg-transparent dark:bg-input/30 dark:hover:bg-input/50`.
pub fn style(theme: &Theme, status: Status) -> Style {
    field_style(theme, status, Size::Default, false, false)
}

fn field_style(theme: &Theme, status: Status, size: Size, invalid: bool, disabled: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    // `opacity-50` on the whole select.
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let background = match (dark, status) {
        (false, _) => Color::TRANSPARENT,
        (true, Status::Hovered | Status::Opened { .. }) if !disabled => alpha(p.input, 0.5),
        (true, _) => alpha(p.input, 0.3),
    };

    let border = match (invalid, status) {
        // `aria-invalid:border-destructive dark:aria-invalid:border-destructive/50`
        (true, _) if dark => alpha(p.destructive, 0.5),
        (true, _) => p.destructive,
        (false, Status::Opened { .. }) => p.ring,
        (false, _) => p.input,
    };

    Style {
        text_color: alpha(p.foreground, opacity),
        // A native select shows its placeholder option like any other.
        placeholder_color: alpha(p.foreground, opacity),
        handle_color: alpha(p.muted_foreground, opacity),
        background: Background::Color(alpha(background, opacity)),
        border: Border {
            color: alpha(border, opacity),
            width: 1.0,
            radius: border::radius(radius(theme, size)),
        },
    }
}

/// The style of the dropdown menu of a select.
pub fn menu_style(theme: &Theme) -> menu::Style {
    let t = tokens(theme);
    let p = &t.palette;

    menu::Style {
        background: Background::Color(p.popover),
        border: style::border(alpha(p.foreground, 0.1), t.radius_lg()),
        text_color: p.popover_foreground,
        selected_text_color: p.accent_foreground,
        selected_background: Background::Color(p.accent),
        shadow: style::shadow_md(),
    }
}

impl<'a, T, Message> From<NativeSelect<'a, T, Message>> for Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    fn from(select: NativeSelect<'a, T, Message>) -> Self {
        let NativeSelect {
            options,
            selected,
            on_select,
            placeholder,
            width,
            size,
            disabled,
            invalid,
        } = select;

        let pick = {
            let on_select = on_select.clone();

            PickList::new(options.clone(), selected.clone(), move |option| {
                on_select(option)
            })
        };

        let pick = match placeholder {
            Some(placeholder) => pick.placeholder(placeholder),
            None => pick,
        };

        let pick = pick
            .width(width)
            .text_size(style::text::SM)
            .text_line_height(text::LineHeight::Absolute(Pixels(20.0)))
            // `h-8 py-1` (`h-7 py-0.5`), `pl-2.5` and `pr-8` inside the
            // border, the chevron at `right-2.5`.
            .padding(Padding {
                top: match size {
                    Size::Default => 6.0,
                    Size::Sm => 4.0,
                },
                bottom: match size {
                    Size::Default => 6.0,
                    Size::Sm => 4.0,
                },
                left: 11.0,
                right: 10.0,
            })
            .font(font::sans())
            .handle(Handle::Static(Icon {
                font: font::ICONS,
                code_point: lucide::CHEVRON_DOWN.codepoint(),
                size: Some(Pixels(16.0)),
                line_height: text::LineHeight::Relative(1.0),
                shaping: text::Shaping::Basic,
            }))
            .style(move |theme, status| field_style(theme, status, size, invalid, disabled))
            .menu_style(menu_style);

        Element::new(Field {
            pick: pick.into(),
            options,
            selected,
            on_select,
            size,
            disabled,
            invalid,
        })
    }
}

/// A [`PickList`] that can be focused like a native `<select>`.
struct Field<'a, T, Message> {
    pick: Element<'a, Message>,
    options: Vec<T>,
    selected: Option<T>,
    on_select: Rc<dyn Fn(T) -> Message + 'a>,
    size: Size,
    disabled: bool,
    invalid: bool,
}

#[derive(Debug, Default)]
struct State {
    focused: bool,
    /// Focused with the keyboard (`focus-visible`).
    visible: bool,
    typeahead: listbox::Typeahead,
    /// The option picked with the keyboard while the list is open: once it
    /// is selected, the list opens again to highlight it.
    picked: Option<usize>,
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.visible = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.visible = false;
    }
}

impl<T, Message> Field<'_, T, Message>
where
    T: ToString + PartialEq + Clone,
    Message: Clone,
{
    /// Whether the list of the [`PickList`] is open.
    fn is_open(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
    ) -> bool {
        self.pick
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                Vector::ZERO,
            )
            .is_some()
    }

    /// Opens or closes the list, by clicking the [`PickList`] where the
    /// pointer would, as it has no other way to be toggled.
    fn toggle(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        viewport: &Rectangle,
        open: bool,
    ) {
        if self.is_open(tree, layout, renderer, viewport) == open {
            return;
        }

        // The click is for the pick list alone: its own shell.
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);

        self.pick.as_widget_mut().update(
            &mut tree.children[0],
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            mouse::Cursor::Available(layout.bounds().center()),
            renderer,
            clipboard,
            &mut shell,
            viewport,
        );
    }

    /// Adds `text` to the search typed ahead, and returns the option it
    /// leads to.
    fn type_ahead(
        &self,
        tree: &mut Tree,
        text: &str,
        now: Instant,
        current: Option<usize>,
    ) -> Option<usize> {
        let labels: Vec<_> = self.options.iter().map(ToString::to_string).collect();
        let search = tree.state.downcast_mut::<State>().typeahead.push(text, now);

        listbox::next_match(
            search,
            labels
                .iter()
                .enumerate()
                .map(|(index, label)| (index, label.as_str())),
            current,
        )
    }

    fn forward(
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
        self.pick.as_widget_mut().update(
            &mut tree.children[0],
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

impl<'a, T, Message> Widget<Message, Theme, iced::Renderer> for Field<'a, T, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.pick)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.pick));
    }

    fn size(&self) -> Dimensions<Length> {
        self.pick.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.pick
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        if !self.disabled {
            operation.focusable(None, layout.bounds(), tree.state.downcast_mut::<State>());
        }
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
        // `disabled:pointer-events-none`
        if self.disabled {
            return;
        }

        let current = self
            .selected
            .as_ref()
            .and_then(|selected| self.options.iter().position(|option| option == selected));
        let is_open = self.is_open(tree, layout, renderer, viewport);

        // The option picked while the list is open is now selected: the
        // list opens again, highlighting it.
        if is_open
            && let Some(picked) = tree.state.downcast_ref::<State>().picked
            && Some(picked) == current
        {
            tree.state.downcast_mut::<State>().picked = None;
            self.toggle(tree, layout, renderer, clipboard, viewport, false);
            self.toggle(tree, layout, renderer, clipboard, viewport, true);
            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = tree.state.downcast_mut::<State>();
                let focused = cursor.is_over(layout.bounds());

                if focused != state.focused || state.visible {
                    state.focused = focused;
                    state.visible = false;
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if tree.state.downcast_ref::<State>().focused =>
            {
                let now = Instant::now();
                let last = self.options.len().checked_sub(1);
                let plain = !modifiers.control() && !modifiers.alt() && !modifiers.logo();
                let typing = tree.state.downcast_ref::<State>().typeahead.is_typing(now);

                let next = match key.as_ref() {
                    // `Alt+ArrowDown`, `F4`, `Enter` and `Space` (unless it
                    // is part of a search) toggle the list.
                    keyboard::Key::Named(key::Named::ArrowDown | key::Named::ArrowUp)
                        if modifiers.alt() =>
                    {
                        self.toggle(tree, layout, renderer, clipboard, viewport, !is_open);
                        None
                    }
                    keyboard::Key::Named(key::Named::F4 | key::Named::Enter) => {
                        self.toggle(tree, layout, renderer, clipboard, viewport, !is_open);
                        None
                    }
                    keyboard::Key::Named(key::Named::Space) if !typing => {
                        self.toggle(tree, layout, renderer, clipboard, viewport, !is_open);
                        None
                    }
                    keyboard::Key::Named(key::Named::Escape) if is_open => {
                        self.toggle(tree, layout, renderer, clipboard, viewport, false);
                        None
                    }
                    // Like a closed native `<select>`: the next or previous
                    // option, without wrapping around.
                    keyboard::Key::Named(named @ (key::Named::ArrowDown | key::Named::ArrowUp))
                        if plain =>
                    {
                        match (current, named == key::Named::ArrowDown) {
                            (None, true) => last.map(|_| 0),
                            (None, false) => None,
                            (Some(index), true) => last.map(|last| (index + 1).min(last)),
                            (Some(index), false) => Some(index.saturating_sub(1)),
                        }
                    }
                    keyboard::Key::Named(key::Named::Home) if plain => last.map(|_| 0),
                    keyboard::Key::Named(key::Named::End) if plain => last,
                    keyboard::Key::Named(key::Named::Space) => {
                        self.type_ahead(tree, " ", now, current)
                    }
                    keyboard::Key::Character(c) if plain => self.type_ahead(tree, c, now, current),
                    _ => {
                        self.forward(
                            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
                        );
                        return;
                    }
                };

                if next != current
                    && let Some(index) = next
                    && let Some(option) = self.options.get(index)
                {
                    shell.publish((self.on_select)(option.clone()));

                    if is_open {
                        tree.state.downcast_mut::<State>().picked = Some(index);
                    }
                }

                shell.request_redraw();
                shell.capture_event();
                return;
            }
            _ => {}
        }

        self.forward(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let t = tokens(theme);
        let radius = radius(theme, self.size);
        let focus_visible = state.focused && state.visible;

        // `focus-visible:ring-3 focus-visible:ring-ring/50`, or
        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`.
        let ring = if self.invalid {
            Some(style::invalid_outset_ring(&t, radius))
        } else if focus_visible {
            Some(style::ring(&t, radius))
        } else {
            None
        };

        if let Some(ring) = ring {
            let opacity = if self.disabled {
                style::DISABLED_OPACITY
            } else {
                1.0
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds.expand(ring.width),
                    border: Border {
                        color: alpha(ring.color, opacity),
                        ..ring.border()
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        self.pick.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        // `focus-visible:border-ring`
        if focus_visible && !self.invalid {
            renderer.with_layer(*viewport, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style::border(t.palette.ring, radius),
                        ..renderer::Quad::default()
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            });
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
        if self.disabled {
            return if cursor.is_over(layout.bounds()) {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::None
            };
        }

        self.pick.as_widget().mouse_interaction(
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.pick.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
