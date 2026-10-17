//! An opinionated toast component.
//!
//! Port of <https://ui.shadcn.com/docs/components/sonner>. Toasts live in
//! your state; the [`toaster`] displays them in a corner of the window and
//! asks you to remove each one once its duration has elapsed (or when its
//! close button is pressed):
//!
//! ```no_run
//! # use shadcn_iced::sonner::{toaster, Toast};
//! # #[derive(Clone)] enum Message { Dismiss(u64) }
//! # fn view<'a>(toasts: &'a [Toast<Message>], content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! toaster(content, toasts, Message::Dismiss).into()
//! # }
//! ```
use super::button::{Size, Variant, button};

use shadcn_iced_core::animation::{self, Appearance, Transition};
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Layer;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
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
    // `[data-icon]`: 16x16, `margin: 0 4px 0 -3px`, in the toast's color
    // (shadcn/ui does not enable sonner's rich colors).
    let icon = toast.kind.icon().map(|icon| {
        container(icon.sized(16).style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.popover_foreground),
        }))
        .padding(Padding::ZERO.left(-3.0).right(4))
    });

    let body = Column::new()
        .push(
            text(toast.title.clone())
                .size(13)
                .font(font::medium())
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
            .align_y(Alignment::Center),
    )
    .padding(16)
    .width(356)
    .style(toast_style)
    .into()
}

/// The style of a toast.
fn toast_style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        // `--normal-border: var(--border)`, `cn-toast: rounded-2xl`
        border: style::border(t.palette.border, t.radius_2xl()),
        // sonner's `0px 4px 12px rgba(0, 0, 0, 0.1)`
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
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
            .animation(shadcn_iced_core::primitive::layer::Animation::None)
            .align_x(align_x)
            .align_y(align_y)
            // sonner's `--offset: 24px`
            .padding(24)
            .into()
    }
}

/// Lays toasts out, animates them in and out, and dismisses them once their
/// duration has elapsed. Timers pause while the cursor is over the toasts,
/// like sonner.
///
/// Like sonner, the toasts are stacked behind the newest one, which they
/// peek out of (scaled down by 5% and lifted by the gap for each toast in
/// front of them, up to three toasts), until the cursor expands them into
/// a list.
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
    expanded: bool,
    /// Where each toast is drawn, moving between the stack and the list.
    motions: HashMap<u64, Motion>,
}

/// How a toast is drawn: its bounds, the opacity of its content and the
/// opacity of its card (the toasts behind the front one only show their
/// card).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Look {
    bounds: Rectangle,
    content: f32,
    card: f32,
}

impl Look {
    fn mix(self, other: Self, t: f32) -> Self {
        let lerp = |a: f32, b: f32| a + (b - a) * t;

        Self {
            bounds: Rectangle {
                x: lerp(self.bounds.x, other.bounds.x),
                y: lerp(self.bounds.y, other.bounds.y),
                width: lerp(self.bounds.width, other.bounds.width),
                height: lerp(self.bounds.height, other.bounds.height),
            },
            content: lerp(self.content, other.content),
            card: lerp(self.card, other.card),
        }
    }
}

/// A toast moving from a [`Look`] to another one, when the stack expands
/// or collapses.
#[derive(Debug, Clone, Copy)]
struct Motion {
    from: Look,
    to: Look,
    progress: Transition,
}

impl Motion {
    fn look(&self, now: Instant) -> Look {
        self.from.mix(self.to, self.progress.value(now))
    }
}

/// How long the stack takes to expand or collapse.
const MOVE: Duration = Duration::from_millis(400);

/// How many toasts peek out of a collapsed stack (`visibleToasts`).
const VISIBLE: usize = 3;

/// How much each toast behind the front one shrinks.
const SCALE_STEP: f32 = 0.05;

