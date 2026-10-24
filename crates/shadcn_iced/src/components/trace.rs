//! A compact log of the steps of an agent: tool calls, edits, searches and
//! reasoning, optionally expandable.
//!
//! Port of <https://ui.shadcn.com/docs/components/trace>.
//!
//! ```no_run
//! # use shadcn_iced::trace::{trace, trace_line, Status};
//! # use shadcn_iced_core::icon::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! trace([
//!     trace_line("Ran pnpm install").icon(lucide::CIRCLE_CHECK).into(),
//!     trace_line("Failed to run pnpm lint")
//!         .icon(lucide::CIRCLE_ALERT)
//!         .status(Status::Error)
//!         .into(),
//! ])
//! .into()
//! # }
//! ```
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::typeset::{TABULAR_NUMS, typeset};
use shadcn_iced_core::primitive::{Pressable, disclosure};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::{Status as ButtonStatus, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Length, Padding, Theme};

/// The status of a [`TraceLine`] (`data-status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Status {
    /// A finished step.
    #[default]
    Done,
    /// A step in progress (`running`).
    Running,
    /// A failed step: `text-destructive`.
    Error,
    /// A step not started yet: `opacity-60`.
    Pending,
}

/// A step of a trace (`TraceLine`): an icon, a title and a description.
#[derive(Debug, Clone)]
pub struct TraceLine<Message> {
    icon: Option<Icon>,
    trailing: Option<Icon>,
    title: String,
    description: Option<String>,
    mono_description: bool,
    status: Status,
    on_press: Option<Message>,
}

/// Creates a [`TraceLine`] with a title (`TraceTitle`).
pub fn trace_line<Message>(title: impl Into<String>) -> TraceLine<Message> {
    TraceLine {
        icon: None,
        trailing: None,
        title: title.into(),
        description: None,
        mono_description: false,
        status: Status::Done,
        on_press: None,
    }
}

impl<Message: Clone> TraceLine<Message> {
    /// Sets the icon (`TraceIcon`, `size-4`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows an icon right after the title, e.g. a chevron or an external
    /// link arrow (`[[data-slot=trace-title]~&]:-ml-1`).
    pub fn trailing_icon(mut self, icon: Icon) -> Self {
        self.trailing = Some(icon);
        self
    }

    /// Sets the description shown after the title (`TraceDescription`,
    /// `text-muted-foreground/70 tabular-nums`), e.g. a duration.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Writes the description in the monospace font (`font-mono`), e.g.
    /// for a diff stat.
    pub fn mono_description(mut self, mono: bool) -> Self {
        self.mono_description = mono;
        self
    }

    /// Sets the status.
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Makes the line a link or a button, highlighted on hover
    /// (`in-[a:hover,button:hover]:text-foreground`).
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Makes the line expandable (`TraceCollapsible`): pressing it shows
    /// `content` below, with a guide line on the left.
    pub fn collapsible<'a>(
        self,
        content: impl Into<Element<'a, Message>>,
    ) -> Collapsible<'a, Message> {
        Collapsible {
            line: self,
            content: content.into(),
            open: None,
            default_open: false,
            on_toggle: None,
        }
    }
}

/// The color of a line in the given state.
fn color(theme: &Theme, status: Status, hovered: bool) -> Color {
    let p = tokens(theme).palette;

    match status {
        Status::Error => p.destructive,
        Status::Pending => alpha(p.muted_foreground, 0.6),
        _ if hovered => p.foreground,
        _ => p.muted_foreground,
    }
}

fn view<'a, Message: Clone + 'a>(line: &TraceLine<Message>) -> Row<'a, Message> {
    let status = line.status;
    let mono = line.mono_description;

    // `gap-2 text-sm`
    Row::new()
        .push(line.icon.map(|icon| icon.sized(16)))
        .push(
            Row::new()
                .push(
                    text(line.title.clone())
                        .size(style::text::SM)
                        .line_height(style::text::leading(style::text::SM))
                        .wrapping(text::Wrapping::None),
                )
                .push(line.trailing.map(|icon| icon.sized(16)))
                .spacing(4)
                .align_y(Alignment::Center),
        )
        .push(line.description.clone().map(|description| {
            typeset(description)
                .size(style::text::SM)
                .line_height(style::text::leading(style::text::SM))
                .font(if mono { font::mono() } else { font::sans() })
                .feature(TABULAR_NUMS)
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(
                        tokens(theme).palette.muted_foreground,
                        if status == Status::Pending { 0.42 } else { 0.7 },
                    )),
                })
        }))
        .spacing(8)
        .align_y(Alignment::Center)
}

