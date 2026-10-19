//! Augments native scroll functionality for custom, cross-browser styling.
//!
//! Port of <https://ui.shadcn.com/docs/components/scroll-area>: a
//! [`Scrollable`] with thin, rounded scrollbars drawn by
//! [`Scrollbars`](shadcn_iced_core::primitive::Scrollbars), inset in their
//! track like Base UI's.
use shadcn_iced_core::primitive::scrollbars;
use shadcn_iced_core::{Element, tokens};

use iced::widget::Scrollable;
use iced::widget::scrollable::{
    self, Direction, Rail, Scrollbar, Scroller, Status, Style, Viewport,
};
use iced::{Background, Color, Length, Theme, border};

/// A scroll area. See the [module documentation](self).
pub struct ScrollArea<'a, Message> {
    scrollable: Scrollable<'a, Message>,
    vertical: bool,
    horizontal: bool,
}

/// Creates a vertical scroll area.
pub fn scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> ScrollArea<'a, Message> {
    ScrollArea::new(content, true, false)
}

/// Creates a horizontal scroll area.
pub fn horizontal_scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> ScrollArea<'a, Message> {
    ScrollArea::new(content, false, true)
}

/// Creates a scroll area scrolling in both directions.
pub fn both_scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> ScrollArea<'a, Message> {
    ScrollArea::new(content, true, true)
}

impl<'a, Message: 'a> ScrollArea<'a, Message> {
    fn new(content: impl Into<Element<'a, Message>>, vertical: bool, horizontal: bool) -> Self {
        let direction = match (vertical, horizontal) {
            (true, true) => Direction::Both {
                vertical: Scrollbar::hidden(),
                horizontal: Scrollbar::hidden(),
            },
            (false, true) => Direction::Horizontal(Scrollbar::hidden()),
            _ => Direction::Vertical(Scrollbar::hidden()),
        };

        Self {
            scrollable: Scrollable::new(content).direction(direction).style(style),
            vertical,
            horizontal,
        }
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.scrollable = self.scrollable.width(width);
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.scrollable = self.scrollable.height(height);
        self
    }

    /// Sets the [`Id`](iced::widget::Id) of the scrollable, to scroll it
    /// with operations.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.scrollable = self.scrollable.id(id);
        self
    }

    /// Sets the message produced when the viewport scrolls.
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.scrollable = self.scrollable.on_scroll(f);
        self
    }
}

impl<'a, Message: 'a> From<ScrollArea<'a, Message>> for Element<'a, Message> {
    fn from(area: ScrollArea<'a, Message>) -> Self {
        // `bg-border rounded-full` thumbs in `p-px` tracks of `w-2.5` (or
        // `h-2.5`) with a transparent `border-l` (or `border-t`).
        scrollbars(area.scrollable)
            .vertical(area.vertical)
            .horizontal(area.horizontal)
            .thumb(|theme: &Theme| Background::Color(tokens(theme).palette.border))
            .into()
    }
}

/// The scrollbar of the scroll areas that keep iced's scrollbars (select,
/// command, code block): a 7px thumb in a 10px track.
pub fn scrollbar() -> Scrollbar {
    Scrollbar::new().width(7).scroller_width(7).margin(1.5)
}

/// The style of a scroll area (`bg-border rounded-full` thumb).
pub fn style(theme: &Theme, status: Status) -> Style {
    let p = tokens(theme).palette;
    let base = scrollable::default(theme, status);

    let rail = Rail {
        background: None,
        border: border::rounded(f32::MAX),
        scroller: Scroller {
            background: Background::Color(p.border),
            border: border::rounded(f32::MAX),
        },
    };

    Style {
        container: iced::widget::container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: Some(Background::Color(Color::TRANSPARENT)),
        auto_scroll: base.auto_scroll,
    }
}