impl<Message> Timer<'_, Message> {
    /// How each toast should look, given the layout.
    fn looks(&self, state: &TimerState, layout: Layout<'_>) -> Vec<Look> {
        let collapsed = self.is_collapsed(state);
        let layouts: Vec<Layout<'_>> = layout.children().collect();

        (0..self.toasts.len())
            .map(|index| {
                let before = self.before(index);

                if !collapsed || before == 0 {
                    return Look {
                        bounds: layouts[index].bounds(),
                        content: 1.0,
                        card: 1.0,
                    };
                }

                // `--y: translateY(calc(var(--lift-amount) * var(--toasts-before)))
                // scale(calc(-1 * var(--scale)))`, with the height of the
                // front toast and its content hidden.
                let visible = before.min(VISIBLE - 1);
                let front = layouts[if self.from_top {
                    0
                } else {
                    self.toasts.len() - 1
                }]
                .bounds();
                let lift = self.spacing * visible as f32 * if self.from_top { 1.0 } else { -1.0 };
                let scale = 1.0 - SCALE_STEP * visible as f32;
                let center = front.center() + Vector::new(0.0, lift);
                let size = iced::Size::new(front.width * scale, front.height * scale);

                Look {
                    bounds: Rectangle::new(
                        iced::Point::new(center.x - size.width / 2.0, center.y - size.height / 2.0),
                        size,
                    ),
                    content: 0.0,
                    card: if before < VISIBLE { 1.0 } else { 0.0 },
                }
            })
            .collect()
    }

    /// Whether the toasts are stacked behind the front one.
    fn is_collapsed(&self, state: &TimerState) -> bool {
        !state.expanded && self.toasts.len() > 1
    }

    /// The number of toasts in front of the toast at `index`.
    fn before(&self, index: usize) -> usize {
        if self.from_top {
            index
        } else {
            self.toasts.len() - 1 - index
        }
    }
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
        if self.is_collapsed(tree.state.downcast_ref::<TimerState>()) {
            let nodes: Vec<layout::Node> = self
                .toasts
                .iter_mut()
                .zip(&mut tree.children)
                .map(|((_, _, toast), tree)| {
                    toast.as_widget_mut().layout(
                        tree,
                        renderer,
                        &layout::Limits::new(iced::Size::ZERO, limits.max()),
                    )
                })
                .collect();

            let front = self.before(0).max(self.before(self.toasts.len() - 1));
            let front = if self.from_top { 0 } else { front };
            let front_size = nodes[front].size();
            let width = nodes.iter().map(|n| n.size().width).fold(0.0, f32::max);
            // The toasts behind peek out by the gap.
            let peek = self.spacing * (self.toasts.len().min(VISIBLE) - 1) as f32;
            let y = if self.from_top { 0.0 } else { peek };

            return layout::Node::with_children(
                iced::Size::new(width, front_size.height + peek),
                nodes
                    .into_iter()
                    .map(|node| node.move_to(iced::Point::new(0.0, y)))
                    .collect(),
            );
        }

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
        if let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) = event {
            let state = tree.state.downcast_mut::<TimerState>();
            let expanded = cursor.is_over(layout.bounds());

            if state.expanded != expanded {
                state.expanded = expanded;
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }

        let exiting = tree.state.downcast_ref::<TimerState>().exiting.clone();
        let collapsed = self.is_collapsed(tree.state.downcast_ref::<TimerState>());
        let fronts: Vec<bool> = (0..self.toasts.len())
            .map(|index| !collapsed || self.before(index) == 0)
            .collect();

        for ((((id, _, toast), child), layout), front) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .zip(fronts)
        {
            if front && !exiting.contains(id) {
                toast.as_widget_mut().update(
                    child, event, layout, cursor, renderer, clipboard, shell, viewport,
                );
            }
        }

        // The toasts move to their new places.
        {
            let looks = self.looks(tree.state.downcast_ref::<TimerState>(), layout);
            let state = tree.state.downcast_mut::<TimerState>();
            let now = Instant::now();

            for ((id, _, _), look) in self.toasts.iter().zip(looks) {
                match state.motions.get_mut(id) {
                    Some(motion) if motion.to != look => {
                        motion.from = motion.look(now);
                        motion.to = look;
                        motion.progress = Transition::new(0.0);
                        motion.progress.go(1.0, MOVE, animation::ease_out, now);
                        shell.request_redraw();
                    }
                    Some(_) => {}
                    None => {
                        let _ = state.motions.insert(
                            *id,
                            Motion {
                                from: look,
                                to: look,
                                progress: Transition::new(1.0),
                            },
                        );
                    }
                }
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
        state.motions.retain(|id, _| ids.contains(id));

        let mut next: Option<Duration> = None;
        let mut animating = state
            .motions
            .values()
            .any(|motion| motion.progress.is_animating(*now));

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
        let targets = self.looks(state, layout);

        // The farthest toasts first.
        let mut order: Vec<usize> = (0..self.toasts.len()).collect();
        order.sort_by_key(|index| std::cmp::Reverse(self.before(*index)));

        let layouts: Vec<Layout<'_>> = layout.children().collect();

        for index in order {
            let (id, _, toast) = &self.toasts[index];
            let child = &tree.children[index];
            let layout = layouts[index];
            let progress = state
                .presence
                .get(id)
                .map_or(1.0, |presence| presence.value(now));
            let look = state
                .motions
                .get(id)
                .map_or(targets[index], |motion| motion.look(now));

            let exiting = state.exiting.contains(id);
            let height = layout.bounds().height + self.spacing;

            // Enters by sliding from the edge; leaves by fading out.
            let slide = if exiting {
                0.0
            } else if self.from_top {
                -height * (1.0 - progress)
            } else {
                height * (1.0 - progress)
            };

            // The card of a toast behind the front one, while its content
            // is hidden.
            let card = look.card * (1.0 - look.content) * progress;

            if card > 0.0 {
                let style = toast_style(theme);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: look.bounds + Vector::new(0.0, slide),
                        border: style.border,
                        shadow: style.shadow,
                        ..renderer::Quad::default()
                    },
                    style
                        .background
                        .unwrap_or(Background::Color(iced::Color::TRANSPARENT))
                        .scale_alpha(card),
                );
            }

            let opacity = look.content * progress;

            if opacity <= 0.0 {
                continue;
            }

            let bounds = layout.bounds();

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
                    opacity,
                    scale: look.bounds.width / bounds.width.max(1.0),
                    offset: look.bounds.center() - bounds.center() + Vector::new(0.0, slide),
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
        let collapsed = self.is_collapsed(tree.state.downcast_ref::<TimerState>());

        self.toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
            .filter(|(index, _)| !collapsed || self.before(*index) == 0)
            .map(|(_, (((_, _, toast), tree), layout))| {
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
