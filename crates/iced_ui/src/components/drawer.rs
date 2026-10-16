//! A panel that slides in from an edge of the window.
//!
//! Port of <https://ui.shadcn.com/docs/components/drawer> (Base UI
//! drawer).
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::drawer::drawer;
//! # #[derive(Clone)] enum Message { Open, Close }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! drawer(button("Open Drawer").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Move Goal")
//!     .description("Set your daily activity goal.")
//!     .on_close(Message::Close)
//!     .into()
//! # }
//! ```
use iced_ui_core::primitive::Layer;
use iced_ui_core::primitive::layer::{Animation, Edge};
use iced_ui_core::{Element, tokens};

use iced::widget::{Column, Row, container, space};
use iced::{Alignment, Background, Border, Length, Padding, Theme, border};

/// The direction a [`Drawer`] is swiped to close it, i.e. the edge of the
/// window it is attached to (`swipeDirection`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Attached to the bottom edge.
    #[default]
    Down,
    /// Attached to the top edge.
    Up,
    /// Attached to the left edge.
    Left,
    /// Attached to the right edge.
    Right,
}

impl Direction {
    fn is_vertical(self) -> bool {
        matches!(self, Direction::Down | Direction::Up)
    }
}

/// A drawer. See the [module documentation](self).
pub struct Drawer<'a, Message> {
    base: Element<'a, Message>,
    open: bool,
    direction: Direction,
    swipe_handle: bool,
    title: Option<String>,
    description: Option<String>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    on_close: Option<Message>,
    max_width: Option<f32>,
}

/// Creates a [`Drawer`] shown at the bottom of the window when `open`.
pub fn drawer<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    open: bool,
) -> Drawer<'a, Message> {
    Drawer {
        base: base.into(),
        open,
        direction: Direction::Down,
        swipe_handle: false,
        title: None,
        description: None,
        content: Vec::new(),
        footer: None,
        on_close: None,
        max_width: None,
    }
}

