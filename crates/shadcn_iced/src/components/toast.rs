//! A succinct message that is displayed temporarily.
//!
//! Port of <https://ui.shadcn.com/docs/components/toast> (Base UI toast).
//! Toasts live in your state; the [`toaster`] stacks them in the bottom
//! right corner of the window, expands the stack on hover, and asks you to
//! remove each toast once its timeout has elapsed or when it is closed:
//!
//! ```no_run
//! # use shadcn_iced::toast::{toaster, Toast};
//! # #[derive(Clone)] enum Message { Close(u64) }
//! # fn view<'a>(toasts: &'a [Toast<Message>], content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! toaster(content, toasts, Message::Close).into()
//! # }
//! ```
//!
//! See [`sonner`](super::sonner) for the other toast of shadcn/ui.
use super::button::{self as buttons, Size, Variant, button};

use shadcn_iced_core::animation::{self, Appearance, Transition};
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::{Layer, Pressable};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};
use iced::widget::{Column, Row, container, text};
use iced::{
    Alignment, Background, Event, Length, Point, Rectangle, Size as Bounds, Theme, Vector, mouse,
    window,
};

use std::collections::HashMap;

/// The type of a [`Toast`], which decides its icon (`type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Kind {
    /// No icon.
    #[default]
    Default,
    /// `CircleCheckIcon`
    Success,
    /// `InfoIcon`
    Info,
    /// `TriangleAlertIcon`
    Warning,
    /// `OctagonXIcon`, in the destructive color.
    Error,
    /// A spinner; loading toasts never time out.
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

/// A toast.
#[derive(Debug, Clone)]
pub struct Toast<Message> {
    /// A unique identifier, reported back when the toast must be removed.
    pub id: u64,
    /// The title (`ToastTitle`).
    pub title: String,
    /// An optional description (`ToastDescription`).
    pub description: Option<String>,
    /// The type.
    pub kind: Kind,
    /// An optional action button (`actionProps`): a label and its message.
    pub action: Option<(String, Message)>,
    /// How long the toast stays on screen. Defaults to 5 seconds.
    pub timeout: Duration,
}

impl<Message> Toast<Message> {
    /// Creates a toast with a title, which may be empty for a toast with
    /// a description only.
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            kind: Kind::Default,
            action: None,
            timeout: Duration::from_secs(5),
        }
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the type.
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    /// Adds an action button (`variant="outline" size="sm"`).
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Sets how long the toast stays on screen.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// A toaster. See the [module documentation](self).
pub struct Toaster<'a, Message> {
    base: Element<'a, Message>,
    toasts: Vec<Toast<Message>>,
    on_close: Box<dyn Fn(u64) -> Message + 'a>,
    limit: usize,
}

/// Displays `toasts` over `base`, which is usually the root of your view.
/// The last toast is the newest.
pub fn toaster<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    toasts: &[Toast<Message>],
    on_close: impl Fn(u64) -> Message + 'a,
) -> Toaster<'a, Message> {
    Toaster {
        base: base.into(),
        toasts: toasts.to_vec(),
        on_close: Box::new(on_close),
        limit: 3,
    }
}

impl<'a, Message: Clone + 'a> Toaster<'a, Message> {
    /// Sets how many toasts are shown at once (`limit`, 3 by default).
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }
}

/// `text-muted-foreground`
fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

fn view<'a, Message: Clone + 'a>(toast: &Toast<Message>, close: Message) -> Element<'a, Message> {
    let kind = toast.kind;

    // `ToastIcon`: `size-4`, `text-destructive` for errors
    let icon = kind.icon().map(|icon| {
        icon.sized(16).style(move |theme: &Theme| text::Style {
            color: (kind == Kind::Error).then(|| tokens(theme).palette.destructive),
        })
    });

    // `flex min-w-0 flex-1 flex-col gap-1`
    let body = Column::new()
        .push((!toast.title.is_empty()).then(|| {
            text(toast.title.clone())
                .size(style::text::SM)
                .line_height(style::text::leading(style::text::SM))
                .font(font::medium())
        }))
        .push(toast.description.clone().map(|description| {
            text(description)
                .size(style::text::SM)
                .line_height(style::text::leading(style::text::SM))
                .style(muted)
        }))
        .spacing(4)
        .width(Length::Fill);

    let action = toast.action.clone().map(|(label, message)| {
        button(label)
            .variant(Variant::Outline)
            .size(Size::Sm)
            .on_press(message)
    });

    // `ToastClose`: `variant="ghost" size="icon-sm" text-muted-foreground
    // hover:text-foreground`
    let size = Size::IconSm;
    let close = Pressable::new(
        container(lucide::X.sized(size.icon_size()))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(close)
    .width(size.height())
    .height(size.height())
    .press_offset(1.0)
    .clip_background(true)
    .style(move |theme, status| {
        let t = tokens(theme);
        let mut style = buttons::style(theme, status, Variant::Ghost);

        if !matches!(
            status,
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
        ) {
            style.text_color = t.palette.muted_foreground;
        }

        style.border.radius = size.radius(&t).into();
        style
    });

    // `ToastContent`: `flex items-center gap-3 p-4`
    container(
        Row::new()
            .push(icon)
            .push(body)
            .push(action)
            .push(close)
            .spacing(12)
            .align_y(Alignment::Center),
    )
    // Inside the 1px border.
    .padding(17)
    .width(Length::Fill)
    .style(self::style)
    .into()
}

