use super::Fixture;

use shadcn_iced::separator::separator;
use shadcn_iced::typography as ty;

use iced::Center;
use iced::widget::{column, row};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "headings" => Fixture::new(|_| {
            column![
                ty::h1("Taxing Laughter: The Joke Tax Chronicles"),
                // `border-b pb-2`
                column![ty::h2("The People of the Kingdom"), separator()]
                    .spacing(8)
                    .width(iced::Fill),
                ty::h3("The Joke Tax"),
                ty::h4("People stopped telling jokes"),
            ]
            .spacing(16)
            .into()
        }),
        "text" => Fixture::new(|_| {
            column![
                ty::p(
                    "The king thought long and hard, and finally came up with a brilliant plan: he would tax the jokes in the kingdom."
                ),
                ty::lead("A modal dialog that interrupts the user with important content."),
                ty::large("Are you absolutely sure?"),
                ty::small("Email address"),
                ty::muted_text("Enter your email address."),
                row![
                    ty::p("Run"),
                    ty::inline_code("cargo shadcn-iced add button"),
                    ty::p("to copy it."),
                ]
                .spacing(6)
                .align_y(Center),
            ]
            .spacing(16)
            .into()
        }),
        "blockquote-list" => Fixture::new(|_| {
            column![
                ty::blockquote(
                    "\"After all,\" he said, \"everyone enjoys a good joke, so it's only fair that they should pay for the privilege.\""
                ),
                ty::list([
                    "1st level of puns: 5 gold coins",
                    "2nd level of jokes: 10 gold coins",
                    "3rd level of one-liners: 20 gold coins",
                ]),
            ]
            .spacing(16)
            .into()
        }),
        _ => return None,
    })
}
