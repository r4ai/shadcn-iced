use super::{DemoMessage, Page};
use crate::{Gallery, Message, example};

use shadcn_iced::button::{Button, Size, Variant, button, icon_button};
use shadcn_iced::prelude::{lucide, spinner};
use shadcn_iced::theme::alpha;

use iced::widget::{column, row, text};
use iced::{Center, Color, Element, Theme};

pub const PAGE: Page = Page {
    id: "button",
    title: "Button",
    description: "Displays a button or a component that looks like a button.",
    view,
};

/// A disabled button with a spinner (`<Spinner data-icon="inline-start" />`).
fn loading<'a>(
    label: &'a str,
    variant: Variant,
    trailing: bool,
    color: fn(&Theme) -> Color,
) -> Button<'a, Message> {
    let spinner = spinner().style(color);
    let label = text(label).size(14).font(shadcn_iced::font::medium());

    let content = if trailing {
        row![label, spinner]
    } else {
        row![spinner, label]
    };

    Button::new(content.spacing(6).align_y(Center))
        .variant(variant)
        .disabled(true)
}

fn view(_gallery: &Gallery) -> Element<'_, Message> {
    let click = Message::Demo(DemoMessage::Clicked);

    let pair = |label, size, icon_size| {
        row![
            button(label)
                .size(size)
                .variant(Variant::Outline)
                .on_press(click.clone()),
            icon_button(lucide::ARROW_UP_RIGHT)
                .size(icon_size)
                .variant(Variant::Outline)
                .on_press(click.clone()),
        ]
        .spacing(8)
    };

    column![
        example(
            "Demo",
            row![
                button("Button")
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                icon_button(lucide::ARROW_UP)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
            ]
            .spacing(8)
            .align_y(Center)
        ),
        example(
            "Size",
            row![
                pair("Extra Small", Size::Xs, Size::IconXs),
                pair("Small", Size::Sm, Size::IconSm),
                pair("Default", Size::Default, Size::Icon),
                pair("Large", Size::Lg, Size::IconLg),
            ]
            .spacing(32)
            .wrap()
        ),
        example(
            "Variants",
            row![
                button("Button").on_press(click.clone()),
                button("Outline")
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Secondary")
                    .variant(Variant::Secondary)
                    .on_press(click.clone()),
                button("Ghost")
                    .variant(Variant::Ghost)
                    .on_press(click.clone()),
                button("Destructive")
                    .variant(Variant::Destructive)
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
            "Icon",
            icon_button(lucide::CIRCLE_FADING_ARROW_UP)
                .variant(Variant::Outline)
                .on_press(click.clone())
        ),
        example(
            "With Icon",
            row![
                button("New Branch")
                    .icon(lucide::GIT_BRANCH)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
                button("Fork")
                    .trailing_icon(lucide::GIT_FORK)
                    .variant(Variant::Outline)
                    .on_press(click.clone()),
            ]
            .spacing(8)
        ),
        example(
            "Rounded",
            row![
                button("Get Started").rounded(true).on_press(click.clone()),
                icon_button(lucide::ARROW_UP)
                    .variant(Variant::Outline)
                    .rounded(true)
                    .on_press(click.clone()),
            ]
            .spacing(8)
        ),
        example(
            "Spinner",
            row![
                // `disabled:opacity-50`
                loading("Generating", Variant::Outline, false, |theme| {
                    alpha(shadcn_iced::tokens(theme).palette.foreground, 0.5)
                }),
                loading("Downloading", Variant::Secondary, true, |theme| {
                    alpha(shadcn_iced::tokens(theme).palette.secondary_foreground, 0.5)
                }),
            ]
            .spacing(8)
        ),
    ]
    .spacing(24)
    .into()
}
