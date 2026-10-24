//! Displays a list of options for the user to pick from—triggered by a
//! button.
//!
//! Port of <https://ui.shadcn.com/docs/components/select>.
//!
//! Like a Radix select, the trigger takes the keyboard focus, and typing
//! while it is focused selects the option starting with what was typed.
//! In the open list, <kbd>Enter</kbd> and <kbd>Space</kbd> select the
//! highlighted option and typing highlights the matching one; a finger
//! selects an option with a tap, and scrolls the list with a drag.
//!
//! ```no_run
//! # use shadcn_iced::select::select;
//! # #[derive(Clone)] enum Message { Fruit(&'static str) }
//! # fn view<'a>(fruit: Option<&'static str>) -> iced::Element<'a, Message> {
//! select(fruit, Message::Fruit)
//!     .placeholder("Select a fruit")
//!     .label("Fruits")
//!     .option("apple", "Apple")
//!     .option("banana", "Banana")
//!     .into()
//! # }
//! ```
use super::menu::{self, menu_tokens};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::{Align, Anchored, Pressable, Side, focus, listbox};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay, renderer};
use iced::keyboard::{self, key};
use iced::time::Instant;
use iced::widget::button::{Status, Style};
use iced::widget::{Row, container, scrollable, text};
use iced::{
    Alignment, Background, Border, Color, Event, Length, Rectangle, Theme, Vector, border, mouse,
};

enum Entry<T> {
    Option(T, String, bool),
    Label(String),
    Separator,
}

impl<T> Entry<T> {
    /// The height of the entry in the list.
    fn height(&self) -> f32 {
        match self {
            // `py-1` + `text-sm`
            Entry::Option(..) => 28.0,
            // `py-1` + `text-xs`
            Entry::Label(_) => 24.0,
            // `my-1 h-px` between two `p-1` groups.
            Entry::Separator => 17.0,
        }
    }
}

/// The size of the trigger of a [`Select`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-7`
    Sm,
    /// `h-8`
    #[default]
    Default,
}

impl Size {
    fn height(self) -> f32 {
        match self {
            Size::Sm => 28.0,
            Size::Default => 32.0,
        }
    }

    /// `rounded-lg`, or `rounded-[min(var(--radius-md),10px)]`.
    fn radius(self, tokens: &shadcn_iced_core::Tokens) -> f32 {
        match self {
            Size::Default => tokens.radius_lg(),
            Size::Sm => tokens.radius_md().min(10.0),
        }
    }
}

/// A select. See the [module documentation](self).
pub struct Select<'a, T, Message> {
    selected: Option<T>,
    entries: Vec<Entry<T>>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    placeholder: String,
    width: Length,
    size: Size,
    disabled: bool,
    invalid: bool,
    align_item_with_trigger: bool,
}

