//! Pagination with page navigation, next and previous links.
//!
//! Port of <https://ui.shadcn.com/docs/components/pagination>.
//!
//! ```no_run
//! # use shadcn_iced::pagination::pagination;
//! # #[derive(Clone)] enum Message { Page(usize) }
//! # fn view<'a>(page: usize) -> iced::Element<'a, Message> {
//! pagination(page, 10, Message::Page).into()
//! # }
//! ```
use super::button::{self, Button, Variant};

use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::{Element, font, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Length, Padding, Theme};

/// A pagination. See the [module documentation](self).
pub struct Pagination<'a, Message> {
    current: usize,
    total: usize,
    siblings: usize,
    numbers: bool,
    controls: bool,
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
        numbers: true,
        controls: true,
        on_change: Box::new(on_change),
    }
}

impl<'a, Message: Clone + 'a> Pagination<'a, Message> {
    /// Sets how many pages are shown around the current one.
    pub fn siblings(mut self, siblings: usize) -> Self {
        self.siblings = siblings;
        self
    }

    /// Shows the page numbers. Without them, only the previous and next
    /// buttons are shown.
    pub fn numbers(mut self, numbers: bool) -> Self {
        self.numbers = numbers;
        self
    }

    /// Shows the previous and next buttons.
    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
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

/// `PaginationPrevious` / `PaginationNext`: a ghost button of the default
/// size, with `pl-1.5!` / `pr-1.5!` on the side of the chevron.
fn step<'a, Message: Clone + 'a>(
    label: &'static str,
    icon: Icon,
    previous: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let size = button::Size::Default;
    let icon = icon.sized(size.icon_size());
    let label = text(label)
        .size(size.text_size())
        .font(font::medium())
        .wrapping(text::Wrapping::None);

    let content = if previous {
        Row::new().push(icon).push(label)
    } else {
        Row::new().push(label).push(icon)
    };

    Pressable::new(
        content
            .spacing(6)
            .align_y(Alignment::Center)
            .height(Length::Fill),
    )
    .disabled(on_press.is_none())
    .on_press_maybe(on_press)
    .height(size.height())
    .padding(Padding {
        left: if previous { 6.0 } else { 10.0 },
        right: if previous { 10.0 } else { 6.0 },
        ..Padding::ZERO
    })
    .press_offset(1.0)
    .style(move |theme, status| {
        let mut style = button::style(theme, status, Variant::Ghost);
        style.border.radius = size.radius(&tokens(theme)).into();
        style
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Pagination<'a, Message>> for Element<'a, Message> {
    fn from(pagination: Pagination<'a, Message>) -> Self {
        let current = pagination.current;
        let total = pagination.total;
        let on_change = &pagination.on_change;

        // `gap-0.5`
        let mut row = Row::new().spacing(2).align_y(Alignment::Center);

        if pagination.controls {
            row = row.push(step(
                "Previous",
                lucide::CHEVRON_LEFT,
                true,
                (current > 1).then(|| on_change(current - 1)),
            ));
        }

        if pagination.numbers {
            for page in pages(current, total, pagination.siblings) {
                row = row.push(match page {
                    // `PaginationLink`: an icon button, outlined when active.
                    Some(page) => Element::from(
                        Button::new(
                            text(page.to_string())
                                .size(14)
                                .font(font::medium())
                                .wrapping(text::Wrapping::None),
                        )
                        .variant(if page == current {
                            Variant::Outline
                        } else {
                            Variant::Ghost
                        })
                        .size(button::Size::Icon)
                        .on_press(on_change(page)),
                    ),
                    // `size-8 [&_svg:not([class*='size-'])]:size-4`
                    None => container(lucide::ELLIPSIS.sized(16).style(|theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(tokens(theme).palette.foreground),
                        }
                    }))
                    .center(32)
                    .into(),
                });
            }
        }

        if pagination.controls {
            row = row.push(step(
                "Next",
                lucide::CHEVRON_RIGHT,
                false,
                (current < total).then(|| on_change(current + 1)),
            ));
        }

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
        assert_eq!(
            pages(2, 10, 1),
            vec![Some(1), Some(2), Some(3), None, Some(10)]
        );
    }
}
