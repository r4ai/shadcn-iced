use super::Fixture;
use crate::store::{self, Msg};

use shadcn_iced::button::{Size, Variant, button};
use shadcn_iced::card::{self, card};
use shadcn_iced::input::input;
use shadcn_iced::label::label;

use iced::widget::{column, row, space, text};
use iced::{Center, Fill, Pixels};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "login" => Fixture::new(|store| {
            card::<Msg>()
                .title("Login to your account")
                .description("Enter your email below to login to your account")
                .action(button("Sign Up").variant(Variant::Link).on_press(Msg::Noop))
                .content(
                    column![
                        column![
                            label("Email"),
                            input("m@example.com", store.str("email"))
                                .on_input(store::set::<String>("email")),
                        ]
                        .spacing(8),
                        column![
                            row![
                                label("Password"),
                                space::horizontal(),
                                text("Forgot your password?")
                                    .size(14)
                                    .line_height(Pixels(20.0)),
                            ]
                            .align_y(Center),
                            input("", store.str("password"))
                                .secure(true)
                                .on_input(store::set::<String>("password")),
                        ]
                        .spacing(8),
                    ]
                    .spacing(24),
                )
                .footer(
                    column![
                        button("Login").width(Fill).on_press(Msg::Noop),
                        button("Login with Google")
                            .variant(Variant::Outline)
                            .width(Fill)
                            .on_press(Msg::Noop),
                    ]
                    .spacing(8),
                )
                .width(384)
                .into()
        }),
        "small" => Fixture::new(|_| {
            card::<Msg>()
                .size(card::Size::Sm)
                .title("Small Card")
                .description("This card uses the small size variant.")
                .content(
                    text(
                        "The card component supports a size prop that can be set to \"sm\" for a more compact appearance.",
                    )
                    .size(14)
                    .line_height(Pixels(20.0)),
                )
                .footer(
                    button("Action")
                        .variant(Variant::Outline)
                        .size(Size::Sm)
                        .width(Fill)
                        .on_press(Msg::Noop),
                )
                .width(384)
                .into()
        }),
        _ => return None,
    })
}