/// Creates a [`Select`] with the given selected value.
pub fn select<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a>(
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> Select<'a, T, Message> {
    Select {
        selected,
        entries: Vec::new(),
        on_select: Box::new(on_select),
        placeholder: String::new(),
        width: Length::Shrink,
        size: Size::Default,
        disabled: false,
        invalid: false,
        align_item_with_trigger: true,
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> Select<'a, T, Message> {
    /// Appends an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Option(value, label.into(), false));
        self
    }

    /// Appends every option of an iterator, using their [`ToString`] as label.
    pub fn options(mut self, values: impl IntoIterator<Item = T>) -> Self
    where
        T: ToString,
    {
        for value in values {
            let label = value.to_string();
            self.entries.push(Entry::Option(value, label, false));
        }
        self
    }

    /// Appends a disabled option.
    pub fn disabled_option(mut self, value: T, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Option(value, label.into(), true));
        self
    }

    /// Appends a group label (`SelectLabel`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into()));
        self
    }

    /// Appends a separator.
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// Sets the text shown when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the width of the trigger.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the size of the trigger.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Disables the select.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets whether the list opens over the trigger, with the selected item
    /// (or the first one) on top of the trigger (`alignItemWithTrigger`,
    /// the default). Otherwise it opens below the trigger.
    pub fn align_item_with_trigger(mut self, align: bool) -> Self {
        self.align_item_with_trigger = align;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> From<Select<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(select: Select<'a, T, Message>) -> Self {
        let selected_label = select.entries.iter().find_map(|entry| match entry {
            Entry::Option(value, label, _) if Some(value) == select.selected.as_ref() => {
                Some(label.clone())
            }
            _ => None,
        });

        let has_value = selected_label.is_some();
        let invalid = select.invalid;
        let size = select.size;
        // `disabled:opacity-50`
        let opacity = if select.disabled {
            style::DISABLED_OPACITY
        } else {
            1.0
        };

        // `gap-1.5 py-2 pr-2 pl-2.5 text-sm`
        let trigger = Pressable::new(
            Row::new()
                .push(
                    text(selected_label.unwrap_or(select.placeholder))
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .wrapping(text::Wrapping::None)
                        .width(if select.width == Length::Shrink {
                            Length::Shrink
                        } else {
                            Length::Fill
                        })
                        .style(move |theme: &Theme| text::Style {
                            color: (!has_value)
                                .then(|| alpha(tokens(theme).palette.muted_foreground, opacity)),
                        }),
                )
                .push(
                    lucide::CHEVRON_DOWN
                        .sized(16)
                        .style(move |theme: &Theme| text::Style {
                            color: Some(alpha(tokens(theme).palette.muted_foreground, opacity)),
                        }),
                )
                .spacing(6)
                .align_y(Alignment::Center)
                .height(Length::Fill),
        )
        .width(select.width)
        .height(size.height())
        // `pr-2 pl-2.5` inside the 1px border.
        .padding(iced::Padding {
            top: 0.0,
            right: 9.0,
            bottom: 0.0,
            left: 11.0,
        })
        .disabled(select.disabled)
        // Like the `<button>` of a Radix select: Tab focuses it, and a click
        // focuses it without showing the ring.
        .focusable(true)
        // `focus-visible:border-ring`, unless `aria-invalid:border-destructive`.
        .focus_style(move |theme, status, focused| {
            let mut style = trigger_style(theme, status, invalid, size);

            if focused && !invalid {
                style.border.color = tokens(theme).palette.ring;
            }

            style
        })
        // `focus-visible:ring-3 focus-visible:ring-ring/50`, or
        // `aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40`.
        .ring(move |theme, _status, focused| {
            focused.then(|| {
                let t = tokens(theme);
                let radius = size.radius(&t);

                if invalid {
                    style::invalid_outset_ring(&t, radius)
                } else {
                    style::ring(&t, radius)
                }
            })
        });

        // Where the selected (or first) option sits in the list, to open
        // the list with it on top of the trigger.
        // (`p-1`, inside the ring.)
        let mut item_top = 4.0;
        let mut aligned = None;
        let mut first = None;

        for entry in &select.entries {
            if let Entry::Option(value, ..) = entry {
                first.get_or_insert(item_top);

                if select.selected.as_ref() == Some(value) {
                    aligned = Some(item_top);
                }
            }

            item_top += entry.height();
        }

        let aligned_top = aligned.or(first).unwrap_or(0.0);

        // The selected option is highlighted when the list opens.
        let selected_index = select.entries.iter().position(|entry| {
            matches!(entry, Entry::Option(value, ..) if select.selected.as_ref() == Some(value))
        });

        // Like the content of a Radix select, the list owns the keyboard:
        // `Space` selects, and typing highlights the matching option.
        let mut list = listbox::Listbox::new(menu::highlight)
            .highlighted(selected_index)
            .typeahead(true);

        // The enabled options, for typing ahead on the closed trigger.
        let mut options = Vec::new();
        let mut current = None;

        for entry in select.entries {
            list = match entry {
                Entry::Option(value, label, disabled) => {
                    let is_selected = select.selected.as_ref() == Some(&value);

                    if !disabled {
                        if is_selected {
                            current = Some(options.len());
                        }

                        options.push((label.clone(), (select.on_select)(value.clone())));
                    }

                    let text_value = label.clone();
                    let label = text(label)
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .wrapping(text::Wrapping::None)
                        .style(move |theme: &Theme| text::Style {
                            color: disabled.then(|| {
                                alpha(
                                    menu_tokens(theme).palette.popover_foreground,
                                    style::DISABLED_OPACITY,
                                )
                            }),
                        });

                    // `absolute right-2 size-4`
                    let check = container(is_selected.then(|| lucide::CHECK.sized(16)))
                        .width(16)
                        .into();

                    // `gap-1.5 py-1 pr-8 pl-1.5`
                    list.item(
                        listbox::row(label, Some(check))
                            .spacing(8.0)
                            .padding(menu::ITEM_PADDING.right(8)),
                        (!disabled).then(|| (select.on_select)(value)),
                    )
                    .text_value(text_value)
                }
                Entry::Label(label) => list.push(
                    // `px-1.5 py-1 text-xs text-muted-foreground`
                    container(
                        text(label)
                            .size(style::text::XS)
                            .line_height(iced::Pixels(16.0))
                            .style(|theme: &Theme| text::Style {
                                color: Some(menu_tokens(theme).palette.muted_foreground),
                            }),
                    )
                    .padding(menu::ITEM_PADDING),
                ),
                // A separator ends a `SelectGroup` (`p-1`) and starts another.
                Entry::Separator => list.push(container(menu::separator()).padding([4, 0])),
            };
        }

        // `w-(--anchor-width) min-w-36 rounded-lg shadow-md ring-1`, `p-1`
        // groups.
        let content = menu::shadow(
            container(listbox::region(
                scrollable(list.padding(4).width(Length::Fill))
                    .width(Length::Fill)
                    .direction(scrollable::Direction::Vertical(
                        super::scroll_area::scrollbar(),
                    ))
                    .style(super::scroll_area::style),
            ))
            .padding(1)
            .max_height(600)
            .style(menu::panel),
            false,
        );

        let anchored = Anchored::new(trigger, content)
            .align(Align::Start)
            .anchor_width(144.0)
            // `ring-1` around the list.
            .outset(1.0)
            .close_on_select(true)
            // `Enter`, `Space` and the arrows open the list from the focused
            // trigger, and `Tab` closes it.
            .menu_keys(true)
            .modal(true)
            // `focus-visible:border-ring focus-visible:ring-3`, once
            // dismissed with the keyboard.
            .focus_ring(move |theme| {
                let t = tokens(theme);
                style::focus_visible(
                    &t,
                    match size {
                        Size::Default => t.radius_lg(),
                        Size::Sm => t.radius_md().min(10.0),
                    },
                )
            });

        let anchored = if select.align_item_with_trigger {
            // The item centered on the trigger, its text aligned with the
            // value, or below the trigger when there is no room for it.
            anchored
                .side(Side::Bottom)
                .offset(-(size.height() + 28.0) / 2.0 - aligned_top)
                .fallback_offset(4.0)
                .animate(false)
        } else {
            anchored.side(Side::Bottom).offset(4.0)
        };

        Element::new(TypeaheadTrigger {
            content: anchored.into(),
            options,
            current,
        })
    }
}

