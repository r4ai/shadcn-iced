//! The theme customizer, like shadcn/ui's `/create` page (without styles).
use crate::{Gallery, Message};

use shadcn_iced::button::{self, button};
use shadcn_iced::code_block::code_block;
use shadcn_iced::field::field;
use shadcn_iced::select::{Size, select};
use shadcn_iced::theme::{Accent, BaseColor, MenuAccent, MenuColor, Radius, ThemeColor};
use shadcn_iced::{font, style};

use iced::widget::{column, text};
use iced::{Element, Fill};

/// The heading font (`fontHeading`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Heading {
    #[default]
    Inherit,
    Geist,
    Inter,
}

impl Heading {
    pub const ALL: &'static [Self] = &[Self::Inherit, Self::Geist, Self::Inter];

    pub fn family(self) -> Option<&'static str> {
        match self {
            Heading::Inherit => None,
            Heading::Geist => Some(font::GEIST),
            Heading::Inter => Some(font::INTER),
        }
    }
}

impl std::fmt::Display for Heading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Heading::Inherit => "Inherit",
            Heading::Geist => "Geist",
            Heading::Inter => "Inter",
        })
    }
}

/// The body font, picked at launch with `GALLERY_FONT=inter` (iced fixes
/// the default font when the application starts).
pub fn body_font() -> &'static str {
    match std::env::var("GALLERY_FONT").as_deref() {
        Ok("inter") => font::INTER,
        _ => font::GEIST,
    }
}

/// The content of the customizer sheet.
pub fn view(gallery: &Gallery) -> Element<'_, Message> {
    let base = select(Some(gallery.base), Message::Base)
        .options(BaseColor::ALL.iter().copied())
        .size(Size::Sm)
        .width(Fill);

    let theme = Accent::ALL
        .iter()
        .fold(
            select(Some(gallery.accent), Message::Accent).option(None, gallery.base.to_string()),
            |select, accent| select.option(Some(*accent), accent.to_string()),
        )
        .size(Size::Sm)
        .width(Fill);

    let chart = Accent::ALL
        .iter()
        .fold(
            select(Some(gallery.chart), Message::Chart)
                .option(None, "Same as theme")
                .option(
                    Some(ThemeColor::Base(gallery.base)),
                    gallery.base.to_string(),
                ),
            |select, accent| select.option(Some(ThemeColor::Accent(*accent)), accent.to_string()),
        )
        .size(Size::Sm)
        .width(Fill);

    let heading = select(Some(gallery.heading), Message::Heading)
        .options(Heading::ALL.iter().copied())
        .size(Size::Sm)
        .width(Fill);

    let radius = select(Some(gallery.radius), Message::Radius)
        .options(Radius::ALL.iter().copied())
        .size(Size::Sm)
        .width(Fill);

    let menu_color = select(Some(gallery.menu_color), Message::MenuColor)
        .options(MenuColor::ALL.iter().copied())
        .size(Size::Sm)
        .width(Fill);

    let menu_accent = select(Some(gallery.menu_accent), Message::MenuAccent)
        .option(MenuAccent::Subtle, "Subtle")
        .option(MenuAccent::Bold, "Bold")
        .size(Size::Sm)
        .width(Fill);

    column![
        field("Base Color", base),
        field("Theme", theme),
        field("Chart Color", chart),
        field("Heading", heading).description(format!(
            "Body font: {} (set GALLERY_FONT=inter at launch).",
            body_font()
        )),
        field("Radius", radius),
        field("Menu Color", menu_color),
        field("Menu Accent", menu_accent),
        text("Code").size(style::text::SM).font(font::medium()),
        code_block(code(gallery))
            .language("rust")
            .copy_button(Message::CopyTheme, gallery.copied)
            .wrap(true),
        button("Reset")
            .variant(button::Variant::Outline)
            .icon(shadcn_iced::prelude::lucide::ROTATE_CCW)
            .width(Fill)
            .on_press(Message::ResetTheme),
    ]
    .spacing(16)
    // Room for the scrollbar.
    .padding(iced::Padding::ZERO.right(12))
    .into()
}

/// The Rust code building the customized theme.
pub fn code(gallery: &Gallery) -> String {
    let mut code = format!(
        "use shadcn_iced::theme::{{self, Accent, BaseColor, MenuAccent, MenuColor, Radius}};\n\n\
         let theme = theme::Config::new(BaseColor::{:?}, mode)",
        gallery.base
    );

    if let Some(accent) = gallery.accent {
        code.push_str(&format!("\n    .accent(Accent::{accent:?})"));
    }

    match gallery.chart {
        Some(ThemeColor::Base(base)) => {
            code.push_str(&format!("\n    .chart_color(BaseColor::{base:?})"));
        }
        Some(ThemeColor::Accent(accent)) => {
            code.push_str(&format!("\n    .chart_color(Accent::{accent:?})"));
        }
        None => {}
    }

    if gallery.radius != Radius::Default {
        code.push_str(&format!(
            "\n    .radius_preset(Radius::{:?})",
            gallery.radius
        ));
    }

    if gallery.menu_accent != MenuAccent::Subtle {
        code.push_str(&format!(
            "\n    .menu_accent(MenuAccent::{:?})",
            gallery.menu_accent
        ));
    }

    if gallery.menu_color != MenuColor::Default {
        code.push_str(&format!(
            "\n    .menu_color(MenuColor::{:?})",
            gallery.menu_color
        ));
    }

    code.push_str("\n    .build();");

    if body_font() != font::GEIST {
        code.push_str(
            "\n\n// Before `.shadcn_iced()`:\nshadcn_iced::font::set_sans(shadcn_iced::font::INTER);",
        );
    }

    if let Some(family) = gallery.heading.family() {
        let name = if family == font::INTER {
            "INTER"
        } else {
            "GEIST"
        };
        code.push_str(&format!(
            "\nshadcn_iced::font::set_heading(Some(shadcn_iced::font::{name}));"
        ));
    }

    code
}
