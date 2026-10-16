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

use iced_ui_core::animation::{self, Appearance, Transition};
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

        let top = matches!(
            toaster.position,
            Position::TopLeft | Position::TopCenter | Position::TopRight
        );
        let mut items: Vec<_> = toaster
            .toasts
            .iter()
            .map(|toast| {
                (
                    toast.id,
                    (toast.kind != Kind::Loading).then_some(toast.duration),
                    view(
                        toast,
                        toaster.close_button.then(|| (toaster.on_dismiss)(toast.id)),
                    ),
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
            toasts: items,
            spacing: 14.0,
            from_top: top,
            on_dismiss: toaster.on_dismiss,
        };

        Layer::new(toaster.base, Element::new(timer), open)
            .backdrop(None)
            .modal(false)
            .animation(iced_ui_core::primitive::layer::Animation::None)
            .align_x(align_x)
            .align_y(align_y)
            .padding(32)
            .into()
    }
}

/// Lays toasts out, animates them in and out, and dismisses them once their
/// duration has elapsed. Timers pause while the cursor is over the toasts,
/// like sonner.
struct Timer<'a, Message> {
    toasts: Vec<(u64, Option<Duration>, Element<'a, Message>)>,
    spacing: f32,
    from_top: bool,
    on_dismiss: Box<dyn Fn(u64) -> Message + 'a>,
}

#[derive(Default)]
struct TimerState {
    elapsed: HashMap<u64, Duration>,
    presence: HashMap<u64, Transition>,
    exiting: Vec<u64>,
    dismissed: Vec<u64>,
    last: Option<Instant>,
}

/// How long a toast takes to slide in (sonner's `400ms`).
const ENTER: Duration = Duration::from_millis(400);

/// How long a toast takes to fade out.
const EXIT: Duration = Duration::from_millis(200);

impl<'a, Message: Clone + 'a> Widget<Message, Theme, iced::Renderer> for Timer<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TimerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TimerState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.toasts
            .iter()
            .map(|(_, _, toast)| Tree::new(toast))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .toasts
                .iter()
                .map(|(_, _, toast)| toast)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Bounds<Length> {
        Bounds::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut y = 0.0;
        let mut width: f32 = 0.0;
        let mut nodes = Vec::with_capacity(self.toasts.len());

        for ((_, _, toast), tree) in self.toasts.iter_mut().zip(&mut tree.children) {
            let node = toast
                .as_widget_mut()
                .layout(
                    tree,
                    renderer,
                    &layout::Limits::new(iced::Size::ZERO, limits.max()),
                )
                .move_to(iced::Point::new(0.0, y));

            y += node.size().height + self.spacing;
            width = width.max(node.size().width);
            nodes.push(node);
        }

        layout::Node::with_children(iced::Size::new(width, (y - self.spacing).max(0.0)), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for (((_, _, toast), tree), layout) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            toast
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
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
        let exiting = tree.state.downcast_ref::<TimerState>().exiting.clone();

        for (((id, _, toast), child), layout) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            if !exiting.contains(id) {
                toast.as_widget_mut().update(
                    child, event, layout, cursor, renderer, clipboard, shell, viewport,
                );
            }
        }

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
        let ids: Vec<u64> = self.toasts.iter().map(|(id, _, _)| *id).collect();

        state.elapsed.retain(|id, _| ids.contains(id));
        state.presence.retain(|id, _| ids.contains(id));
        state.exiting.retain(|id| ids.contains(id));
        state.dismissed.retain(|id| ids.contains(id));

        let mut next: Option<Duration> = None;
        let mut animating = false;

        for (id, duration, _) in &self.toasts {
            let presence = state.presence.entry(*id).or_insert_with(|| {
                let mut presence = Transition::new(0.0);
                presence.go(1.0, ENTER, animation::ease_out, *now);
                presence
            });

            if state.exiting.contains(id) {
                // Dismissed once the fade-out is over.
                if !presence.is_animating(*now) && !state.dismissed.contains(id) {
                    state.dismissed.push(*id);
                    shell.publish((self.on_dismiss)(*id));
                }
            } else if let Some(duration) = duration {
                let elapsed = state.elapsed.entry(*id).or_default();

                if !paused {
                    *elapsed += delta;
                }

                if *elapsed >= *duration {
                    state.exiting.push(*id);
                    presence.go(0.0, EXIT, animation::ease_in, *now);

                    if !animation::enabled() && !state.dismissed.contains(id) {
                        state.dismissed.push(*id);
                        shell.publish((self.on_dismiss)(*id));
                    }
                } else {
                    let remaining = *duration - *elapsed;
                    next = Some(next.map_or(remaining, |next| next.min(remaining)));
                }
            }

            animating |= presence.is_animating(*now);
        }

        if animating {
            shell.request_redraw();
        } else if let Some(next) = next {
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
        let state = tree.state.downcast_ref::<TimerState>();
        let now = Instant::now();

        for (((id, _, toast), child), layout) in self
            .toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            let progress = state
                .presence
                .get(id)
                .map_or(1.0, |presence| presence.value(now));
            let exiting = state.exiting.contains(id);
            let height = layout.bounds().height + self.spacing;

            // Enters by sliding from the edge; leaves by fading out.
            let offset = if exiting {
                0.0
            } else if self.from_top {
                -height * (1.0 - progress)
            } else {
                height * (1.0 - progress)
            };

            animation::draw(
                toast,
                child,
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
                Appearance {
                    opacity: progress,
                    scale: 1.0,
                    offset: iced::Vector::new(0.0, offset),
                },
            );
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
        self.toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|(((_, _, toast), tree), layout)| {
                toast
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let overlays: Vec<_> = self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|(((_, _, toast), tree), layout)| {
                toast
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
