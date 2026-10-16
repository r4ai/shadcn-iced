//! A vertically stacked set of interactive headings that each reveal a
//! section of content.
//!
//! Port of <https://ui.shadcn.com/docs/components/accordion>.
//!
//! By default, items open and close on their own (`type="multiple"`). Use
//! [`Accordion::single`] to only allow one open item, controlled by your
//! application.
//!
//! ```no_run
//! # use iced_ui::accordion::accordion;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! accordion()
//!     .item("Is it accessible?", iced::widget::text("Yes."))
//!     .item("Is it styled?", iced::widget::text("Yes."))
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::Disclosure;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Length, Theme};

/// An accordion. See the [module documentation](self).
pub struct Accordion<'a, Message> {
    items: Vec<(String, Element<'a, Message>, bool)>,
    single: Option<(Option<usize>, Box<dyn Fn(Option<usize>) -> Message + 'a>)>,
    width: Length,
}

/// Creates an empty [`Accordion`].
pub fn accordion<'a, Message: Clone + 'a>() -> Accordion<'a, Message> {
    Accordion {
        items: Vec::new(),
        single: None,
        width: Length::Fill,
    }
}

impl<'a, Message: Clone + 'a> Accordion<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, title: impl Into<String>, content: impl Into<Element<'a, Message>>) -> Self {
        self.items.push((title.into(), content.into(), false));
        self
    }

    /// Appends an item that starts open.
    pub fn open_item(
        mut self,
        title: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        self.items.push((title.into(), content.into(), true));
        self
    }

    /// Only allows one item to be open at a time (`type="single"
    /// collapsible`). `open` is the index of the open item.
    pub fn single(mut self, open: Option<usize>, on_change: impl Fn(Option<usize>) -> Message + 'a) -> Self {
        self.single = Some((open, Box::new(on_change)));
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

fn trigger<'a, Message: 'a>(title: &str, open: bool) -> Element<'a, Message> {
    Row::new()
        .push(
            text(title.to_owned())
                .size(style::text::SM)
                .font(font::MEDIUM)
                .width(Length::Fill),
        )
        .push(
            (if open { lucide::CHEVRON_UP } else { lucide::CHEVRON_DOWN })
                .sized(16)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                }),
        )
        .spacing(16)
        .align_y(Alignment::Center)
        .padding([16, 0])
        .into()
}

impl<'a, Message: Clone + 'a> From<Accordion<'a, Message>> for Element<'a, Message> {
    fn from(accordion: Accordion<'a, Message>) -> Self {
        let count = accordion.items.len();
        let single = accordion.single;

        let items = accordion.items.into_iter().enumerate().map(|(index, (title, content, default_open))| {
            let content = container(content).padding(iced::Padding::ZERO.bottom(16)).width(Length::Fill);
            let mut disclosure = Disclosure::new(trigger(&title, false), trigger(&title, true), content)
                .default_open(default_open);

            if let Some((open, on_change)) = &single {
                let is_open = *open == Some(index);
                let message = on_change(if is_open { None } else { Some(index) });

                disclosure = disclosure.open(is_open).on_toggle(move |_| message.clone());
            }

            let item = Column::new().push(disclosure);

            if index + 1 < count {
                item.push(super::separator::separator()).into()
            } else {
                item.into()
            }
        });

        Column::with_children(items).width(accordion.width).into()
    }
}
