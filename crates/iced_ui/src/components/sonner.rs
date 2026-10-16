//! An opinionated toast component.
//!
//! Port of <https://ui.shadcn.com/docs/components/sonner>. Toasts live in
//! your state; the [`toaster`] displays them in a corner of the window and
//! asks you to remove each one once its duration has elapsed (or when its
//! close button is pressed):
//!
//! ```no_run
//! # use iced_ui::sonner::{toaster, Toast};
//! # #[derive(Clone)] enum Message { Dismiss(u64) }
//! # fn view<'a>(toasts: &'a [Toast<Message>], content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! toaster(content, toasts, Message::Dismiss).into()
//! # }
//! ```
use super::button::{Size, Variant, button};

use iced_ui_core::icon::{Icon, lucide};
use iced_ui_core::primitive::Layer;
use iced_ui_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};
use iced::widget::{Column, Row, container, text};
use iced::{
    Alignment, Background, Event, Length, Padding, Rectangle, Size as Bounds, Theme, Vector, mouse,
    window,
};

use std::collections::HashMap;

/// The kind of a [`Toast`], which decides its icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Kind {
    /// No icon.
    #[default]
    Default,
    /// A check.
    Success,
    /// An info circle.
    Info,
    /// A warning triangle.
    Warning,
    /// An error octagon.
    Error,
    /// A spinner; loading toasts never expire on their own.
    Loading,
}

impl Kind {
    fn icon(self) -> Option<Icon> {
        match self {
            Kind::Default => None,
            Kind::Success => Some(lucide::CIRCLE_CHECK),
            Kind::Info => Some(lucide::INFO),
            Kind::Warning => Some(lucide::TRIANGLE_ALERT),
            Kind::Error => Some(lucide::OCTAGON_X),
            Kind::Loading => Some(lucide::LOADER_CIRCLE),
        }
    }
}

/// A toast notification.
#[derive(Debug, Clone)]
pub struct Toast<Message> {
    /// A unique identifier, reported back when the toast must be removed.
    pub id: u64,
    /// The title.
    pub title: String,
    /// An optional description.
    pub description: Option<String>,
    /// The kind.
    pub kind: Kind,
    /// An optional action button: a label and its message.
    pub action: Option<(String, Message)>,
    /// How long the toast stays on screen. Defaults to 4 seconds.
    pub duration: Duration,
}

impl<Message> Toast<Message> {
    /// Creates a toast with a title.
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            kind: Kind::Default,
            action: None,
            duration: Duration::from_secs(4),
        }
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the kind.
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    /// Adds an action button.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Sets how long the toast stays on screen.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Where the toaster displays toasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Position {
    /// Top left corner.
    TopLeft,
    /// Top center.
    TopCenter,
    /// Top right corner.
    TopRight,
    /// Bottom left corner.
    BottomLeft,
    /// Bottom center.
    BottomCenter,
    /// Bottom right corner.
    #[default]
    BottomRight,
}

/// A toaster. See the [module documentation](self).
pub struct Toaster<'a, Message> {
    base: Element<'a, Message>,
    toasts: Vec<Toast<Message>>,
    on_dismiss: Box<dyn Fn(u64) -> Message + 'a>,
    position: Position,
    close_button: bool,
}

/// Displays `toasts` over `base`, which is usually the root of your view.
pub fn toaster<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    toasts: &[Toast<Message>],
    on_dismiss: impl Fn(u64) -> Message + 'a,
) -> Toaster<'a, Message> {
    Toaster {
        base: base.into(),
        toasts: toasts.to_vec(),
        on_dismiss: Box::new(on_dismiss),
        position: Position::BottomRight,
        close_button: false,
    }
}

impl<'a, Message: Clone + 'a> Toaster<'a, Message> {
    /// Shows a close button on every toast (`closeButton`).
    pub fn close_button(mut self, show: bool) -> Self {
        self.close_button = show;
        self
    }

    /// Sets the position of the toasts.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}

