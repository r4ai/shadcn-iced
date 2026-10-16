//! Pagination with page navigation, next and previous links.
//!
//! Port of <https://ui.shadcn.com/docs/components/pagination>.
//!
//! ```no_run
//! # use iced_ui::pagination::pagination;
//! # #[derive(Clone)] enum Message { Page(usize) }
//! # fn view<'a>(page: usize) -> iced::Element<'a, Message> {
//! pagination(page, 10, Message::Page).into()
//! # }
//! ```
use super::button::{self, Button, Variant};

use iced_ui_core::icon::lucide;
use iced_ui_core::{Element, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Theme};

/// A pagination. See the [module documentation](self).
pub struct Pagination<'a, Message> {
    current: usize,
    total: usize,
    siblings: usize,
    on_change: Box<dyn Fn(usize) -> Message + 'a>,
}

/// Creates a [`Pagination`] for `total` pages, with `current` (1-based)
/// being the active one.
pub fn pagination<'a, Message: Clone + 'a>(
    current: usize,
    total: usize,
    on_change: impl Fn(usize) -> Message + 'a,
) -> Pagination<'a, Message> {
    Pagination {
        current: current.clamp(1, total.max(1)),
        total: total.max(1),
        siblings: 1,
        on_change: Box::new(on_change),
    }
}

impl<'a, Message: Clone + 'a> Pagination<'a, Message> {
    /// Sets how many pages are shown around the current one.
    pub fn siblings(mut self, siblings: usize) -> Self {
        self.siblings = siblings;
        self
    }
}

/// Returns the page numbers to display, with `None` standing for an
/// ellipsis.
pub fn pages(current: usize, total: usize, siblings: usize) -> Vec<Option<usize>> {
    let start = current.saturating_sub(siblings).max(1);
    let end = (current + siblings).min(total);

    let mut pages = Vec::new();

    if start > 1 {
        pages.push(Some(1));
        if start > 2 {
            pages.push(None);
        }
    }

    pages.extend((start..=end).map(Some));

    if end < total {
        if end + 1 < total {
            pages.push(None);
        }
        pages.push(Some(total));
    }

    pages
}

impl<'a, Message: Clone + 'a> From<Pagination<'a, Message>> for Element<'a, Message> {
    fn from(pagination: Pagination<'a, Message>) -> Self {
        let current = pagination.current;
        let total = pagination.total;
        let on_change = &pagination.on_change;

        let mut row = Row::new().spacing(4).align_y(Alignment::Center);

        row = row.push(
            button::button("Previous")
                .icon(lucide::CHEVRON_LEFT)
                .variant(Variant::Ghost)
                .disabled(current == 1)
                .on_press_maybe((current > 1).then(|| on_change(current - 1))),
        );

        for page in pages(current, total, pagination.siblings) {
            row = row.push(match page {
                Some(page) => Element::from(
                    Button::new(text(page.to_string()).size(14))
                        .variant(if page == current { Variant::Outline } else { Variant::Ghost })
                        .size(button::Size::Icon)
                        .on_press(on_change(page)),
                ),
                None => container(lucide::ELLIPSIS.sized(16).style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.foreground),
                }))
                .center(36)
                .into(),
            });
        }

        row = row.push(
            button::button("Next")
                .trailing_icon(lucide::CHEVRON_RIGHT)
                .variant(Variant::Ghost)
                .disabled(current == total)
                .on_press_maybe((current < total).then(|| on_change(current + 1))),
        );

        row.into()
    }
}

#[cfg(test)]
mod tests {
    use super::pages;

    #[test]
    fn collapses_with_ellipsis() {
        assert_eq!(pages(1, 3, 1), vec![Some(1), Some(2), Some(3)]);
        assert_eq!(
            pages(5, 10, 1),
            vec![Some(1), None, Some(4), Some(5), Some(6), None, Some(10)]
        );
        assert_eq!(pages(2, 10, 1), vec![Some(1), Some(2), Some(3), None, Some(10)]);
    }
}
