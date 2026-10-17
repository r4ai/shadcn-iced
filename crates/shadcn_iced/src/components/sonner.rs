//! An opinionated toast component.
//!
//! Port of <https://ui.shadcn.com/docs/components/sonner>. Toasts live in
//! your state; the [`toaster`] displays them in a corner of the window and
//! asks you to remove each one once it has been dismissed: when its duration
//! has elapsed, when its action or close button is pressed, or when it is
//! swiped away. Like sonner, the toasts stack behind the newest one and
//! expand into a list on hover; the behavior and the animations live in
//! [`shadcn_iced_core::primitive::toaster`].
//!
//! ```no_run
//! # use shadcn_iced::sonner::{toaster, Toast};
//! # #[derive(Clone)] enum Message { Dismiss(u64) }
//! # fn view<'a>(toasts: &'a [Toast<Message>], content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! toaster(content, toasts, Message::Dismiss).into()
//! # }
//! ```
use super::button::{Size, Variant, button};

use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Layer;
use shadcn_iced_core::primitive::toaster::{self as stack, Action, Direction, Edge};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::time::Duration;
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Padding, Theme};

/// The gap between toasts (`--gap`).
const GAP: f32 = 14.0;

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
    /// Whether the toast can be swiped away. Defaults to `true`.
    pub dismissible: bool,
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
            duration: stack::LIFETIME,
            dismissible: true,
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

    /// Adds an action button, which dismisses the toast once pressed.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Sets how long the toast stays on screen.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets whether the toast can be swiped away (`dismissible`).
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
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
    expand: bool,
    visible_toasts: usize,
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
        expand: false,
        visible_toasts: stack::VISIBLE,
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

    /// Always shows the toasts expanded instead of stacked (`expand`).
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Sets how many toasts are visible at once (`visibleToasts`, 3 by
    /// default).
    pub fn visible_toasts(mut self, count: usize) -> Self {
        self.visible_toasts = count;
        self
    }
}

/// The content of a toast; its box is drawn by the toaster.
fn view<'a, Message: Clone + 'a>(
    toast: &Toast<Message>,
    close_button: bool,
) -> Element<'a, Action<Message>> {
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

    let action = toast.action.clone().map(|(label, message)| {
        button(label)
            .size(Size::Xs)
            .on_press(Action::PublishAndDismiss(message))
    });

    let close = (close_button && toast.kind != Kind::Loading).then(|| {
        super::button::icon_button(lucide::X)
            .variant(Variant::Ghost)
            .size(Size::IconXs)
            .on_press(Action::Dismiss)
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
    // `--width`
    .width(356)
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
        use iced::alignment::{Horizontal, Vertical};

        let open = !toaster.toasts.is_empty();

        let (align_x, align_y) = match toaster.position {
            Position::TopLeft => (Horizontal::Left, Vertical::Top),
            Position::TopCenter => (Horizontal::Center, Vertical::Top),
            Position::TopRight => (Horizontal::Right, Vertical::Top),
            Position::BottomLeft => (Horizontal::Left, Vertical::Bottom),
            Position::BottomCenter => (Horizontal::Center, Vertical::Bottom),
            Position::BottomRight => (Horizontal::Right, Vertical::Bottom),
        };

        // `getDefaultSwipeDirections`: towards the edges of the position.
        let edge = match align_y {
            Vertical::Top => (Edge::Top, Direction::Up),
            _ => (Edge::Bottom, Direction::Down),
        };
        let side = match align_x {
            Horizontal::Left => Some(Direction::Left),
            Horizontal::Right => Some(Direction::Right),
            Horizontal::Center => None,
        };

        let close_button = toaster.close_button;
        let toasts = toaster
            .toasts
            .iter()
            .map(|toast| {
                stack::Toast::new(toast.id, view(toast, close_button))
                    .duration((toast.kind != Kind::Loading).then_some(toast.duration))
                    .dismissible(toast.dismissible)
            })
            .collect();

        let stack = stack::Toaster::new(toasts, toaster.on_dismiss)
            .edge(edge.0)
            .swipe_directions(std::iter::once(edge.1).chain(side))
            .gap(GAP)
            .visible(toaster.visible_toasts)
            .expand(toaster.expand)
            .surface(toast_style);

        Layer::new(toaster.base, stack, open)
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
