use super::Fixture;
use crate::store::{Msg, set};

use shadcn_iced::field::{Orientation, field, field_group, field_separator, field_set};
use shadcn_iced::input::input;
use shadcn_iced::switch::switch;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            narrow({
                field(
                    "Username",
                    input("Max Leiter", store.str("username")).on_input(set("username")),
                )
                .description("Choose a unique username for your account.")
                .into()
            })
        }),
        "error" => Fixture::new(|store| {
            narrow({
                field(
                    "Username",
                    input("Max Leiter", store.str("username"))
                        .on_input(set("username"))
                        .invalid(true),
                )
                .error("Username is already taken.")
                .into()
            })
        }),
        "disabled" => Fixture::new(|_| {
            narrow({
                field("Username", input::<Msg>("Max Leiter", ""))
                    .description("Choose a unique username for your account.")
                    .disabled(true)
                    .into()
            })
        }),
        "horizontal" => Fixture::new(|store| {
            narrow({
                field(
                    "Multi-factor authentication",
                    switch(store.bool("mfa")).on_toggle(set("mfa")),
                )
                .description("Enable multi-factor authentication.")
                .orientation(Orientation::Responsive)
                .into()
            })
        }),
        "fieldset" => Fixture::new(|store| {
            narrow({
                field_set("Address Information")
                    .description("We need your address to deliver your order.")
                    .push(field(
                        "Street Address",
                        input("123 Main St", store.str("street")).on_input(set("street")),
                    ))
                    .push(field(
                        "City",
                        input("New York", store.str("city")).on_input(set("city")),
                    ))
                    .into()
            })
        }),
        "separator" => Fixture::new(|store| {
            narrow({
                field_group([
                    field(
                        "Email",
                        input("m@example.com", store.str("email")).on_input(set("email")),
                    )
                    .into(),
                    field_separator(Some("Or continue with")),
                    field(
                        "Phone",
                        input("+1 555 0100", store.str("phone")).on_input(set("phone")),
                    )
                    .into(),
                ])
                .into()
            })
        }),
        _ => return None,
    })
}

/// The page of the fixture is 768px wide (the `md` breakpoint), in a `w-80`.
fn narrow(content: iced::Element<'_, Msg>) -> iced::Element<'_, Msg> {
    iced::widget::container(content).width(320).into()
}
