//! A set of layered sections of content, known as tab panels, that are
//! displayed one at a time.
//!
//! Port of <https://ui.shadcn.com/docs/components/tabs>.
//!
//! ```no_run
//! # use shadcn_iced::tabs::tabs;
//! # #[derive(Clone, Copy, PartialEq)] enum Tab { Account, Password }
//! # #[derive(Clone)] enum Message { Tab(Tab) }
//! # fn view<'a>(tab: Tab) -> iced::Element<'a, Message> {
//! tabs(tab, Message::Tab)
//!     .tab(Tab::Account, "Account")
//!     .tab(Tab::Password, "Password")
//!     .content(iced::widget::text("..."))
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::roving::Keys;
use shadcn_iced_core::primitive::{Pressable, Roving};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Border, Color, Length, Theme, border};

/// The visual style of the tab list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Triggers inside a muted pill (`bg-muted`).
    #[default]
    Default,
    /// Transparent triggers with an underline under the active one.
    Line,
}

/// The direction of the tab list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// The list above the content.
    #[default]
    Horizontal,
    /// The list on the left of the content, triggers stacked.
    Vertical,
}

/// Tabs. See the [module documentation](self).
pub struct Tabs<'a, T, Message> {
    selected: T,
    tabs: Vec<(T, Option<Icon>, String, bool)>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    content: Option<Element<'a, Message>>,
    variant: Variant,
    orientation: Orientation,
    width: Length,
}

/// Creates [`Tabs`] with the given selected value.
pub fn tabs<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a>(
    selected: T,
    on_select: impl Fn(T) -> Message + 'a,
) -> Tabs<'a, T, Message> {
    Tabs {
        selected,
        tabs: Vec::new(),
        on_select: Box::new(on_select),
        content: None,
        variant: Variant::Default,
        orientation: Orientation::Horizontal,
        width: Length::Shrink,
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> Tabs<'a, T, Message> {
    /// Appends a tab.
    pub fn tab(mut self, value: T, label: impl Into<String>) -> Self {
        self.tabs.push((value, None, label.into(), false));
        self
    }

    /// Appends a tab with an icon.
    pub fn tab_with_icon(mut self, value: T, icon: Icon, label: impl Into<String>) -> Self {
        self.tabs.push((value, Some(icon), label.into(), false));
        self
    }

    /// Appends a disabled tab.
    pub fn disabled_tab(mut self, value: T, label: impl Into<String>) -> Self {
        self.tabs.push((value, None, label.into(), true));
        self
    }

    /// Sets the content of the selected tab, displayed below (or beside)
    /// the list.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the width of the tab list. With [`Length::Fill`], triggers share
    /// the width equally.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

/// `p-[3px]`
const LIST_PADDING: f32 = 3.0;
/// `h-8`
const LIST_HEIGHT: f32 = 32.0;
/// `h-[calc(100%-1px)]`
const TRIGGER_HEIGHT: f32 = LIST_HEIGHT - 2.0 * LIST_PADDING - 1.0;

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> From<Tabs<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(tabs: Tabs<'a, T, Message>) -> Self {
        let variant = tabs.variant;
        let vertical = tabs.orientation == Orientation::Vertical;
        let fill = tabs.width != Length::Shrink;

        // Vertical triggers are as wide as the widest one (`w-full` in a
        // `w-fit` list): each one also lays out the others, invisibly.
        let labels: Vec<(Option<Icon>, String)> = if vertical {
            tabs.tabs
                .iter()
                .map(|(_, icon, label, _)| (*icon, label.clone()))
                .collect()
        } else {
            Vec::new()
        };

        let triggers = tabs.tabs.into_iter().map(|(value, icon, label, disabled)| {
            let active = value == tabs.selected;

            // `justify-center`, or `justify-start` when vertical.
            let content: Element<'a, Message> = if vertical {
                let hidden = labels.iter().map(|(icon, label)| {
                    Element::new(Ghost {
                        content: trigger_label(*icon, label.clone(), false),
                    })
                });

                // The labels overlap (`spacing(-20)`); the visible one
                // comes last.
                container(
                    Column::with_children(hidden)
                        .push(trigger_label(icon, label, true))
                        .spacing(-20.0),
                )
                .center_y(Length::Fill)
                .into()
            } else {
                let content = trigger_label(icon, label, true);

                container(content).center(Length::Fill).into()
            };

            // Roving focus: the active tab is the one in the tab order.
            let trigger = Pressable::new(content)
                .on_press_maybe((!disabled && !active).then(|| (tabs.on_select)(value)))
                .disabled(disabled)
                .focusable(true)
                .tab_stop(active)
                .width(if fill { Length::Fill } else { Length::Shrink })
                .height(if vertical { 26.0 } else { TRIGGER_HEIGHT })
                // `px-1.5 py-0.5` inside a `border`
                .padding([3, 7])
                .focus_style(move |theme, status, focused| {
                    let mut style = trigger(theme, status, active, variant);

                    // `focus-visible:border-ring`
                    if focused {
                        style.border.color = tokens(theme).palette.ring;
                    }

                    style
                })
                // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
                .ring(|theme, _status, focused| {
                    focused.then(|| {
                        let t = tokens(theme);

                        style::ring(&t, t.radius_md())
                    })
                });

            if variant == Variant::Line {
                Element::new(ActiveLine {
                    content: trigger.into(),
                    active,
                    vertical,
                })
            } else {
                Element::from(trigger)
            }
        });

        // `gap-1` for the line variant.
        let spacing = match variant {
            Variant::Default => 0,
            Variant::Line => 4,
        };

        // The arrow keys move the focus between triggers; Enter or Space
        // activates the focused one (`activateOnFocus={false}`).
        let list: Element<'a, Message> = if vertical {
            container(
                Column::with_children(triggers)
                    .spacing(spacing)
                    .width(Length::Shrink),
            )
            .padding(LIST_PADDING)
            .width(if fill { tabs.width } else { Length::Shrink })
            .style(move |theme| self::list(theme, variant))
            .into()
        } else {
            container(
                Row::with_children(triggers)
                    .spacing(spacing)
                    .height(Length::Fill)
                    .align_y(Alignment::Center),
            )
            .padding(iced::Padding {
                // Center the `calc(100%-1px)` triggers.
                top: LIST_PADDING + 0.5,
                bottom: LIST_PADDING + 0.5,
                left: LIST_PADDING,
                right: LIST_PADDING,
            })
            .height(LIST_HEIGHT)
            .width(tabs.width)
            .style(move |theme| self::list(theme, variant))
            .into()
        };

        let list = Roving::new(list).keys(if vertical {
            Keys::Vertical
        } else {
            Keys::Horizontal
        });

        let has_content = tabs.content.is_some();

        // `gap-2`
        if vertical {
            Row::new()
                .push(list)
                .push(tabs.content)
                .spacing(8)
                .width(if has_content {
                    Length::Fill
                } else {
                    Length::Shrink
                })
                .into()
        } else {
            Column::new()
                .push(list)
                .push(tabs.content)
                .spacing(8)
                .width(if fill || has_content {
                    Length::Fill
                } else {
                    Length::Shrink
                })
                .into()
        }
    }
}