/// The style of a toast (`rounded-2xl border bg-popover
/// text-popover-foreground shadow-lg`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        border: style::border(t.palette.border, t.radius_2xl()),
        // `shadow-lg`, whose layers shrink by their negative spread.
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 10.0,
        },
        ..container::Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Toaster<'a, Message>> for Element<'a, Message> {
    fn from(toaster: Toaster<'a, Message>) -> Self {
        let open = !toaster.toasts.is_empty();
        let skip = toaster.toasts.len().saturating_sub(toaster.limit);

        let toasts: Vec<_> = toaster
            .toasts
            .iter()
            .skip(skip)
            .map(|toast| {
                (
                    toast.id,
                    (toast.kind != Kind::Loading).then_some(toast.timeout),
                    view(toast, (toaster.on_close)(toast.id)),
                )
            })
            .collect();

        let stack = Stack {
            toasts,
            on_close: toaster.on_close,
        };

        // `fixed bottom-4 right-4 max-w-sm w-full`
        Layer::new(
            toaster.base,
            container(Element::new(stack)).max_width(384),
            open,
        )
        .backdrop(None)
        .modal(false)
        .animation(shadcn_iced_core::primitive::layer::Animation::None)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(16)
        .into()
    }
}

/// `[--gap:0.75rem]`, between expanded toasts.
const GAP: f32 = 12.0;

/// `[--peek:0.75rem]`, how much each toast behind the front one shows.
const PEEK: f32 = 12.0;

/// How long a toast takes to slide in or out (`500ms`).
const SLIDE: Duration = Duration::from_millis(500);

/// `cubic-bezier(0.22, 1, 0.36, 1)`
fn ease(t: f32) -> f32 {
    animation::cubic_bezier(0.22, 1.0, 0.36, 1.0, t)
}

/// Stacks toasts like Base UI: collapsed behind the newest one, expanded
/// while hovered. Dismisses them once their timeout has elapsed; timers
/// pause while the stack is hovered.
struct Stack<'a, Message> {
    toasts: Vec<(u64, Option<Duration>, Element<'a, Message>)>,
    on_close: Box<dyn Fn(u64) -> Message + 'a>,
}

#[derive(Default)]
struct State {
    elapsed: HashMap<u64, Duration>,
    presence: HashMap<u64, Transition>,
    exiting: Vec<u64>,
    closed: Vec<u64>,
    expanded: bool,
    last: Option<Instant>,
}

