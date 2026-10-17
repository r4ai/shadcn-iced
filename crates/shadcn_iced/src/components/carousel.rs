//! A carousel with motion and swipe built using iced.
//!
//! Port of <https://ui.shadcn.com/docs/components/carousel>. The current
//! index lives in your state:
//!
//! ```no_run
//! # use shadcn_iced::carousel::carousel;
//! # #[derive(Clone)] enum Message { Slide(usize) }
//! # fn view<'a>(index: usize) -> iced::Element<'a, Message> {
//! carousel(index, Message::Slide)
//!     .item(iced::widget::text("1"))
//!     .item(iced::widget::text("2"))
//!     .into()
//! # }
//! ```
use super::button::{Size, Variant, icon_button};

use shadcn_iced_core::Element;
use shadcn_iced_core::icon::lucide;

use iced::widget::{Column, Row, container};
use iced::{Alignment, Length};

/// The scrolling direction of a [`Carousel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Items side by side, previous/next on the left/right.
    #[default]
    Horizontal,
    /// Items stacked, previous/next above/below.
    Vertical,
}

/// A carousel. See the [module documentation](self).
pub struct Carousel<'a, Message> {
    index: usize,
    on_change: Box<dyn Fn(usize) -> Message + 'a>,
    items: Vec<Element<'a, Message>>,
    per_view: usize,
    spacing: f32,
    orientation: Orientation,
    looping: bool,
}

/// Creates a [`Carousel`] showing the item at `index` first.
pub fn carousel<'a, Message: Clone + 'a>(
    index: usize,
    on_change: impl Fn(usize) -> Message + 'a,
) -> Carousel<'a, Message> {
    Carousel {
        index,
        on_change: Box::new(on_change),
        items: Vec::new(),
        per_view: 1,
        spacing: 16.0,
        orientation: Orientation::Horizontal,
        looping: false,
    }
}

impl<'a, Message: Clone + 'a> Carousel<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Appends several items.
    pub fn items(mut self, items: impl IntoIterator<Item = Element<'a, Message>>) -> Self {
        self.items.extend(items);
        self
    }

    /// Sets how many items are visible at once (`basis-1/3`...).
    pub fn per_view(mut self, count: usize) -> Self {
        self.per_view = count.max(1);
        self
    }

    /// Sets the space between visible items (`pl-4` by default).
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Wraps around at both ends (`loop: true`).
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Carousel<'a, Message>> for Element<'a, Message> {
    fn from(carousel: Carousel<'a, Message>) -> Self {
        let count = carousel.items.len();
        let last = count.saturating_sub(carousel.per_view);
        let index = carousel.index.min(last);
        let horizontal = carousel.orientation == Orientation::Horizontal;

        let previous = if index > 0 {
            Some(index - 1)
        } else if carousel.looping && count > 0 {
            Some(last)
        } else {
            None
        };

        let next = if index < last {
            Some(index + 1)
        } else if carousel.looping && count > 0 {
            Some(0)
        } else {
            None
        };

        // `variant="outline" size="icon-sm" rounded-full`
        let nav = |icon, target: Option<usize>| {
            icon_button(icon)
                .variant(Variant::Outline)
                .size(Size::IconSm)
                .disabled(target.is_none())
                .on_press_maybe(target.map(|target| (carousel.on_change)(target)))
                .position(shadcn_iced_core::style::Position::Alone, false)
        };

        let visible = carousel
            .items
            .into_iter()
            .skip(index)
            .take(carousel.per_view)
            .map(|item| container(item).width(Length::Fill).into());

        if horizontal {
            Row::new()
                .push(round(nav(lucide::CHEVRON_LEFT, previous)))
                .push(
                    Row::with_children(visible)
                        .spacing(carousel.spacing)
                        .width(Length::Fill),
                )
                .push(round(nav(lucide::CHEVRON_RIGHT, next)))
                // `-left-12` / `-right-12`: 48px from the slides, minus
                // the 28px button.
                .spacing(20)
                .align_y(Alignment::Center)
                .into()
        } else {
            Column::new()
                .push(round(nav(lucide::CHEVRON_UP, previous)))
                .push(Column::with_children(visible).spacing(carousel.spacing))
                .push(round(nav(lucide::CHEVRON_DOWN, next)))
                // `-top-12` / `-bottom-12`
                .spacing(20)
                .align_x(Alignment::Center)
                .into()
        }
    }
}

/// Makes the navigation buttons fully rounded (`rounded-full`).
fn round<'a, Message: Clone + 'a>(
    button: super::button::Button<'a, Message>,
) -> Element<'a, Message> {
    button.rounded(true).into()
}
