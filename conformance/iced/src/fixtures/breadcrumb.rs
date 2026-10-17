use super::Fixture;
use crate::store::Msg;

use shadcn_iced::breadcrumb::breadcrumb;
use shadcn_iced::prelude::lucide;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            breadcrumb()
                .link("Home", Msg::Noop)
                .link("Components", Msg::Noop)
                .page("Breadcrumb")
                .into()
        }),
        "ellipsis" => Fixture::new(|_| {
            breadcrumb()
                .link("Home", Msg::Noop)
                .ellipsis()
                .link("Components", Msg::Noop)
                .page("Breadcrumb")
                .into()
        }),
        "custom-separator" => Fixture::new(|_| {
            breadcrumb()
                .separator(lucide::DOT)
                .link("Home", Msg::Noop)
                .link("Components", Msg::Noop)
                .page("Breadcrumb")
                .into()
        }),
        _ => return None,
    })
}
