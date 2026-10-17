use super::Fixture;
use crate::store::Msg;

use shadcn_iced::alert::{Variant, alert};
use shadcn_iced::button::{Size, button};
use shadcn_iced::prelude::lucide;

use iced::widget::column;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            column![
                alert::<Msg>("Payment successful")
                    .icon(lucide::CIRCLE_CHECK)
                    .description(
                        "Your payment of $29.99 has been processed. A receipt has been sent to your email address."
                    ),
                alert::<Msg>("New feature available")
                    .icon(lucide::INFO)
                    .description(
                        "We've added dark mode support. You can enable it in your account settings."
                    ),
            ]
            .spacing(16)
            .max_width(448)
            .into()
        }),
        "destructive" => Fixture::new(|_| {
            column![
                alert::<Msg>("Payment failed")
                    .icon(lucide::CIRCLE_ALERT)
                    .variant(Variant::Destructive)
                    .description(
                        "Your payment could not be processed. Please check your payment method and try again."
                    )
            ]
            .max_width(448)
            .into()
        }),
        "action" => Fixture::new(|_| {
            column![
                alert::<Msg>("Dark mode is now available")
                    .description("Enable it under your profile settings to get started.")
                    .action(button("Enable").size(Size::Xs).on_press(Msg::Noop))
            ]
            .max_width(448)
            .into()
        }),
        _ => return None,
    })
}
