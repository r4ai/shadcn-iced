//! Styles for headings, paragraphs, lists...
//!
//! Port of <https://ui.shadcn.com/docs/components/typography>.
use shadcn_iced_core::primitive::typeset::{Typeset, typeset};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::{Column, Row, Text, container, text};
use iced::{Background, Event, Font, Length, Rectangle, Size, Theme, Vector, border, mouse};

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// A heading of the given size and font, with `tracking-tight`
/// (`-0.025em`).
fn heading<'a>(content: impl text::IntoFragment<'a>, size: f32, font: Font) -> Typeset<'a> {
    typeset(content.into_fragment().into_owned())
        .size(size)
        .font(font)
        .line_height(style::text::leading(size))
        .letter_spacing(-0.025 * size)
}

/// `text-4xl font-extrabold tracking-tight`
pub fn h1<'a>(content: impl text::IntoFragment<'a>) -> Typeset<'a> {
    heading(content, style::text::XL4, font::extrabold())
}

/// `text-3xl font-semibold tracking-tight`
pub fn h2<'a>(content: impl text::IntoFragment<'a>) -> Typeset<'a> {
    heading(content, style::text::XL3, font::semibold())
}

/// A section heading as in the documentation: an [`h2`] above a bottom
/// border (`border-b pb-2`) that fills the available width.
pub fn h2_bordered<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    Column::new()
        .push(h2(content))
        .push(
            container(iced::widget::space().height(1))
                .width(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(tokens(theme).palette.border)),
                    ..container::Style::default()
                }),
        )
        .spacing(8)
        .width(Length::Fill)
        .into()
}

/// `text-2xl font-semibold tracking-tight`
pub fn h3<'a>(content: impl text::IntoFragment<'a>) -> Typeset<'a> {
    heading(content, style::text::XL2, font::semibold())
}

/// `text-xl font-semibold tracking-tight`
pub fn h4<'a>(content: impl text::IntoFragment<'a>) -> Typeset<'a> {
    heading(content, style::text::XL, font::semibold())
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
pub fn inline_code<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(
        text(content)
            .size(style::text::SM)
            .line_height(style::text::leading(style::text::SM))
            .font(font::weight(font::mono(), iced::font::Weight::Semibold)),
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
/// Geist has no italic face: unless one is loaded, the text is slanted like
/// browsers synthesize an oblique face.
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
                typeset(content.into_fragment().into_owned())
                    .size(style::text::BASE)
                    .line_height(style::text::leading(style::text::BASE))
                    .font(font::sans())
                    .italic(true),
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

/// Balances the lines of wrapped text (`text-balance`): lays out `content`
/// in the narrowest width that does not add lines, so that its lines have
/// similar lengths.
///
/// ```no_run
/// # use shadcn_iced::typography::balance;
/// # fn view<'a>() -> iced::Element<'a, ()> {
/// balance(iced::widget::text("A heading long enough to wrap")).into()
/// # }
/// ```
pub fn balance<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Balance<'a, Message> {
    Balance {
        content: content.into(),
    }
}

/// Wrapped text with balanced lines. See [`balance`].
pub struct Balance<'a, Message> {
    content: Element<'a, Message>,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Balance<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tree = &mut tree.children[0];
        let content = self.content.as_widget_mut();
        let node = content.layout(tree, renderer, limits);
        let height = node.size().height;

        let mut fits = node.size().width.min(limits.max().width);
        let mut narrow = limits.min().width;

        if !fits.is_finite() {
            return layout::Node::with_children(node.size(), vec![node]);
        }

        // Like browsers, bisect the width while the height does not change.
        while fits - narrow > 0.5 {
            let width = (fits + narrow) / 2.0;
            let candidate = content.layout(tree, renderer, &limits.max_width(width));

            if candidate.size().height > height + 0.01 {
                narrow = width;
            } else {
                fits = width;
            }
        }

        let node = content.layout(tree, renderer, &limits.max_width(fits));

        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Balance<'a, Message>> for Element<'a, Message> {
    fn from(balance: Balance<'a, Message>) -> Self {
        Element::new(balance)
    }
}