/// Wraps a line so that it reacts to hover, pressable or not.
fn surface<'a, Message: Clone + 'a>(
    content: Row<'a, Message>,
    status: Status,
    on_press: Option<Message>,
    interactive: bool,
) -> Element<'a, Message> {
    let pressable =
        Pressable::new(content)
            .on_press_maybe(on_press)
            .style(move |theme, button_status| {
                let hovered = interactive
                    && matches!(button_status, ButtonStatus::Hovered | ButtonStatus::Pressed);

                Style {
                    background: None,
                    text_color: color(theme, status, hovered),
                    border: iced::border::rounded(tokens(theme).radius_md()),
                    ..Style::default()
                }
            });

    // A pressable line (or the trigger of a collapsible one) is a
    // `<button>`: `rounded-md focus-visible:ring-3 focus-visible:ring-ring/50`.
    if interactive {
        super::button::focus_ring(pressable, |t| t.radius_md()).into()
    } else {
        pressable.into()
    }
}

impl<'a, Message: Clone + 'a> From<TraceLine<Message>> for Element<'a, Message> {
    fn from(line: TraceLine<Message>) -> Self {
        let interactive = line.on_press.is_some();
        let status = line.status;
        let on_press = line.on_press.clone();

        surface(view(&line), status, on_press, interactive)
    }
}

/// An expandable [`TraceLine`]. See [`TraceLine::collapsible`].
pub struct Collapsible<'a, Message> {
    line: TraceLine<Message>,
    content: Element<'a, Message>,
    open: Option<bool>,
    default_open: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message: Clone + 'a> Collapsible<'a, Message> {
    /// Sets whether the content starts open.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Controls whether the content is open; `on_toggle` receives the
    /// requested state.
    pub fn open(mut self, open: bool, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.open = Some(open);
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }
}

impl<'a, Message: Clone + 'a> From<Collapsible<'a, Message>> for Element<'a, Message> {
    fn from(collapsible: Collapsible<'a, Message>) -> Self {
        let line = collapsible.line;
        let status = line.status;

        // The chevron follows the title after `gap-1`.
        let trigger = |open: bool| {
            let row = view(&line);
            let row = Row::new()
                .push(row)
                .push(
                    if open {
                        lucide::CHEVRON_DOWN
                    } else {
                        lucide::CHEVRON_RIGHT
                    }
                    .sized(16),
                )
                .spacing(4)
                .align_y(Alignment::Center);

            surface(row, status, None, true)
        };

        // `mt-2 mb-1 ml-[calc(--spacing(2)-1px)] border-l pr-1 pl-4
        // leading-relaxed text-muted-foreground`
        let content = container(
            container(collapsible.content)
                // `pl-4` after the 1px border.
                .padding(Padding::ZERO.left(17).right(4))
                .width(Length::Fill)
                .style(|theme: &Theme| {
                    let p = tokens(theme).palette;
                    container::Style {
                        text_color: Some(p.muted_foreground),
                        ..container::Style::default()
                    }
                }),
        )
        .padding(Padding::ZERO.top(8).bottom(4).left(7));

        let content = Row::new().push(content).width(Length::Fill);

        let guided = iced::widget::Stack::new().push(content).push(
            container(
                container(iced::widget::space().width(1).height(Length::Fill)).style(
                    |theme: &Theme| container::Style {
                        background: Some(Background::Color(tokens(theme).palette.border)),
                        ..container::Style::default()
                    },
                ),
            )
            .padding(Padding::ZERO.top(8).bottom(4).left(7)),
        );

        let mut disclosure = disclosure(trigger(false), trigger(true), guided)
            .default_open(collapsible.default_open)
            .width(Length::Fill);

        if let Some(open) = collapsible.open {
            disclosure = disclosure.open(open);
        }

        if let Some(on_toggle) = collapsible.on_toggle {
            disclosure = disclosure.on_toggle(on_toggle);
        }

        disclosure.into()
    }
}

/// Stacks the steps of a trace (`Trace`, `gap-2`).
pub fn trace<'a, Message: 'a>(
    lines: impl IntoIterator<Item = Element<'a, Message>>,
) -> Column<'a, Message> {
    Column::with_children(lines).spacing(8).width(Length::Fill)
}
