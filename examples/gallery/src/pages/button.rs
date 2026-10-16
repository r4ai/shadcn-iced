use super::{DemoMessage, Page};
use crate::{Gallery, Message, example};

use iced_ui::button::{Size, Variant, button, icon_button};
use iced_ui::prelude::lucide;

use iced::widget::{column, row};
use iced::{Center, Element};

pub const PAGE: Page = Page {
    id: "button",
    title: "Button",
    description: "Displays a button or a component that looks like a button.",
    view,
};

fn view(_gallery: &Gallery) -> Element<'_, Message> {
    let click = Message::Demo(DemoMessage::Clicked);

    column![
        example(
            "Variants",
            row![
                button("Button").on_press(click.clone()),
                button("Secondary")
                    .variant(Variant::Secondary)
                    .on_press(click.clone()),
                button("Destructive")
                    .variant(Variant::Destructive)
                    .on_press(click.clone()),
                button("Outline")
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Ghost")
                    .variant(Variant::Ghost)
                    .on_press(click.clone()),
                button("Link")
                    .variant(Variant::Link)
                    .on_press(click.clone()),
            ]
            .spacing(8)
            .align_y(Center)
            .wrap()
        ),
        example(
            "Sizes",
            row![
                button("Extra small")
                    .size(Size::Xs)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Small")
                    .size(Size::Sm)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Default")
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Large")
                    .size(Size::Lg)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
            ]
            .spacing(8)
            .align_y(Center)
        ),
        example(
            "With icon",
            row![
                button("New branch")
                    .icon(lucide::GIT_BRANCH)
                    .variant(Variant::Outline)
                    .size(Size::Sm)
                    .on_press(click.clone()),
                button("Login with Email")
                    .icon(lucide::MAIL)
                    .on_press(click.clone()),
                button("Next")
                    .trailing_icon(lucide::CHEVRON_RIGHT)
                    .variant(Variant::Secondary)
                    .on_press(click.clone()),
                button("Please wait")
                    .icon(lucide::LOADER_CIRCLE)
                    .disabled(true),
            ]
            .spacing(8)
            .align_y(Center)
            .wrap()
        ),
        example(
            "Icon",
            row![
                icon_button(lucide::CIRCLE_FADING_ARROW_UP)
                    .variant(Variant::Outline)
                    .size(Size::IconXs)
                    .on_press(click.clone()),
                icon_button(lucide::CIRCLE_FADING_ARROW_UP)
                    .variant(Variant::Outline)
                    .size(Size::IconSm)
                    .on_press(click.clone()),
                icon_button(lucide::CIRCLE_FADING_ARROW_UP)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                icon_button(lucide::CIRCLE_FADING_ARROW_UP)
                    .variant(Variant::Outline)
                    .size(Size::IconLg)
                    .on_press(click.clone()),
            ]
            .spacing(8)
            .align_y(Center)
        ),
        example(
            "Disabled",
            row![
                button("Disabled").disabled(true),
                button("Disabled").variant(Variant::Outline).disabled(true),
                button("Disabled")
                    .variant(Variant::Secondary)
                    .disabled(true),
            ]
            .spacing(8)
        ),
    ]
    .into()
}
