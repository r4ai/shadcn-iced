use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::prelude::lucide;
use shadcn_iced::toggle::{Size, Variant};
use shadcn_iced::toggle_group::{Orientation, ToggleGroup, toggle_group};
use shadcn_iced::tokens;

use iced::widget::{column, row, text};
use iced::{Center, Theme};

const FORMATS: [&str; 3] = ["bold", "italic", "underline"];

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "multiple" => Fixture::new(|store| {
            formatting(store)
                .variant(Variant::Outline)
                .spacing(0.0)
                .into()
        }),
        "single" => Fixture::new(|store| {
            let selected = store.opt("align");
            let on: Vec<&'static str> = ["left", "center", "right"]
                .into_iter()
                .filter(|value| Some(*value) == selected)
                .collect();

            // Pressing the pressed item turns it off, like Base UI does.
            let current = selected.map(str::to_owned);

            column![
                toggle_group(&on, move |value: &'static str| {
                    if current.as_deref() == Some(value) {
                        Msg::Set("align", Value::None)
                    } else {
                        Msg::Set("align", Value::from(value))
                    }
                })
                .label("left", "Left")
                .label("center", "Center")
                .label("right", "Right")
                .variant(Variant::Outline)
                .spacing(0.0),
                text(format!("Selected: {}", selected.unwrap_or("none")))
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            ]
            .spacing(12)
            .into()
        }),
        "sizes" => Fixture::new(|store| {
            let outline = |size| {
                formatting(store)
                    .variant(Variant::Outline)
                    .size(size)
                    .spacing(0.0)
            };

            row![outline(Size::Sm), outline(Size::Default), outline(Size::Lg),]
                .spacing(16)
                .align_y(Center)
                .into()
        }),
        "spacing" => {
            Fixture::new(|store| formatting(store).variant(Variant::Outline).into()).init(|store| {
                store.set("bold", true);
            })
        }
        "vertical" => Fixture::new(|store| {
            formatting(store)
                .variant(Variant::Outline)
                .spacing(0.0)
                .orientation(Orientation::Vertical)
                .into()
        })
        .init(|store| {
            store.set("italic", true);
        }),
        "disabled" => Fixture::new(|store| formatting(store).disabled(true).into()).init(|store| {
            store.set("bold", true);
        }),
        _ => return None,
    })
}

/// Bold, italic and underline, any of which can be on.
fn formatting(store: &Store) -> ToggleGroup<'_, &'static str, Msg> {
    let on: Vec<&'static str> = FORMATS
        .into_iter()
        .filter(|format| store.bool(format))
        .collect();

    toggle_group(&on, Msg::Toggle)
        .icon("bold", lucide::BOLD)
        .icon("italic", lucide::ITALIC)
        .icon("underline", lucide::UNDERLINE)
}
