use super::Fixture;
use crate::store::Msg;

use shadcn_iced::resizable::{Axis, resizable};
use shadcn_iced::style::border;
use shadcn_iced::{font, tokens};

use iced::widget::{container, text};
use iced::{Element, Fill, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|_| {
            panels(
                resizable(
                    panel("One"),
                    resizable(panel("Two"), panel("Three"))
                        .axis(Axis::Vertical)
                        .default_ratio(0.25),
                )
                .default_ratio(0.5),
            )
        }),
        "vertical" => Fixture::new(|_| {
            panels(
                resizable(panel("Header"), panel("Content"))
                    .axis(Axis::Vertical)
                    .default_ratio(0.25),
            )
        }),
        "handle" | "keyboard" => Fixture::new(|_| {
            panels(
                resizable(panel("Sidebar"), panel("Content"))
                    .default_ratio(0.25)
                    .grip(true),
            )
        }),
        _ => return None,
    })
}

/// `max-w-sm rounded-lg border`, 200px of content plus the border.
fn panels<'a>(content: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    container(content)
        .width(Fill)
        .max_width(384)
        .height(202)
        .padding(1)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                border: border(t.palette.border, t.radius_lg()),
                ..container::Style::default()
            }
        })
        .into()
}

/// `flex items-center justify-center p-6`, `font-semibold`
fn panel(label: &'static str) -> Element<'static, Msg> {
    container(text(label).font(font::semibold()))
        .center(Fill)
        .padding(24)
        .into()
}
