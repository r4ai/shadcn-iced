//! The example apps of the website (`/examples`): mock apps built from the
//! components, each a crate of its own that also runs on its own
//! (`cargo run -p dashboard`).
use super::Page;
use crate::{Message, example};

pub const DASHBOARD: Page = Page {
    id: "dashboard",
    title: "Dashboard",
    description: "The dashboard of a fictional store: metrics, charts, recent sales and invoices.",
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
    view: |gallery| example("Settings", gallery.settings.view().map(Message::Settings)),
};
