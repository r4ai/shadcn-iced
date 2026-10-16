//! Styles for headings, paragraphs, lists...
//!
//! Port of <https://ui.shadcn.com/docs/components/typography>.
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Text, container, text};
use iced::{Background, Font, Length, Theme, border};

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// `text-4xl font-extrabold tracking-tight`
pub fn h1<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL4)
        .font(font::bold())
        .line_height(1.11)
}

/// `text-3xl font-semibold tracking-tight`
pub fn h2<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL3)
        .font(font::semibold())
        .line_height(1.2)
}

/// `text-2xl font-semibold tracking-tight`
pub fn h3<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL2)
        .font(font::semibold())
        .line_height(1.33)
}

/// `text-xl font-semibold tracking-tight`
pub fn h4<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::XL)
        .font(font::semibold())
        .line_height(1.4)
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
        .line_height(1.4)
        .style(muted)
}

/// `text-lg font-semibold`
pub fn large<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(style::text::LG).font(font::semibold())
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
    text(content).size(style::text::SM).style(muted)
}

/// Inline code: `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`
pub fn inline_code<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(content).size(style::text::SM).font(Font {
        weight: iced::font::Weight::Semibold,
        ..iced_ui_core::font::mono()
    }))
    .padding([3, 5])
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(tokens(theme).palette.muted)),
        border: border::rounded(4),
        ..container::Style::default()
    })
    .into()
}

/// `border-l-2 pl-6 italic`
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
        .push(container(p(content)).padding(iced::Padding::ZERO.left(24)))
        .height(Length::Shrink)
        .into()
}

/// A bulleted list (`ml-6 list-disc [&>li]:mt-2`).
pub fn list<'a, Message: 'a>(
    items: impl IntoIterator<Item = impl text::IntoFragment<'a>>,
) -> Element<'a, Message> {
    Column::with_children(items.into_iter().map(|item| {
        Row::new()
            .push(
                text("•")
                    .size(style::text::BASE)
                    .line_height(iced::Pixels(24.0)),
            )
            .push(
                text(item)
                    .size(style::text::BASE)
                    .line_height(iced::Pixels(24.0)),
            )
            .spacing(10)
            .into()
    }))
    .spacing(8)
    .padding(iced::Padding::ZERO.left(24))
    .into()
}
