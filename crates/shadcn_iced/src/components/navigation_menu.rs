//! A collection of links for navigating websites.
//!
//! Port of <https://ui.shadcn.com/docs/components/navigation-menu>.
//!
//! A trigger reveals its panel on hover, or with a tap on a touch screen
//! (a second tap, a tap outside or on another trigger closes it). Its
//! triggers and links take the keyboard focus, like Radix: <kbd>←</kbd> /
//! <kbd>→</kbd> move it between them, <kbd>Enter</kbd>, <kbd>Space</kbd>
//! or <kbd>↓</kbd> open the panel of a trigger and focus its first link,
//! <kbd>Tab</kbd> cycles through the links of the open panel, and
//! <kbd>Escape</kbd> closes it, the focus back on the trigger.
//!
//! ```no_run
//! # use shadcn_iced::navigation_menu::{navigation_menu, link};
//! # #[derive(Clone)] enum Message { Docs, Intro }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! navigation_menu()
//!     .menu("Getting started", link("Introduction", "Re-usable components.", Message::Intro))
//!     .link("Docs", Message::Docs)
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::anchored::TouchOpen;
use shadcn_iced_core::primitive::focus::Ring;
use shadcn_iced_core::primitive::{Align, Anchored, Pressable, Side, roving};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::time::Duration;
use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Padding, Theme, border};

enum Entry<'a, Message> {
    Menu(String, Element<'a, Message>),
    Link(String, Message),
}

/// A navigation menu. See the [module documentation](self).
pub struct NavigationMenu<'a, Message> {
    entries: Vec<Entry<'a, Message>>,
}

/// Creates an empty [`NavigationMenu`].
pub fn navigation_menu<'a, Message: Clone + 'a>() -> NavigationMenu<'a, Message> {
    NavigationMenu {
        entries: Vec::new(),
    }
}

impl<'a, Message: Clone + 'a> NavigationMenu<'a, Message> {
    /// Appends a trigger revealing `content` on hover (or a tap, or the
    /// keyboard).
    pub fn menu(
        mut self,
        label: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        self.entries.push(Entry::Menu(label.into(), content.into()));
        self
    }

    /// Appends a plain link.
    pub fn link(mut self, label: impl Into<String>, on_press: Message) -> Self {
        self.entries.push(Entry::Link(label.into(), on_press));
        self
    }
}

/// The style of a trigger (`navigationMenuTriggerStyle`: `h-9 rounded-lg
/// px-2.5 py-1.5 text-sm font-medium hover:bg-muted`).
pub fn trigger_style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: hovered.then_some(Background::Color(t.palette.muted)),
        text_color: t.palette.foreground,
        border: border::rounded(t.radius_lg()),
        ..Style::default()
    }
}

/// The focus ring of triggers and links (`focus-visible:ring-[3px]
/// focus-visible:ring-ring/50`), around a control of the given radius.
fn focus_ring(
    theme: &Theme,
    focused: bool,
    radius: fn(&shadcn_iced_core::Tokens) -> f32,
) -> Option<Ring> {
    focused.then(|| {
        let t = tokens(theme);
        style::ring(&t, radius(&t))
    })
}

/// The style of a link in the content of a navigation menu
/// (`hover:bg-muted rounded-md p-2`).
fn link_style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: hovered.then_some(Background::Color(t.palette.muted)),
        text_color: t.palette.popover_foreground,
        border: border::rounded(t.radius_md()),
        ..Style::default()
    }
}

/// A link of the content of a navigation menu: a title and a short
/// description (`ListItem` in the shadcn/ui example: `flex-col gap-1
/// text-sm`, the title `leading-none font-medium`, the description
/// `line-clamp-2`).
pub fn link<'a, Message: Clone + 'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    Pressable::new(
        Column::new()
            .push(
                text(title.into())
                    .size(style::text::SM)
                    .font(font::medium())
                    .line_height(1.0),
            )
            .push(
                // `line-clamp-2`, without the ellipsis iced cannot draw.
                container(
                    text(description.into())
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }),
                )
                .max_height(40)
                .clip(true),
            )
            .spacing(4),
    )
    .on_press(on_press)
    .padding(8)
    .width(Length::Fill)
    .style(link_style)
    .focusable(true)
    .ring(|theme, _, focused| focus_ring(theme, focused, |t| t.radius_md()))
    .into()
}

/// A link with an icon (`NavigationMenuLink` with `flex-row items-center
/// gap-2`).
pub fn icon_link<'a, Message: Clone + 'a>(
    icon: Icon,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    Pressable::new(
        Row::new()
            .push(icon.sized(16))
            .push(
                text(label.into())
                    .size(style::text::SM)
                    .line_height(iced::Pixels(20.0)),
            )
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .on_press(on_press)
    .padding(8)
    .width(Length::Fill)
    .style(link_style)
    .focusable(true)
    .ring(|theme, _, focused| focus_ring(theme, focused, |t| t.radius_md()))
    .into()
}

/// The style of the popup (`rounded-lg bg-popover shadow ring-1
/// ring-foreground/10`).
pub fn popup_style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        border: style::ring_border(&t, t.radius_lg()),
        shadow: style::shadow_sm(),
        ..container::Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<NavigationMenu<'a, Message>> for Element<'a, Message> {
    fn from(menu: NavigationMenu<'a, Message>) -> Self {
        let label = |label: String| {
            text(label)
                .size(style::text::SM)
                .font(font::medium())
                .line_height(iced::Pixels(20.0))
        };

        let items = menu.entries.into_iter().map(|entry| match entry {
            Entry::Menu(title, content) => {
                // `ml-1 size-3 top-px`
                let trigger = Pressable::new(
                    Row::new()
                        .push(label(title))
                        .push(
                            container(lucide::CHEVRON_DOWN.sized(12))
                                .padding(Padding::ZERO.top(1).left(4)),
                        )
                        .align_y(Alignment::Center),
                )
                .padding([8, 10])
                .height(36)
                .style(trigger_style)
                .focusable(true)
                .ring(|theme, _, focused| focus_ring(theme, focused, |t| t.radius_lg()));

                // `p-1` inside the 1px ring.
                let panel = container(content).padding(5).style(popup_style);

                Anchored::new(trigger, panel)
                    .hover(Duration::from_millis(50), Duration::from_millis(50))
                    // A tap on the trigger toggles the panel, like a click
                    // in Radix.
                    .touch_open(TouchOpen::Tap)
                    .side(Side::Bottom)
                    .align(Align::Start)
                    .offset(8.0)
                    // `ring-1` around the popup.
                    .outset(1.0)
                    // `data-popup-open:bg-muted/50`
                    .open_highlight(|theme| {
                        let t = tokens(theme);
                        (
                            Background::Color(alpha(t.palette.muted, 0.5)),
                            border::rounded(t.radius_lg()),
                        )
                    })
                    .into()
            }
            Entry::Link(title, on_press) => Pressable::new(label(title))
                .on_press(on_press)
                .padding([8, 10])
                .height(36)
                .style(trigger_style)
                .focusable(true)
                .ring(|theme, _, focused| focus_ring(theme, focused, |t| t.radius_lg()))
                .into(),
        });

        // `gap-0`; the arrow keys move the focus between the items.
        roving(Row::with_children(items).align_y(Alignment::Center)).into()
    }
}