impl<Message> Stack<'_, Message> {
    /// The bounds of the collapsed stack: the front toast and the peeks.
    fn collapsed(&self, layout: Layout<'_>) -> Rectangle {
        let bounds = layout.bounds();
        let front = layout
            .children()
            .last()
            .map_or(Rectangle::default(), |layout| layout.bounds());
        let peeks = self.toasts.len().saturating_sub(1) as f32 * PEEK;

        Rectangle {
            y: front.y - peeks,
            height: front.height + peeks,
            ..bounds
        }
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, iced::Renderer> for Stack<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
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
        Bounds::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let child_limits = layout::Limits::new(Bounds::ZERO, limits.max()).width(width);

        let nodes: Vec<_> = self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .map(|((_, _, toast), tree)| {
                toast.as_widget_mut().layout(tree, renderer, &child_limits)
            })
            .collect();

        // Expanded positions, the newest toast at the bottom.
        let total: f32 = nodes.iter().map(|node| node.size().height).sum::<f32>()
            + nodes.len().saturating_sub(1) as f32 * GAP;
        let mut y = total;
        let mut placed = Vec::with_capacity(nodes.len());

        for node in nodes.into_iter().rev() {
            y -= node.size().height;
            placed.push(node.move_to(Point::new(0.0, y)));
            y -= GAP;
        }

        placed.reverse();

        layout::Node::with_children(Bounds::new(width, total), placed)
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
        let (expanded, exiting) = {
            let state = tree.state.downcast_ref::<State>();
            (state.expanded, state.exiting.clone())
        };
        let count = self.toasts.len();

        for (index, (((id, _, toast), child), layout)) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
        {
            // Only the front toast is interactive while collapsed.
            if !exiting.contains(id) && (expanded || index + 1 == count) {
                toast.as_widget_mut().update(
                    child, event, layout, cursor, renderer, clipboard, shell, viewport,
                );
            }
        }

        let hovered = if expanded {
            cursor.is_over(layout.bounds())
        } else {
            cursor.is_over(self.collapsed(layout))
        };

        let state = tree.state.downcast_mut::<State>();

        if hovered != state.expanded {
            state.expanded = hovered;
            shell.invalidate_layout();
            shell.request_redraw();
        }

        let Event::Window(window::Event::RedrawRequested(now)) = event else {
            return;
        };

        let delta = state
            .last
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or_default();
        state.last = Some(*now);

        let ids: Vec<u64> = self.toasts.iter().map(|(id, _, _)| *id).collect();
        state.elapsed.retain(|id, _| ids.contains(id));
        state.presence.retain(|id, _| ids.contains(id));
        state.exiting.retain(|id| ids.contains(id));
        state.closed.retain(|id| ids.contains(id));

        let mut next: Option<Duration> = None;
        let mut animating = false;

        for (id, timeout, _) in &self.toasts {
            let presence = state.presence.entry(*id).or_insert_with(|| {
                let mut presence = Transition::new(0.0);
                presence.go(1.0, SLIDE, ease, *now);
                presence
            });

            if state.exiting.contains(id) {
                if !presence.is_animating(*now) && !state.closed.contains(id) {
                    state.closed.push(*id);
                    shell.publish((self.on_close)(*id));
                }
            } else if let Some(timeout) = timeout {
                let elapsed = state.elapsed.entry(*id).or_default();

                if !state.expanded {
                    *elapsed += delta;
                }

                if *elapsed >= *timeout {
                    state.exiting.push(*id);
                    presence.go(0.0, SLIDE, ease, *now);

                    if !animation::enabled() && !state.closed.contains(id) {
                        state.closed.push(*id);
                        shell.publish((self.on_close)(*id));
                    }
                } else {
                    let remaining = *timeout - *elapsed;
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
        let state = tree.state.downcast_ref::<State>();
        let now = Instant::now();
        let count = self.toasts.len();
        let front = layout
            .children()
            .last()
            .map_or(Rectangle::default(), |layout| layout.bounds());

        for (index, (((id, _, toast), child), layout)) in self
            .toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            let progress = state
                .presence
                .get(id)
                .map_or(1.0, |presence| presence.value(now));
            // `data-starting-style:[transform:translateY(150%)]`
            let slide = Vector::new(0.0, layout.bounds().height * 1.5 * (1.0 - progress));
            let behind = count - 1 - index;

            if state.expanded || behind == 0 {
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
                        opacity: 1.0,
                        scale: 1.0,
                        offset: slide,
                    },
                );
                continue;
            }

            // Behind the front toast: an empty card of the front height,
            // `scale(1 - index * 0.1)` from the bottom and raised by
            // `index * peek` (`data-behind:opacity-0` hides its content).
            let scale = (1.0 - behind as f32 * 0.1).max(0.0);
            let width = front.width * scale;
            let top = front.y - behind as f32 * PEEK;
            let bounds = Rectangle {
                x: front.x + (front.width - width) / 2.0,
                y: top + slide.y,
                width,
                height: front.height * scale,
            };
            let style = self::style(theme);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..renderer::Quad::default()
                },
                style
                    .background
                    .unwrap_or(Background::Color(iced::Color::TRANSPARENT)),
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
        let expanded = tree.state.downcast_ref::<State>().expanded;
        let count = self.toasts.len();

        self.toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
            .filter(|(index, _)| expanded || index + 1 == count)
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
