//! Styles for headings, paragraphs, lists...
//!
//! Port of <https://ui.shadcn.com/docs/components/typography>.
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Text, container, text};
use iced::{Background, Font, Length, Theme, border};

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// `text-4xl font-extrabold tracking-tight`
///
/// Drawn bold (the heaviest bundled weight) and without tracking, which iced
/// does not support.
pub fn h1<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL4)
        .font(font::bold())
        .line_height(style::text::leading(style::text::XL4))
}

/// `text-3xl font-semibold tracking-tight`
pub fn h2<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL3)
        .font(font::semibold())
        .line_height(style::text::leading(style::text::XL3))
}

/// `text-2xl font-semibold tracking-tight`
pub fn h3<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL2)
        .font(font::semibold())
        .line_height(style::text::leading(style::text::XL2))
}

/// `text-xl font-semibold tracking-tight`
pub fn h4<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL)
        .font(font::semibold())
        .line_height(style::text::leading(style::text::XL))
}

/// `leading-7`
pub fn p<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::BASE)
        .line_height(iced::Pixels(28.0))
}

/// `text-xl text-muted-foreground`
pub fn lead<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL)
        .line_height(style::text::leading(style::text::XL))
        .style(muted)
}

/// `text-lg font-semibold`
pub fn large<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::LG)
        .font(font::semibold())
        .line_height(style::text::leading(style::text::LG))
}

/// `text-sm leading-none font-medium`
pub fn small<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::SM)
        .font(font::medium())
        .line_height(iced::Pixels(14.0))
}

/// `text-sm text-muted-foreground`
pub fn muted_text<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::SM)
        .line_height(style::text::leading(style::text::SM))
        .style(muted)
}

/// Inline code: `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`
///
/// Drawn with the medium weight, the heaviest bundled weight of Geist Mono.
pub fn inline_code<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(
        text(content)
            .size(style::text::SM)
            .line_height(style::text::leading(style::text::SM))
            .font(font::weight(font::mono(), iced::font::Weight::Medium)),
    )
    .padding([3.2, 4.8])
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(tokens(theme).palette.muted)),
        border: border::rounded(4),
        ..container::Style::default()
    })
    .into()
}

/// `border-l-2 pl-6 italic`, with the line height of the page (24px).
///
/// Geist has no italic face: unless one is loaded, the text stays upright
/// (iced does not synthesize oblique faces).
pub fn blockquote<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    Row::new()
        .push(
            container(iced::widget::space())
                .width(2)
                .height(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(tokens(theme).palette.border)),
                    ..container::Style::default()
                }),
        )
        .push(
            container(
                text(content)
                    .size(style::text::BASE)
                    .line_height(style::text::leading(style::text::BASE))
                    .font(Font {
                        style: iced::font::Style::Italic,
                        ..font::sans()
                    }),
            )
            .padding(iced::Padding::ZERO.left(24)),
        )
        .height(Length::Shrink)
        .into()
}

/// A bulleted list (`ml-6 list-disc [&>li]:mt-2`).
///
/// Like `list-style-position: outside`, the 5px discs sit in the left
/// margin, 14px before the center of the first line of their items.
pub fn list<'a, Message: 'a>(
    items: impl IntoIterator<Item = impl text::IntoFragment<'a>>,
) -> Element<'a, Message> {
    let line_height = style::text::leading(style::text::BASE);

    Column::with_children(items.into_iter().map(|item| {
        let disc = container(container(iced::widget::space().width(5).height(5)).style(
            |theme: &Theme| container::Style {
                background: Some(Background::Color(tokens(theme).palette.foreground)),
                border: border::rounded(2.5),
                ..container::Style::default()
            },
        ))
        .center_y(24);

        Row::new()
            .push(disc)
            .push(text(item).size(style::text::BASE).line_height(line_height))
            .spacing(11.75)
            .into()
    }))
    // `[&>li]:mt-2`, the first item included.
    .spacing(8)
    .padding(iced::Padding::ZERO.top(8).left(7.25))
    .into()
}
