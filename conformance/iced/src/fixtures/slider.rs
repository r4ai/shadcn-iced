use super::Fixture;
use crate::store::{Msg, Value};

use shadcn_iced::slider::{range_slider, slider, vertical};
use shadcn_iced::tokens;

use iced::Theme;
use iced::widget::{column, text};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|store| {
            slider(0.0..=100.0, store.num("value"), set_value)
                .step(1.0)
                .into()
        })
        .init(|store| {
            store.set("value", 50.0f32);
        }),
        "controlled" => Fixture::new(|store| {
            let value = store.num("value");

            column![
                slider(0.0..=100.0, value, set_value).step(1.0),
                text(format!("Value: {value}"))
                    .size(14)
                    .line_height(iced::Pixels(20.0))
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            ]
            .spacing(12)
            .into()
        })
        .init(|store| {
            store.set("value", 50.0f32);
        }),
        "range" => Fixture::new(|store| {
            range_slider(
                0.0..=100.0,
                &[store.num("low"), store.num("high")],
                |index, value| Msg::Set(if index == 0 { "low" } else { "high" }, Value::Num(value)),
            )
            .step(1.0)
            .into()
        })
        .init(|store| {
            store.set("low", 25.0f32).set("high", 75.0f32);
        }),
        "vertical" => Fixture::new(|store| {
            vertical(0.0..=100.0, store.num("value"), set_value)
                .step(1.0)
                .height(160)
                .into()
        })
        .init(|store| {
            store.set("value", 50.0f32);
        }),
        "disabled" => Fixture::new(|_| {
            slider(0.0..=100.0, 50.0, |_| Msg::Noop)
                .step(1.0)
                .disabled(true)
                .into()
        }),
        _ => return None,
    })
}

fn set_value(value: f32) -> Msg {
    Msg::Set("value", Value::Num(value))
}
