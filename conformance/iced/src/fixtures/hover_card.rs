use super::Fixture;

use shadcn_iced::button::{Variant, button};
use shadcn_iced::hover_card::hover_card;
use shadcn_iced::tokens;

use iced::widget::{column, container, text};
use iced::{Element, Pixels, Theme};

use crate::store::Msg;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "escape" | "stays-open" => Fixture::new(|_| card()),
        _ => return None,
    })
}

fn card() -> Element<'static, Msg> {
    // `flex flex-col gap-0.5`, `text-sm` (20px line height).
    let content = column![
        text("@nextjs")
            .size(14)
            .line_height(Pixels(20.0))
            .font(shadcn_iced::font::semibold()),
        text("The React Framework – created and maintained by @vercel.")
            .size(14)
            .line_height(Pixels(20.0)),
        // `mt-1 text-xs text-muted-foreground`
        container(
            text("Joined December 2021")
                .size(12)
                .line_height(Pixels(16.0))
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
        )
        .padding(iced::Padding::ZERO.top(4)),
    ]
    .spacing(2);

    hover_card(button("@nextjs").variant(Variant::Link), content).into()
}
