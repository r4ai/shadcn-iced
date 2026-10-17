use super::Fixture;
use crate::store::Msg;

use shadcn_iced::attachment::attachment;
use shadcn_iced::bubble::{Align, Variant, bubble};
use shadcn_iced::button::{Size, Variant as ButtonVariant, icon_button};
use shadcn_iced::message::message;
use shadcn_iced::prelude::lucide;

use iced::widget::{column, container, row};
use iced::{Element, Fill};

/// `w-full max-w-sm`
fn sm(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(384).into()
}

fn ghost(icon: shadcn_iced::Icon) -> Element<'static, Msg> {
    icon_button(icon)
        .variant(ButtonVariant::Ghost)
        .size(Size::Icon)
        .on_press(Msg::Noop)
        .into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "conversation" => Fixture::new(|_| {
            sm(column![
                message()
                    .align(Align::End)
                    .avatar(shadcn_iced::avatar::avatar("ME"))
                    .bubble(bubble("Deploying to prod real quick.")),
                message()
                    .avatar(shadcn_iced::avatar::avatar("R"))
                    .bubble(bubble("It's 4:55 PM. On a Friday.").variant(Variant::Muted)),
                message()
                    .align(Align::End)
                    .avatar(shadcn_iced::avatar::avatar("ME"))
                    .bubble(bubble("It's a one-line change."))
                    .footer("Delivered"),
            ]
            .spacing(24)
            .into())
        }),
        "header-footer" => Fixture::new(|_| {
            sm(column![
                message()
                    .header("Olivia")
                    .bubble(bubble("I already checked the logs.").variant(Variant::Muted)),
                message()
                    .align(Align::End)
                    .bubble(bubble("Send the report to the team."))
                    .footer("Read Yesterday"),
            ]
            .spacing(32)
            .into())
        }),
        "actions" => Fixture::new(|_| {
            sm(column![
                message()
                    .bubble(
                        bubble("The install failure is in the workspace package.")
                            .variant(Variant::Muted)
                    )
                    .footer_content(row![
                        ghost(lucide::COPY),
                        ghost(lucide::THUMBS_UP),
                        ghost(lucide::THUMBS_DOWN),
                    ]),
            ]
            .spacing(32)
            .into())
        }),
        "attachment" => Fixture::new(|_| {
            sm(message()
                .align(Align::End)
                .push(
                    attachment("quarterly-report.pdf")
                        .icon(lucide::FILE_TEXT)
                        .description("PDF · 2.4 MB"),
                )
                .bubble(bubble("Here is the report you asked for."))
                .into())
        }),
        _ => return None,
    })
}
