//! A set of layered sections of content, known as tab panels, that are
//! displayed one at a time.
//!
//! Port of <https://ui.shadcn.com/docs/components/tabs>.
//!
//! ```no_run
//! # use iced_ui::tabs::tabs;
//! # #[derive(Clone, Copy, PartialEq)] enum Tab { Account, Password }
//! # #[derive(Clone)] enum Message { Tab(Tab) }
//! # fn view<'a>(tab: Tab) -> iced::Element<'a, Message> {
//! tabs(tab, Message::Tab)
//!     .tab(Tab::Account, "Account")
//!     .tab(Tab::Password, "Password")
//!     .content(iced::widget::text("..."))
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Border, Color, Length, Theme, border};

/// The visual style of the tab list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Triggers inside a muted pill.
    #[default]
    Default,
    /// Transparent triggers with an underline under the active one.
    Line,
}

/// Tabs. See the [module documentation](self).
pub struct Tabs<'a, T, Message> {
    selected: T,
    tabs: Vec<(T, Option<Icon>, String, bool)>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    content: Option<Element<'a, Message>>,
    variant: Variant,
    width: Length,
}

/// Creates [`Tabs`] with the given selected value.
pub fn tabs<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a>(
    selected: T,
    on_select: impl Fn(T) -> Message + 'a,
) -> Tabs<'a, T, Message> {
    Tabs {
        selected,
        tabs: Vec::new(),
        on_select: Box::new(on_select),
        content: None,
        variant: Variant::Default,
        width: Length::Shrink,
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> Tabs<'a, T, Message> {
    /// Appends a tab.
    pub fn tab(mut self, value: T, label: impl Into<String>) -> Self {
        self.tabs.push((value, None, label.into(), false));
        self
    }

    /// Appends a tab with an icon.
    pub fn tab_with_icon(mut self, value: T, icon: Icon, label: impl Into<String>) -> Self {
        self.tabs.push((value, Some(icon), label.into(), false));
        self
    }

    /// Appends a disabled tab.
    pub fn disabled_tab(mut self, value: T, label: impl Into<String>) -> Self {
        self.tabs.push((value, None, label.into(), true));
        self
    }

    /// Sets the content of the selected tab, displayed below the list.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the width of the tab list. With [`Length::Fill`], triggers share
    /// the width equally.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> From<Tabs<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(tabs: Tabs<'a, T, Message>) -> Self {
        let variant = tabs.variant;
        let fill = tabs.width != Length::Shrink;

        let triggers = tabs.tabs.into_iter().map(|(value, icon, label, disabled)| {
            let active = value == tabs.selected;

            let content = Row::new()
                .push(icon.map(|icon| icon.sized(16)))
                .push(
                    text(label)
                        .size(style::text::SM)
                        .font(font::MEDIUM)
                        .wrapping(text::Wrapping::None),
                )
                .spacing(6)
                .align_y(Alignment::Center);

            let content: Element<'a, Message> = if fill {
                container(content).center_x(Length::Fill).into()
            } else {
                content.into()
            };

            Pressable::new(container(content).center_y(Length::Fill))
                .on_press_maybe((!disabled && !active).then(|| (tabs.on_select)(value)))
                .disabled(disabled)
                .width(if fill { Length::Fill } else { Length::Shrink })
                .height(Length::Fill)
                .padding([0, 8])
                .style(move |theme, status| trigger(theme, status, active, variant))
                .into()
        });

        let list = container(Row::with_children(triggers).spacing(match variant {
            Variant::Default => 0,
            Variant::Line => 4,
        }))
        .padding(3)
        .height(36)
        .width(tabs.width)
        .style(move |theme| self::list(theme, variant));

        let has_content = tabs.content.is_some();

        Column::new()
            .push(list)
            .push(tabs.content)
            .spacing(8)
            .width(if fill || has_content { Length::Fill } else { Length::Shrink })
            .into()
    }
}

/// The style of the tab list.
pub fn list(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: match variant {
            Variant::Default => Some(Background::Color(t.palette.muted)),
            Variant::Line => None,
        },
        text_color: Some(t.palette.muted_foreground),
        border: match variant {
            Variant::Default => border::rounded(t.radius_lg()),
            Variant::Line => border::rounded(0),
        },
        ..container::Style::default()
    }
}

/// The style of a tab trigger.
pub fn trigger(theme: &Theme, status: Status, active: bool, variant: Variant) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    let text_color = if active || hovered {
        p.foreground
    } else if dark {
        p.muted_foreground
    } else {
        alpha(p.foreground, 0.6)
    };

    let mut style = match variant {
        Variant::Default => Style {
            background: active.then(|| {
                Background::Color(if dark { alpha(p.input, 0.3) } else { p.background })
            }),
            text_color,
            border: Border {
                color: if active && dark { p.input } else { Color::TRANSPARENT },
                width: 1.0,
                radius: border::radius(t.radius_md()),
            },
            shadow: if active && !dark { style::shadow_sm() } else { iced::Shadow::default() },
            ..Style::default()
        },
        Variant::Line => Style {
            background: None,
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(0),
            },
            ..Style::default()
        },
    };

    if status == Status::Disabled {
        style.text_color = alpha(style.text_color, style::DISABLED_OPACITY);
    }

    style
}
