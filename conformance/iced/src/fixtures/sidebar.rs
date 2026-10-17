use super::Fixture;
use crate::store::{Msg, Store};

use shadcn_iced::prelude::lucide;
use shadcn_iced::separator::vertical_separator;
use shadcn_iced::sidebar::{Item, Size, group, item, layout, sidebar, trigger};
use shadcn_iced::style::{border, rounded};
use shadcn_iced::{Icon, font, tokens};

use iced::widget::canvas::{self, Frame, Path};
use iced::widget::{Canvas, column, container, row, space, stack, text};
use iced::{Center, Element, Fill, Theme};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "collapse" => Fixture::new(view).init(|s| {
            s.set("active", "Playground");
            s.set("collapsed", false);
        }),
        _ => return None,
    })
}

/// A menu button that becomes the active one when pressed.
fn button(s: &Store, title: &'static str, icon: Icon) -> Item<'static, Msg> {
    item(title)
        .icon(icon)
        .active(s.str("active") == title)
        .on_press(Msg::Set("active", title.into()))
}

fn view(s: &Store) -> Element<'_, Msg> {
    let collapsed = s.bool("collapsed");

    let nav = sidebar()
        .header_item(
            item("Acme Inc")
                .size(Size::Lg)
                .icon_element(logo(lucide::GALLERY_VERTICAL_END))
                .content(two_lines(
                    "Acme Inc",
                    "Enterprise",
                    lucide::CHEVRONS_UP_DOWN,
                ))
                .on_press(Msg::Noop),
        )
        .group(
            group("Platform")
                .item(button(s, "Playground", lucide::SQUARE_TERMINAL))
                .item(button(s, "Models", lucide::BOT))
                .item(button(s, "Documentation", lucide::BOOK_OPEN))
                .item(button(s, "Settings", lucide::SETTINGS_2)),
        )
        .group(
            group("Projects")
                .item(button(s, "Design Engineering", lucide::FRAME))
                .item(button(s, "Sales & Marketing", lucide::CHART_PIE))
                .item(button(s, "Travel", lucide::MAP))
                .item(
                    item("More")
                        .icon(lucide::ELLIPSIS)
                        .badge("24")
                        .on_press(Msg::Set("active", "More".into())),
                ),
        )
        .footer_item(item("Support").icon(lucide::LIFE_BUOY).on_press(Msg::Noop))
        .collapsed(collapsed);

    // `flex h-12 items-center gap-2 px-4`
    let main = row![
        trigger(Msg::Toggle("collapsed")),
        container(vertical_separator()).height(16),
        text("Data Fetching").size(14),
    ]
    .spacing(8)
    .height(48)
    .padding([0, 16])
    .align_y(Center);

    // `h-[528px] overflow-hidden rounded-lg border`. iced only clips to
    // rectangles: the corners outside of the border are masked with the
    // page background, and the border is drawn over the content.
    let frame = container(space().width(Fill).height(Fill)).style(|theme: &Theme| {
        let t = tokens(theme);

        container::Style {
            border: border(t.palette.border, t.radius_lg()),
            ..container::Style::default()
        }
    });

    stack![
        container(layout(nav, main))
            .width(Fill)
            .height(Fill)
            .padding(1)
            .clip(true),
        Canvas::new(Corners).width(Fill).height(Fill),
        frame
    ]
    .width(Fill)
    .height(528)
    .into()
}

/// Two lines of text in a large sidebar item
/// (`grid flex-1 text-left text-sm leading-tight`).
fn two_lines(title: &'static str, subtitle: &'static str, trailing: Icon) -> Element<'static, Msg> {
    row![
        column![
            text(title).size(14).font(font::medium()).line_height(1.25),
            text(subtitle).size(12).line_height(1.25),
        ]
        .width(Fill),
        trailing.sized(16),
    ]
    .align_y(Center)
    .into()
}

/// The square logo of the team switcher (`size-8 rounded-lg
/// bg-sidebar-primary`).
fn logo(icon: Icon) -> Element<'static, Msg> {
    container(icon.sized(16))
        .center(32)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(t.palette.sidebar_primary.into()),
                text_color: Some(t.palette.sidebar_primary_foreground),
                border: rounded(t.radius_lg()),
                ..container::Style::default()
            }
        })
        .into()
}

/// Masks the corners of the content outside of a `rounded-lg` frame with
/// the page background.
struct Corners;

impl canvas::Program<Msg> for Corners {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use iced::{Point, Vector};
        use std::f32::consts::FRAC_PI_2;

        let mut frame = Frame::new(renderer, bounds.size());
        let r = tokens(theme).radius_lg();
        let (w, h) = (bounds.width, bounds.height);

        // Each corner, with the center of its arc and its starting angle.
        for (corner, center, start) in [
            (Point::ORIGIN, Point::new(r, r), 2.0),
            (Point::new(w, 0.0), Point::new(w - r, r), 3.0),
            (Point::new(w, h), Point::new(w - r, h - r), 0.0),
            (Point::new(0.0, h), Point::new(r, h - r), 1.0),
        ] {
            let path = Path::new(|builder| {
                builder.move_to(corner);

                for step in 0..=16 {
                    let angle = (start + step as f32 / 16.0) * FRAC_PI_2;

                    builder.line_to(center + Vector::new(angle.cos(), angle.sin()) * r);
                }

                builder.close();
            });

            frame.fill(&path, tokens(theme).palette.background);
        }

        vec![frame.into_geometry()]
    }
}
