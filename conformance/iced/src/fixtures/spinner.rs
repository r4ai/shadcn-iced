use super::Fixture;
use crate::store::Msg;

use shadcn_iced::item::{Variant, item};
use shadcn_iced::primitive::typeset::{TABULAR_NUMS, typeset};
use shadcn_iced::spinner::spinner;

use iced::Center;
use iced::widget::{container, row};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "sizes" => Fixture::new(|_| {
            row![
                spinner().size(12.0),
                spinner(),
                spinner().size(24.0),
                spinner().size(32.0),
            ]
            .spacing(24)
            .align_y(Center)
            .into()
        }),
        "item" => Fixture::new(|_| {
            container(
                item::<Msg>("Processing payment...")
                    .media(spinner())
                    .action(
                        // `text-sm tabular-nums`
                        typeset("$100.00")
                            .size(14)
                            .line_height(iced::Pixels(20.0))
                            .feature(TABULAR_NUMS),
                    )
                    .variant(Variant::Muted),
            )
            .max_width(320)
            .into()
        }),
        _ => return None,
    })
}
