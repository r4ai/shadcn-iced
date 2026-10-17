use super::Fixture;
use crate::store::Msg;

use shadcn_iced::collapsible::collapsible;
use shadcn_iced::style::border;
use shadcn_iced::{font, tokens};

use iced::widget::{column, container, text};
use iced::{Element, Fill, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "keyboard" => Fixture::new(|_| repositories(false)),
        "default-open" => Fixture::new(|_| repositories(true)),
        _ => return None,
    })
}

/// `w-[350px]`
fn repositories<'a>(open: bool) -> Element<'a, Msg> {
    let content = column![
        repository("@radix-ui/colors"),
        repository("@stitches/react")
    ]
    .spacing(8);

    container(collapsible("@peduarte starred 3 repositories", content).default_open(open))
        .width(350)
        .into()
}

/// `rounded-md border px-4 py-2 font-mono text-sm`
fn repository<'a>(name: &'a str) -> Element<'a, Msg> {
    container(
        text(name)
            .size(14)
            .font(font::mono())
            .line_height(iced::Pixels(20.0)),
    )
    // `px-4 py-2` inside the `border`
    .padding([9, 17])
    .width(Fill)
    .style(|theme: &Theme| {
        let t = tokens(theme);

        container::Style {
            border: border(t.palette.border, t.radius_md()),
            ..container::Style::default()
        }
    })
    .into()
}
