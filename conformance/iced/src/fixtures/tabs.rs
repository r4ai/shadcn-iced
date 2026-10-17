use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::prelude::lucide;
use shadcn_iced::tabs::{Orientation, Variant, tabs};
use shadcn_iced::tokens;

use iced::widget::{container, text};
use iced::{Element, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "keyboard" | "tab" => Fixture::new(account).init(|s| {
            s.set("tab", "account");
        }),
        "line" => Fixture::new(|s| {
            let content = match s.str("tab") {
                "analytics" => "Track performance and engagement.",
                "reports" => "Generate your reports.",
                _ => "View your key metrics.",
            };

            tabs(s.str("tab"), set("tab"))
                .variant(Variant::Line)
                .tab("overview", "Overview")
                .tab("analytics", "Analytics")
                .tab("reports", "Reports")
                .content(muted(content))
                .into()
        })
        .init(|s| {
            s.set("tab", "overview");
        }),
        "vertical" => Fixture::new(|s| {
            let content = match s.str("tab") {
                "password" => "Change your password here.",
                "notifications" => "Choose what you get notified about.",
                _ => "Update your account details.",
            };

            tabs(s.str("tab"), set("tab"))
                .orientation(Orientation::Vertical)
                .tab("account", "Account")
                .tab("password", "Password")
                .tab("notifications", "Notifications")
                .content(muted(content))
                .into()
        })
        .init(|s| {
            s.set("tab", "account");
        }),
        "disabled" => Fixture::new(|s| {
            let content = match s.str("tab") {
                "settings" => "Disabled content.",
                _ => "Home content.",
            };

            tabs(s.str("tab"), set("tab"))
                .tab("home", "Home")
                .disabled_tab("settings", "Disabled")
                .content(muted(content))
                .into()
        })
        .init(|s| {
            s.set("tab", "home");
        }),
        "icons" => Fixture::new(|s| {
            let content = match s.str("tab") {
                "code" => "Source code.",
                _ => "A live preview.",
            };

            tabs(s.str("tab"), set("tab"))
                .tab_with_icon("preview", lucide::APP_WINDOW, "Preview")
                .tab_with_icon("code", lucide::CODE, "Code")
                .content(muted(content))
                .into()
        })
        .init(|s| {
            s.set("tab", "preview");
        }),
        _ => return None,
    })
}

/// `w-[400px]`, two tabs.
fn account(s: &Store) -> Element<'_, Msg> {
    let content = match s.str("tab") {
        "password" => "Change your password here.",
        _ => "Make changes to your account here.",
    };

    container(
        tabs(s.str("tab"), set("tab"))
            .tab("account", "Account")
            .tab("password", "Password")
            .content(muted(content)),
    )
    .width(400)
    .into()
}

/// `text-sm text-muted-foreground`
fn muted(content: &str) -> iced::widget::Text<'_> {
    text(content)
        .size(14)
        .line_height(iced::Pixels(20.0))
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
}
