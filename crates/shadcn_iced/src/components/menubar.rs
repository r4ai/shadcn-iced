//! A visually persistent menu common in desktop applications that provides
//! quick access to a consistent set of commands.
//!
//! Port of <https://ui.shadcn.com/docs/components/menubar>.
//!
//! ```no_run
//! # use shadcn_iced::menubar::menubar;
//! # use shadcn_iced::menu::{menu, item};
//! # #[derive(Clone)] enum Message { New }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! menubar()
//!     .menu("File", menu().item(item("New Tab").shortcut("⌘T").on_select(Message::New)))
//!     .into()
//! # }
//! ```
use super::menu::{Kind, Menu, menu_tokens};

use shadcn_iced_core::primitive::anchored::group;
use shadcn_iced_core::primitive::{Align, Anchored, Pressable, Side};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{container, text};
use iced::{Background, Length, Theme, border};

/// A menubar. See the [module documentation](self).
pub struct Menubar<'a, Message> {
    menus: Vec<(String, Menu<'a, Message>)>,
    width: Length,
}

/// Creates an empty [`Menubar`].
pub fn menubar<'a, Message: Clone + 'a>() -> Menubar<'a, Message> {
    Menubar {
        menus: Vec::new(),
        width: Length::Fill,
    }
}

impl<'a, Message: Clone + 'a> Menubar<'a, Message> {
    /// Appends a menu.
    pub fn menu(mut self, label: impl Into<String>, menu: Menu<'a, Message>) -> Self {
        self.menus.push((label.into(), menu));
        self
    }

    /// Sets the width. Defaults to [`Length::Fill`]: like the `flex` block
    /// of shadcn/ui, a menubar spans the available width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

/// The style of a menubar trigger (`hover:bg-muted rounded-sm`).
fn trigger_style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: hovered.then_some(Background::Color(t.palette.muted)),
        text_color: t.palette.foreground,
        border: border::rounded(t.radius_sm()),
        ..Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Menubar<'a, Message>> for Element<'a, Message> {
    fn from(menubar: Menubar<'a, Message>) -> Self {
        let width = menubar.width;
        let menus = menubar.menus.into_iter().map(|(label, menu)| {
            // `px-1.5 py-[2px] text-sm font-medium`
            let trigger = Pressable::new(
                text(label)
                    .size(style::text::SM)
                    .font(font::medium())
                    .line_height(iced::Pixels(20.0)),
            )
            .padding([2, 6])
            .style(trigger_style);

            let menu = menu.kind(Kind::Menubar);
            let anchor_width = menu.anchor_min_width();

            // `align="start" alignOffset={-4} sideOffset={8}`
            let anchored = Anchored::new(trigger, menu)
                .side(Side::Bottom)
                .align(Align::Start)
                .offset(8.0)
                .align_offset(-4.0)
                .outset(1.0)
                .close_on_select(true)
                .modal(true)
                // `aria-expanded:bg-muted`
                .open_highlight(|theme| {
                    let t = menu_tokens(theme);
                    (
                        Background::Color(t.palette.muted),
                        border::rounded(t.radius_sm()),
                    )
                });

            // `w-(--anchor-width) min-w-36`
            match anchor_width {
                Some(min) => anchored.anchor_width(min),
                None => anchored,
            }
        });

        // `flex h-8 gap-0.5 rounded-lg border p-[3px]`: a block spanning the
        // available width. While a menu is open, hovering another trigger
        // or pressing `ArrowLeft` / `ArrowRight` switches menus.
        container(group(menus).spacing(2.0))
            // The 1px border plus `p-[3px]`.
            .padding(4)
            .width(width)
            .height(32)
            .style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    border: style::border(t.palette.border, t.radius_lg()),
                    ..container::Style::default()
                }
            })
            .into()
    }
}