fn view<'a, Message: Clone + 'a>(
    toast: &Toast<Message>,
    dismiss: Option<Message>,
) -> Element<'a, Message> {
    let icon = toast.kind.icon().map(|icon| {
        let kind = toast.kind;
        container(icon.sized(16).style(move |theme: &Theme| text::Style {
            color: Some(match kind {
                Kind::Error => tokens(theme).palette.destructive,
                _ => tokens(theme).palette.popover_foreground,
            }),
        }))
        .padding(Padding::ZERO.top(2))
    });

    let body = Column::new()
        .push(
            text(toast.title.clone())
                .size(13)
                .font(font::MEDIUM)
                .line_height(1.5),
        )
        .push(toast.description.clone().map(|description| {
            text(description)
                .size(13)
                .line_height(1.4)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
        }))
        .spacing(2)
        .width(Length::Fill);

    let action = toast
        .action
        .clone()
        .map(|(label, message)| button(label).size(Size::Xs).on_press(message));

    let close = dismiss.map(|dismiss| {
        super::button::icon_button(lucide::X)
            .variant(Variant::Ghost)
            .size(Size::IconXs)
            .on_press(dismiss)
    });

    container(
        Row::new()
            .push(icon)
            .push(body)
            .push(action)
            .push(close)
            .spacing(6)
            .align_y(Alignment::Start),
    )
    .padding(16)
    .width(356)
    .style(|theme: &Theme| {
        let t = tokens(theme);
        container::Style {
            background: Some(Background::Color(t.palette.popover)),
            text_color: Some(t.palette.popover_foreground),
            border: style::border(t.palette.border, t.radius_lg()),
            shadow: style::shadow_lg(),
            ..container::Style::default()
        }
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Toaster<'a, Message>> for Element<'a, Message> {
    fn from(toaster: Toaster<'a, Message>) -> Self {
        let open = !toaster.toasts.is_empty();
        let deadlines: Vec<_> = toaster
            .toasts
            .iter()
            .map(|toast| {
                (
                    toast.id,
                    (toast.kind != Kind::Loading).then_some(toast.duration),
                )
            })
            .collect();

        let top = matches!(
            toaster.position,
            Position::TopLeft | Position::TopCenter | Position::TopRight
        );
        let mut items: Vec<_> = toaster
            .toasts
            .iter()
            .map(|toast| {
                view(
                    toast,
                    toaster.close_button.then(|| (toaster.on_dismiss)(toast.id)),
                )
            })
            .collect();

        if top {
            items.reverse();
        }

        let (align_x, align_y) = match toaster.position {
            Position::TopLeft => (
                iced::alignment::Horizontal::Left,
                iced::alignment::Vertical::Top,
            ),
            Position::TopCenter => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Top,
            ),
            Position::TopRight => (
                iced::alignment::Horizontal::Right,
                iced::alignment::Vertical::Top,
            ),
            Position::BottomLeft => (
                iced::alignment::Horizontal::Left,
                iced::alignment::Vertical::Bottom,
            ),
            Position::BottomCenter => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Bottom,
            ),
            Position::BottomRight => (
                iced::alignment::Horizontal::Right,
                iced::alignment::Vertical::Bottom,
            ),
        };

        let timer = Timer {
            content: Column::with_children(items).spacing(14).into(),
            deadlines,
            on_dismiss: toaster.on_dismiss,
        };

        Layer::new(toaster.base, Element::new(timer), open)
            .backdrop(None)
            .modal(false)
            .align_x(align_x)
            .align_y(align_y)
            .padding(32)
            .into()
    }
}

/// Dismisses toasts once their duration has elapsed. Timers pause while the
/// cursor is over the toasts, like sonner.
struct Timer<'a, Message> {
    content: Element<'a, Message>,
    deadlines: Vec<(u64, Option<Duration>)>,
    on_dismiss: Box<dyn Fn(u64) -> Message + 'a>,
}

#[derive(Default)]
struct TimerState {
    elapsed: HashMap<u64, Duration>,
    dismissed: Vec<u64>,
    last: Option<Instant>,
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, iced::Renderer> for Timer<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TimerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TimerState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Bounds<Length> {
        self.content.as_widget().size()
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

        let Event::Window(window::Event::RedrawRequested(now)) = event else {
            return;
        };

        let state = tree.state.downcast_mut::<TimerState>();
        let delta = state
            .last
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or_default();
        state.last = Some(*now);

        let paused = cursor.is_over(layout.bounds());
        let ids: Vec<u64> = self.deadlines.iter().map(|(id, _)| *id).collect();

        state.elapsed.retain(|id, _| ids.contains(id));
        state.dismissed.retain(|id| ids.contains(id));

        let mut next: Option<Duration> = None;

        for (id, duration) in &self.deadlines {
            let Some(duration) = duration else { continue };
            let elapsed = state.elapsed.entry(*id).or_default();

            if !paused {
                *elapsed += delta;
            }

            if *elapsed >= *duration {
                if !state.dismissed.contains(id) {
                    state.dismissed.push(*id);
                    shell.publish((self.on_dismiss)(*id));
                }
            } else {
                let remaining = *duration - *elapsed;
                next = Some(next.map_or(remaining, |next| next.min(remaining)));
            }
        }

        if let Some(next) = next {
            shell.request_redraw_at(*now + next.min(Duration::from_millis(250)));
        }
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
