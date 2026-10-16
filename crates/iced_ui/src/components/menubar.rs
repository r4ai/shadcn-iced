//! A visually persistent menu common in desktop applications that provides
//! quick access to a consistent set of commands.
//!
//! Port of <https://ui.shadcn.com/docs/components/menubar>.
//!
//! ```no_run
//! # use iced_ui::menubar::menubar;
//! # use iced_ui::menu::{menu, item};
//! # #[derive(Clone)] enum Message { New }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! menubar()
//!     .menu("File", menu().item(item("New Tab").shortcut("⌘T").on_select(Message::New)))
//!     .into()
//! # }
//! ```
use super::menu::Menu;

use iced_ui_core::primitive::{Align, Anchored, Pressable, Side};
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, container, text};
use iced::{Background, Theme, border};

/// A menubar. See the [module documentation](self).
pub struct Menubar<'a, Message> {
    menus: Vec<(String, Menu<'a, Message>)>,
}

/// Creates an empty [`Menubar`].
pub fn menubar<'a, Message: Clone + 'a>() -> Menubar<'a, Message> {
    Menubar { menus: Vec::new() }
}

impl<'a, Message: Clone + 'a> Menubar<'a, Message> {
    /// Appends a menu.
    pub fn menu(mut self, label: impl Into<String>, menu: Menu<'a, Message>) -> Self {
        self.menus.push((label.into(), menu));
        self
    }
}

fn trigger_style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: hovered.then_some(Background::Color(t.palette.accent)),
        text_color: if hovered {
            t.palette.accent_foreground
        } else {
            t.palette.foreground
        },
        border: border::rounded(t.radius_sm()),
        ..Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Menubar<'a, Message>> for Element<'a, Message> {
    fn from(menubar: Menubar<'a, Message>) -> Self {
        let menus = menubar.menus.into_iter().map(|(label, menu)| {
            let trigger = Pressable::new(
                text(label)
                    .size(style::text::SM)
                    .font(font::MEDIUM)
                    .line_height(iced::Pixels(20.0)),
            )
            .padding([4, 8])
            .style(trigger_style);

            Anchored::new(trigger, menu.min_width(192.0))
                .side(Side::Bottom)
                .align(Align::Start)
                .offset(8.0)
                .align_offset(-4.0)
                .close_on_select(true)
                .modal(true)
                .into()
        });

        container(Row::with_children(menus).spacing(4))
            .padding(4)
            .height(36)
            .style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.background)),
                    border: style::border(t.palette.border, t.radius_md()),
                    shadow: style::shadow_xs(),
                    ..container::Style::default()
                }
            })
            .into()
    }
}
