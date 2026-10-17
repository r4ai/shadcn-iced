use super::Fixture;
use crate::store::Msg;

use shadcn_iced::avatar::{Size as AvatarSize, avatar, avatar_group};
use shadcn_iced::button::{self, Size, button, icon_button};
use shadcn_iced::item::{self, Item, Variant, item};
use shadcn_iced::prelude::lucide;
use shadcn_iced::separator::separator;

use iced::widget::{column, container};
use iced::{Element, Padding};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "demo" => Fixture::new(|_| {
            column![
                item::<Msg>("Basic Item")
                    .description("A simple item with title and description.")
                    .variant(Variant::Outline)
                    .action(
                        button("Action")
                            .variant(button::Variant::Outline)
                            .size(Size::Sm)
                            .on_press(Msg::Noop)
                    ),
                item::<Msg>("Your profile has been verified.")
                    .media(lucide::BADGE_CHECK.sized(20))
                    .variant(Variant::Outline)
                    .size(item::Size::Sm)
                    .action(lucide::CHEVRON_RIGHT.sized(16))
                    .on_press(Msg::Noop),
            ]
            .spacing(24)
            .max_width(448)
            .into()
        }),
        "variants" => Fixture::new(|_| {
            column![
                inbox(
                    "Default Variant",
                    "Transparent background with no border.",
                    Variant::Default,
                    item::Size::Default
                ),
                inbox(
                    "Outline Variant",
                    "Outlined style with a visible border.",
                    Variant::Outline,
                    item::Size::Default
                ),
                inbox(
                    "Muted Variant",
                    "Muted background for secondary content.",
                    Variant::Muted,
                    item::Size::Default
                ),
            ]
            .spacing(24)
            .max_width(448)
            .into()
        }),
        "sizes" => Fixture::new(|_| {
            column![
                inbox(
                    "Default Size",
                    "The standard size for most use cases.",
                    Variant::Outline,
                    item::Size::Default
                ),
                inbox(
                    "Small Size",
                    "A compact size for dense layouts.",
                    Variant::Outline,
                    item::Size::Sm
                ),
                inbox(
                    "Extra Small Size",
                    "The most compact size available.",
                    Variant::Outline,
                    item::Size::Xs
                ),
            ]
            .spacing(24)
            .max_width(448)
            .into()
        }),
        "avatar" => Fixture::new(|_| {
            column![
                item::<Msg>("Evil Rabbit")
                    .media(avatar("ER").size(AvatarSize::Lg))
                    .description("Last seen 5 months ago")
                    .variant(Variant::Outline)
                    .action(
                        icon_button(lucide::PLUS)
                            .size(Size::IconSm)
                            .variant(button::Variant::Outline)
                            .rounded(true)
                            .on_press(Msg::Noop)
                    ),
                item::<Msg>("No Team Members")
                    .media(avatar_group([avatar("CN"), avatar("LR"), avatar("ER")]))
                    .description("Invite your team to collaborate on this project.")
                    .variant(Variant::Outline)
                    .action(
                        button("Invite")
                            .size(Size::Sm)
                            .variant(button::Variant::Outline)
                            .on_press(Msg::Noop)
                    ),
            ]
            .spacing(24)
            .max_width(512)
            .into()
        }),
        // `ItemGroup` (`gap-4`) with an `ItemSeparator` (`my-2`) between
        // the items.
        "group" => Fixture::new(|_| {
            let people = [
                ("shadcn", "shadcn@vercel.com"),
                ("maxleiter", "maxleiter@vercel.com"),
                ("evilrabbit", "evilrabbit@vercel.com"),
            ];
            let last = people.len() - 1;

            let mut group = column![].spacing(16).max_width(384);

            for (index, (name, email)) in people.into_iter().enumerate() {
                group = group.push(person(name, email));

                if index != last {
                    group = group.push(container(separator()).padding(Padding::from([8, 0])));
                }
            }

            group.into()
        }),
        _ => return None,
    })
}

/// An item with an inbox icon (`<ItemMedia variant="icon">`).
fn inbox(
    title: &'static str,
    description: &'static str,
    variant: Variant,
    size: item::Size,
) -> Item<'static, Msg> {
    item(title)
        .icon(lucide::INBOX)
        .description(description)
        .variant(variant)
        .size(size)
}

fn person(name: &'static str, email: &'static str) -> Element<'static, Msg> {
    item(name)
        .media(avatar(&name[..1]))
        .description(email)
        .variant(Variant::Outline)
        .action(
            icon_button(lucide::PLUS)
                .variant(button::Variant::Ghost)
                .rounded(true)
                .on_press(Msg::Noop),
        )
        .into()
}
