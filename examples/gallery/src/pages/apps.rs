//! The example apps of the website (`/examples`): mock apps built from the
//! components, each a crate of its own that also runs on its own
//! (`cargo run -p dashboard`).
use super::Page;
use crate::{Message, example};

use shadcn_iced::prelude::scroll_area;

use iced::Fill;

pub const DASHBOARD: Page = Page {
    id: "dashboard",
    title: "Dashboard",
    description: "The dashboard-01 block of shadcn/ui: an inset sidebar, metrics, an interactive chart and a data table.",
    view: |gallery| {
        example(
            "Dashboard",
            gallery.dashboard.view().map(Message::Dashboard),
        )
    },
};

pub const SETTINGS: Page = Page {
    id: "settings",
    title: "Settings",
    description: "The settings of a fictional app: a profile, an account, the appearance and the notifications.",
    // Scrolled in a frame of a fixed height, like `cargo run -p settings`:
    // grown to its natural height, the preview would change its height, and
    // move the page below it, with each section.
    view: |gallery| {
        example(
            "Settings",
            scroll_area(gallery.settings.view().map(Message::Settings)).height(Fill),
        )
    },
};
