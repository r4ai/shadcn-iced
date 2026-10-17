use super::Fixture;
use crate::store::{Msg, Store, set};

use shadcn_iced::switch::{Size, switch};
use shadcn_iced::tokens;

use iced::widget::{column, row, text};
use iced::{Center, Element, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            switch(store.bool("checked"))
                .label("Airplane Mode")
                .on_toggle(set("checked"))
                .into()
        }),
        "sizes" => Fixture::new(|_| {
            let on_toggle = |_| Msg::Noop;

            column![
                row![
                    switch(false).size(Size::Sm).on_toggle(on_toggle),
                    switch(true).size(Size::Sm).on_toggle(on_toggle),
                ]
                .spacing(16)
                .align_y(Center),
                row![
                    switch(false).on_toggle(on_toggle),
                    switch(true).on_toggle(on_toggle),
                ]
                .spacing(16)
                .align_y(Center),
            ]
            .spacing(12)
            .into()
        }),
        "states" => Fixture::new(|_| {
            column![
                switch::<Msg>(false).label("Disabled").disabled(true),
                switch::<Msg>(true).label("Disabled checked").disabled(true),
                switch(false)
                    .label("Invalid")
                    .invalid(true)
                    .on_toggle(|_| Msg::Noop),
                switch(true)
                    .label("Invalid checked")
                    .invalid(true)
                    .on_toggle(|_| Msg::Noop),
            ]
            .spacing(12)
            .into()
        }),
        "keyboard" => Fixture::new(controlled),
        _ => return None,
    })
}

fn controlled(store: &Store) -> Element<'_, Msg> {
    let checked = store.bool("checked");

    column![
        switch(checked)
            .label("Airplane Mode")
            .on_toggle(set("checked")),
        text(if checked {
            "Airplane mode is on"
        } else {
            "Airplane mode is off"
        })
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        }),
    ]
    .spacing(12)
    .into()
}
