//! Augments native scroll functionality for custom, cross-browser styling.
//!
//! Port of <https://ui.shadcn.com/docs/components/scroll-area>: a
//! [`Scrollable`] with a thin, rounded scrollbar.
use iced_ui_core::{Element, tokens};

use iced::widget::Scrollable;
use iced::widget::scrollable::{self, Direction, Rail, Scrollbar, Scroller, Status, Style};
use iced::{Background, Color, Theme, border};

/// Creates a vertical scroll area.
pub fn scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(Direction::Vertical(scrollbar()))
        .style(style)
}

/// Creates a horizontal scroll area.
pub fn horizontal_scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(Direction::Horizontal(scrollbar()))
        .style(style)
}

/// Creates a scroll area scrolling in both directions.
pub fn both_scroll_area<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(Direction::Both {
            vertical: scrollbar(),
            horizontal: scrollbar(),
        })
        .style(style)
}

/// The scrollbar of a scroll area (`w-2.5 p-px border-l
/// border-l-transparent`): a 7px thumb in a 10px track.
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
