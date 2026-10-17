//! A succinct message that is displayed temporarily.
//!
//! Port of <https://ui.shadcn.com/docs/components/toast> (Base UI toast).
//! Toasts live in your state; the [`toaster`] stacks them in the bottom
//! right corner of the window, expands the stack on hover, and asks you to
//! remove each toast once its timeout has elapsed, when it is closed or when
//! it is swiped away. The behavior and the animations live in
//! [`shadcn_iced_core::primitive::toaster`], shared with
//! [`sonner`](super::sonner):
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

use shadcn_iced_core::animation;
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::toaster::{self as stack, Action, Direction, Motion, Origin};
use shadcn_iced_core::primitive::{Layer, Pressable};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::time::Duration;
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Theme, Vector};

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

fn view<'a, Message: Clone + 'a>(toast: &Toast<Message>) -> Element<'a, Action<Message>> {
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
            .on_press(Action::Publish(message))
    });

    // `ToastClose`: `variant="ghost" size="icon-sm" text-muted-foreground
    // hover:text-foreground`
    let size = Size::IconSm;
    let close = Pressable::new(
        container(lucide::X.sized(size.icon_size()))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(Action::Dismiss)
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
    // Inside the 1px border, drawn by the stack.
    .padding(17)
    .width(Length::Fill)
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

/// `[--gap:0.75rem]`, between expanded toasts.
const GAP: f32 = 12.0;

/// `[--peek:0.75rem]`, how much each toast behind the front one shows.
const PEEK: f32 = 12.0;

/// `cubic-bezier(0.22, 1, 0.36, 1)`
fn ease(t: f32) -> f32 {
    animation::cubic_bezier(0.22, 1.0, 0.36, 1.0, t)
}

/// How the toasts of Base UI move: they slide in from `translateY(150%)` in
/// 500ms without fading, and the toasts behind the front one take its
/// height, peek out by `--peek` and shrink by 10% each from their top.
fn motion() -> Motion {
    Motion {
        travel: 1.5,
        fade: false,
        presence: Duration::from_millis(500),
        duration: Duration::from_millis(400),
        easing: ease,
        peek: Some(PEEK),
        shrink: 0.1,
        origin: Origin::Top,
        unmount: Duration::from_millis(500),
    }
}

impl<'a, Message: Clone + 'a> From<Toaster<'a, Message>> for Element<'a, Message> {
    fn from(toaster: Toaster<'a, Message>) -> Self {
        let open = !toaster.toasts.is_empty();
        let skip = toaster.toasts.len().saturating_sub(toaster.limit);

        let toasts = toaster
            .toasts
            .iter()
            .skip(skip)
            .map(|toast| {
                stack::Toast::new(toast.id, view(toast))
                    .duration((toast.kind != Kind::Loading).then_some(toast.timeout))
            })
            .collect();

        // `swipeDirection={['down', 'right']}`
        let stack = stack::Toaster::new(toasts, toaster.on_close)
            .gap(GAP)
            .visible(toaster.limit)
            .swipe_directions([Direction::Down, Direction::Right])
            .motion(motion())
            .surface(self::style);

        // `fixed bottom-4 right-4 max-w-sm w-full`
        Layer::new(toaster.base, container(stack).max_width(384), open)
            .backdrop(None)
            .modal(false)
            .animation(shadcn_iced_core::primitive::layer::Animation::None)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(16)
            .into()
    }
}