/// The trigger of a select, picking the option whose label starts with the
/// characters typed while it is focused and closed, like a native
/// `<select>` (and the trigger of a Radix select).
struct TypeaheadTrigger<'a, Message> {
    content: Element<'a, Message>,
    /// The label of the enabled options, and the message selecting them.
    options: Vec<(String, Message)>,
    /// The selected option, in `options`.
    current: Option<usize>,
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, iced::Renderer>
    for TypeaheadTrigger<'a, Message>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<listbox::Typeahead>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(listbox::Typeahead::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> iced::Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // The open list handles (and captures) the keys typed into it.
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && !shell.is_event_captured()
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
        {
            let now = Instant::now();
            let typeahead = tree.state.downcast_ref::<listbox::Typeahead>();

            let typed = match key.as_ref() {
                keyboard::Key::Character(c) => Some(c),
                // While typing ahead, `Space` is part of the search rather
                // than opening the list.
                keyboard::Key::Named(key::Named::Space) if typeahead.is_typing(now) => Some(" "),
                _ => None,
            };

            if let Some(typed) = typed
                && focus::is_focus_within(
                    &mut self.content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                )
            {
                let search = tree
                    .state
                    .downcast_mut::<listbox::Typeahead>()
                    .push(typed, now);

                if let Some(index) = listbox::next_match(
                    search,
                    self.options
                        .iter()
                        .enumerate()
                        .map(|(index, (label, _))| (index, label.as_str())),
                    self.current,
                ) {
                    shell.publish(self.options[index].1.clone());
                    // Until the selection comes back in the next view.
                    self.current = Some(index);
                }

                shell.capture_event();
                return;
            }
        }

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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// The style of the trigger of a select (`rounded-lg border border-input
/// bg-transparent dark:bg-input/30 dark:hover:bg-input/50`).
pub fn trigger(theme: &Theme, status: Status, invalid: bool) -> Style {
    trigger_style(theme, status, invalid, Size::Default)
}

fn trigger_style(theme: &Theme, status: Status, invalid: bool, size: Size) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);
    let opacity = if status == Status::Disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    Style {
        background: Some(Background::Color(match (dark, hovered) {
            (false, _) => Color::TRANSPARENT,
            (true, false) => alpha(p.input, 0.3 * opacity),
            (true, true) => alpha(p.input, 0.5 * opacity),
        })),
        text_color: alpha(p.foreground, opacity),
        border: Border {
            color: alpha(
                if invalid {
                    if dark {
                        alpha(p.destructive, 0.5)
                    } else {
                        p.destructive
                    }
                } else {
                    p.input
                },
                opacity,
            ),
            width: 1.0,
            radius: border::radius(match size {
                Size::Default => t.radius_lg(),
                // `rounded-[min(var(--radius-md),10px)]`
                Size::Sm => t.radius_md().min(10.0),
            }),
        },
        ..Style::default()
    }
}
