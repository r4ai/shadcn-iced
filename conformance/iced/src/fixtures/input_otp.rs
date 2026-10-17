use super::Fixture;
use crate::store::set;

use shadcn_iced::input_otp::input_otp;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            input_otp(6, store.str("code"), set("code"))
                .groups(&[3, 3])
                .into()
        }),
        "value" => Fixture::new(|store| input_otp(4, store.str("code"), set("code")).into()).init(
            |store| {
                store.set("code", "12");
            },
        ),
        "invalid" => Fixture::new(|store| {
            input_otp(6, store.str("code"), set("code"))
                .groups(&[3, 3])
                .invalid(true)
                .into()
        })
        .init(|store| {
            store.set("code", "123");
        }),
        "disabled" => Fixture::new(|store| {
            input_otp(6, store.str("code"), set("code"))
                .groups(&[3, 3])
                .disabled(true)
                .into()
        })
        .init(|store| {
            store.set("code", "123");
        }),
        _ => return None,
    })
}
