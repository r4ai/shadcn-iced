use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::combobox::combobox;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "basic" | "filter" | "empty" | "keyboard" => Fixture::new(basic),
        _ => return None,
    })
}

fn basic(s: &Store) -> Element<'_, Msg> {
    combobox(
        s.opt("framework"),
        set("framework"),
        s.str("query"),
        set("query"),
    )
    .placeholder("Select a framework")
    .empty("No items found.")
    .option("Next.js", "Next.js")
    .option("SvelteKit", "SvelteKit")
    .option("Nuxt.js", "Nuxt.js")
    .option("Remix", "Remix")
    .option("Astro", "Astro")
    // `w-64`
    .width(256.0)
    .into()
}
