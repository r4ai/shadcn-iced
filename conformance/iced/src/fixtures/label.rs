use super::Fixture;
use crate::store::{Msg, set};

use shadcn_iced::checkbox::checkbox;
use shadcn_iced::input::input;
use shadcn_iced::label::{label, labelled};
use shadcn_iced::tokens;

use iced::widget::{column, row, text};
use iced::{Center, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        // `htmlFor` associates the labels with their controls.
        "default" => Fixture::new(|store| {
            labelled(
                row![
                    checkbox(store.bool("checked")).on_toggle(set("checked")),
                    label("Accept terms and conditions"),
                ]
                .spacing(12)
                .align_y(Center),
            )
            .into()
        }),
        "with-input" => Fixture::new(|store| {
            narrow({
                let value = store.str("email");

                column![
                    labelled(
                        column![label("Email"), input("Email", value).on_input(set("email"))]
                            .spacing(12)
                    ),
                    (!value.is_empty()).then(|| {
                        text(format!("Typed: {value}"))
                            .size(14)
                            .style(|theme: &Theme| text::Style {
                                color: Some(tokens(theme).palette.muted_foreground),
                            })
                    }),
                ]
                .spacing(12)
                .into()
            })
        }),
        "disabled" => Fixture::new(|_| {
            labelled(
                row![
                    checkbox::<Msg>(false).disabled(true),
                    label("Accept terms and conditions"),
                ]
                .spacing(12)
                .align_y(Center),
            )
            .into()
        }),
        _ => return None,
    })
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: iced::Element<'_, Msg>) -> iced::Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