/// The icon and label of a trigger: `gap-1.5 text-sm font-medium`.
fn trigger_label<'a, Message: 'a>(
    icon: Option<Icon>,
    label: String,
    visible: bool,
) -> Element<'a, Message> {
    let hidden = move |_: &Theme| text::Style {
        color: Some(Color::TRANSPARENT),
    };

    Row::new()
        .push(icon.map(|icon| {
            let icon = icon.sized(16);

            if visible { icon } else { icon.style(hidden) }
        }))
        .push({
            let label = text(label)
                .size(style::text::SM)
                .font(font::medium())
                .line_height(iced::Pixels(20.0))
                .wrapping(text::Wrapping::None);

            if visible { label } else { label.style(hidden) }
        })
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
}

/// The style of the tab list.
pub fn list(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: match variant {
            Variant::Default => Some(Background::Color(t.palette.muted)),
            Variant::Line => None,
        },
        text_color: Some(t.palette.muted_foreground),
        // `rounded-lg`, `data-[variant=line]:rounded-none`
        border: match variant {
            Variant::Default => border::rounded(t.radius_lg()),
            Variant::Line => border::rounded(0),
        },
        ..container::Style::default()
    }
}

/// The style of a tab trigger.
pub fn trigger(theme: &Theme, status: Status, active: bool, variant: Variant) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    // `text-foreground/60 dark:text-muted-foreground hover:text-foreground`
    let text_color = if active || hovered {
        p.foreground
    } else if dark {
        p.muted_foreground
    } else {
        alpha(p.foreground, 0.6)
    };

    let mut style = match variant {
        // `data-active:bg-background data-active:shadow-sm`,
        // `dark:data-active:border-input dark:data-active:bg-input/30`
        Variant::Default => Style {
            background: active.then(|| {
                Background::Color(if dark {
                    alpha(p.input, 0.3)
                } else {
                    p.background
                })
            }),
            text_color,
            border: Border {
                color: if active && dark {
                    p.input
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                // `rounded-md`
                radius: border::radius(t.radius_md()),
            },
            shadow: if active {
                style::shadow_sm()
            } else {
                iced::Shadow::default()
            },
            ..Style::default()
        },
        Variant::Line => Style {
            background: None,
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 1.0,
                radius: border::radius(t.radius_md()),
            },
            ..Style::default()
        },
    };

    // `disabled:opacity-50`
    if status == Status::Disabled {
        style.text_color = alpha(style.text_color, style::DISABLED_OPACITY);
    }

    style
}

/// Takes the space of its content without showing it, nor exposing it to
/// operations (`aria-hidden`): the hidden labels that size vertical
/// triggers are not texts to find.
struct Ghost<'a, Message> {
    content: Element<'a, Message>,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Ghost<'_, Message> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
    }
}

/// Draws the line of the active trigger of the line variant, overflowing
/// the trigger like its `::after`: `after:h-0.5 after:bottom-[-5px]`, or
/// `after:w-0.5 after:-right-1` when vertical.
struct ActiveLine<'a, Message> {
    content: Element<'a, Message>,
    active: bool,
    vertical: bool,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for ActiveLine<'_, Message> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
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
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);

        if !self.active {
            return;
        }

        let bounds = layout.bounds();

        let line = if self.vertical {
            iced::Rectangle {
                x: bounds.x + bounds.width + 2.0,
                width: 2.0,
                ..bounds
            }
        } else {
            iced::Rectangle {
                y: bounds.y + bounds.height + 3.0,
                height: 2.0,
                ..bounds
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: line,
                ..renderer::Quad::default()
            },
            Background::Color(tokens(theme).palette.foreground),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
}
