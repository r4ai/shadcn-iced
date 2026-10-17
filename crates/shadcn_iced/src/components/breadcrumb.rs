//! Displays the path to the current resource using a hierarchy of links.
//!
//! Port of <https://ui.shadcn.com/docs/components/breadcrumb>.
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::{Element, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, text};
use iced::{Alignment, Theme};

enum Item<'a, Message> {
    Link(String, Option<Message>),
    Page(String),
    Ellipsis,
    Custom(Element<'a, Message>),
}

/// A breadcrumb. See the [module documentation](self).
pub struct Breadcrumb<'a, Message> {
    items: Vec<Item<'a, Message>>,
    separator: Icon,
}

/// Creates an empty [`Breadcrumb`].
pub fn breadcrumb<'a, Message: Clone + 'a>() -> Breadcrumb<'a, Message> {
    Breadcrumb {
        items: Vec::new(),
        separator: lucide::CHEVRON_RIGHT,
    }
}

impl<'a, Message: Clone + 'a> Breadcrumb<'a, Message> {
    /// Appends a link.
    pub fn link(mut self, label: impl Into<String>, on_press: Message) -> Self {
        self.items.push(Item::Link(label.into(), Some(on_press)));
        self
    }

    /// Appends the current page (the last item).
    pub fn page(mut self, label: impl Into<String>) -> Self {
        self.items.push(Item::Page(label.into()));
        self
    }

    /// Appends an ellipsis, for collapsed items.
    pub fn ellipsis(mut self) -> Self {
        self.items.push(Item::Ellipsis);
        self
    }

    /// Appends a custom item, e.g. a dropdown menu.
    pub fn item(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.items.push(Item::Custom(element.into()));
        self
    }

    /// Sets the separator icon. Defaults to `chevron-right`.
    pub fn separator(mut self, icon: Icon) -> Self {
        self.separator = icon;
        self
    }
}

fn link_style(theme: &Theme, status: Status) -> Style {
    let p = tokens(theme).palette;

    Style {
        background: None,
        text_color: match status {
            Status::Hovered | Status::Pressed => p.foreground,
            _ => p.muted_foreground,
        },
        ..Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Breadcrumb<'a, Message>> for Element<'a, Message> {
    fn from(breadcrumb: Breadcrumb<'a, Message>) -> Self {
        // `text-muted-foreground gap-1.5 text-sm`
        let mut row = Row::new().spacing(6).align_y(Alignment::Center);
        let count = breadcrumb.items.len();

        for (index, item) in breadcrumb.items.into_iter().enumerate() {
            let element: Element<'a, Message> = match item {
                // `hover:text-foreground transition-colors`
                Item::Link(label, on_press) => Pressable::new(text(label).size(style::text::SM))
                    .on_press_maybe(on_press)
                    .style(link_style)
                    .into(),
                // `text-foreground font-normal`
                Item::Page(label) => text(label)
                    .size(style::text::SM)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.foreground),
                    })
                    .into(),
                Item::Ellipsis => iced::widget::container(lucide::ELLIPSIS.sized(16).style(muted))
                    // `size-5 [&>svg]:size-4`
                    .center(20)
                    .into(),
                Item::Custom(element) => element,
            };

            row = row.push(element);

            if index + 1 < count {
                // `[&>svg]:size-3.5`
                row = row.push(breadcrumb.separator.sized(14).style(muted));
            }
        }

        row.into()
    }
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}