impl<'a, Message: Clone + 'a> Drawer<'a, Message> {
    /// Sets the edge the drawer is attached to.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Shows the swipe handle (`showSwipeHandle`): a small bar on the inner
    /// edge.
    pub fn swipe_handle(mut self, show: bool) -> Self {
        self.swipe_handle = show;
        self
    }

    /// Sets the title (`text-base font-medium`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description (`text-sm text-muted-foreground`).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends content, in a `flex-1 p-4` area between the header and the
    /// footer.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer (`mt-auto gap-2 p-4 pt-0`).
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the message produced when the user closes the drawer.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Limits the width of the inner content of a top/bottom drawer and
    /// centers it (`mx-auto max-w-sm` in the shadcn/ui examples).
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Drawer<'a, Message>> for Element<'a, Message> {
    fn from(drawer: Drawer<'a, Message>) -> Self {
        let direction = drawer.direction;
        let vertical = direction.is_vertical();

        // `gap-0.5 p-4 pb-0`, centered on the y axis.
        let header = (drawer.title.is_some() || drawer.description.is_some()).then(|| {
            Column::new()
                .push(drawer.title.map(super::dialog::title_text))
                .push(
                    drawer
                        .description
                        .map(|description| super::dialog::description_text(description, vertical)),
                )
                .spacing(2)
                .padding(Padding::new(16.0).bottom(0))
                .align_x(if vertical {
                    Alignment::Center
                } else {
                    Alignment::Start
                })
                .width(Length::Fill)
        });

        let content = (!drawer.content.is_empty()).then(|| {
            container(Column::with_children(drawer.content).spacing(16))
                .padding(16)
                .width(Length::Fill)
        });

        let footer = drawer.footer.map(|footer| {
            container(footer)
                .padding(Padding::new(16.0).top(0))
                .width(Length::Fill)
        });

        let body = Column::new()
            .push(header)
            .push(content)
            .push((!vertical).then(space::vertical))
            .push(footer)
            .width(Length::Fill)
            .height(if vertical {
                Length::Shrink
            } else {
                Length::Fill
            });

        let body: Element<'a, Message> = match drawer.max_width {
            Some(max_width) if vertical => container(body.max_width(max_width))
                .center_x(Length::Fill)
                .into(),
            _ => body.into(),
        };

        // `after:bg-muted`: `h-3 after:h-1 after:w-24` on the y axis,
        // `w-3 after:w-1 after:h-24` on the x axis.
        let handle = drawer.swipe_handle.then(|| {
            let bar = container(if vertical {
                space().width(96).height(4)
            } else {
                space().width(4).height(96)
            })
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(tokens(theme).palette.muted)),
                border: border::rounded(f32::MAX),
                ..container::Style::default()
            });

            let strip = container(bar);

            match direction {
                Direction::Down => strip.center_x(Length::Fill).align_bottom(12),
                Direction::Up => strip.center_x(Length::Fill).align_top(12),
                Direction::Right => strip.center_y(Length::Fill).align_right(12),
                Direction::Left => strip.center_y(Length::Fill).align_left(12),
            }
        });

        let content: Element<'a, Message> = match direction {
            Direction::Down => Column::new().push(handle).push(body).into(),
            Direction::Up => Column::new().push(body).push(handle).into(),
            Direction::Right => Row::new().push(handle).push(body).into(),
            Direction::Left => Row::new().push(body).push(handle).into(),
        };

        let radius = move |r: f32| match direction {
            Direction::Down => border::radius(0).top_left(r).top_right(r),
            Direction::Up => border::radius(0).bottom_left(r).bottom_right(r),
            Direction::Right => border::radius(0).top_left(r).bottom_left(r),
            Direction::Left => border::radius(0).top_right(r).bottom_right(r),
        };

        // `bg-popover text-popover-foreground text-sm`, rounded on the inner
        // edge (`rounded-t-xl` when attached to the bottom).
        let inner = container(content)
            .width(Length::Fill)
            .height(if vertical {
                Length::Shrink
            } else {
                Length::Fill
            })
            .style(move |theme: &Theme| {
                let t = tokens(theme);

                container::Style {
                    background: Some(Background::Color(t.palette.popover)),
                    text_color: Some(t.palette.popover_foreground),
                    border: Border {
                        radius: radius(t.radius_xl()),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            });

        // The border of the inner edge (`border-t` when attached to the
        // bottom), drawn as a border-colored panel peeking out by 1px.
        let panel = container(inner)
            .padding(match direction {
                Direction::Down => Padding::ZERO.top(1),
                Direction::Up => Padding::ZERO.bottom(1),
                Direction::Right => Padding::ZERO.left(1),
                Direction::Left => Padding::ZERO.right(1),
            })
            .width(if vertical {
                Length::Fill
            } else {
                // `w-3/4 sm:w-[24rem]`
                Length::Fixed(384.0)
            })
            .height(if vertical {
                Length::Shrink
            } else {
                Length::Fill
            })
            .style(move |theme: &Theme| {
                let t = tokens(theme);

                container::Style {
                    background: Some(Background::Color(t.palette.border)),
                    border: Border {
                        radius: radius(t.radius_xl()),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            });

        let (align_x, align_y, edge, padding) = match direction {
            Direction::Down => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Bottom,
                Edge::Bottom,
                // `max-h-[calc(100dvh-6rem)]`
                Padding::ZERO.top(96),
            ),
            Direction::Up => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Top,
                Edge::Top,
                Padding::ZERO.bottom(96),
            ),
            Direction::Right => (
                iced::alignment::Horizontal::Right,
                iced::alignment::Vertical::Center,
                Edge::Right,
                Padding::ZERO,
            ),
            Direction::Left => (
                iced::alignment::Horizontal::Left,
                iced::alignment::Vertical::Center,
                Edge::Left,
                Padding::ZERO,
            ),
        };

        Layer::new(drawer.base, panel, drawer.open)
            .animation(Animation::Slide(edge))
            .align_x(align_x)
            .align_y(align_y)
            .padding(padding)
            .backdrop_with(super::dialog::backdrop)
            .on_dismiss_maybe(drawer.on_close)
            .into()
    }
}
