//! Displays code or terminal output with a header, actions, line numbers
//! and highlighted lines.
//!
//! Port of <https://ui.shadcn.com/docs/components/code-block>. Copying is
//! up to you: answer the message of the copy button with
//! `iced::clipboard::write(code)`.
//!
//! ```no_run
//! # use shadcn_iced::code_block::code_block;
//! # use shadcn_iced_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Copy }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! code_block("pnpm add ai")
//!     .title("Terminal")
//!     .title_icon(lucide::TERMINAL)
//!     .copy_button(Message::Copy, false)
//!     .into()
//! # }
//! ```
use super::button::{self, Size as ButtonSize, Variant as ButtonVariant, icon_button};

use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Stack, container, scrollable, space, text};
use iced::{Alignment, Background, Border, Length, Padding, Pixels, Theme, border};

/// The visual style of a [`CodeBlock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// `border-input text-foreground dark:bg-input/30`
    #[default]
    Default,
    /// `border-destructive/30 text-destructive`, e.g. for errors.
    Destructive,
}

/// The size of a [`CodeBlock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `text-sm`, `px-3 py-3`
    #[default]
    Default,
    /// `text-xs`, `px-2.5 py-2.5`
    Sm,
}

impl Size {
    fn text_size(self) -> f32 {
        match self {
            Size::Default => style::text::SM,
            Size::Sm => style::text::XS,
        }
    }

    /// `leading-relaxed`
    fn line_height(self) -> f32 {
        self.text_size() * 1.625
    }

    /// The line height of the title (`text-sm`, `text-xs` when small).
    fn title_height(self) -> f32 {
        match self {
            Size::Default => 20.0,
            Size::Sm => 16.0,
        }
    }

    fn inset(self) -> f32 {
        match self {
            Size::Default => 12.0,
            Size::Sm => 10.0,
        }
    }
}

/// The style of a line (`CodeBlockLine variant`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Line {
    /// A plain line.
    #[default]
    Default,
    /// `bg-muted`
    Highlighted,
    /// `bg-muted`, marked with `+`.
    Added,
    /// `bg-destructive/10`, marked with `−`.
    Removed,
}

/// A code block. See the [module documentation](self).
pub struct CodeBlock<'a, Message> {
    lines: Vec<(String, Line)>,
    title: Option<String>,
    title_icon: Option<Icon>,
    actions: Vec<Action<'a, Message>>,
    floating: bool,
    footer: Option<Element<'a, Message>>,
    variant: Variant,
    size: Size,
    line_numbers: bool,
    start_line: usize,
    wrap: bool,
    max_height: Option<f32>,
}

/// Creates a [`CodeBlock`] showing `code` (`CodeBlockContent`).
pub fn code_block<'a, Message: Clone + 'a>(code: impl AsRef<str>) -> CodeBlock<'a, Message> {
    CodeBlock {
        lines: code
            .as_ref()
            .trim_end_matches('\n')
            .split('\n')
            .map(|line| (line.to_owned(), Line::Default))
            .collect(),
        title: None,
        title_icon: None,
        actions: Vec::new(),
        floating: false,
        footer: None,
        variant: Variant::Default,
        size: Size::Default,
        line_numbers: false,
        start_line: 1,
        wrap: false,
        max_height: None,
    }
}

impl<'a, Message: Clone + 'a> CodeBlock<'a, Message> {
    /// Sets the title shown in the header (`CodeBlockTitle`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Shows an icon before the title (`size-3.5 me-1.5`).
    pub fn title_icon(mut self, icon: Icon) -> Self {
        self.title_icon = Some(icon);
        self
    }

    /// Appends an action to the header (`CodeBlockActions`), e.g. an
    /// [`action`] button.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(Action::Element(action.into()));
        self
    }

    /// Appends a copy button (`CodeBlockCopyButton`) producing `on_press`;
    /// `copied` swaps its icon for a check.
    pub fn copy_button(mut self, on_press: Message, copied: bool) -> Self {
        self.actions.push(Action::Copy(on_press, copied));
        self
    }

    /// Floats the actions over the top right corner of the content when
    /// there is no header (`[data-slot=code-block]>&:absolute top-2
    /// end-2`).
    pub fn floating_actions(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }

    /// Sets the footer (`CodeBlockFooter`), e.g. a "Show more" button.
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Numbers the lines (`lineNumbers`).
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    /// Sets the number of the first line (`startLine`).
    pub fn start_line(mut self, line: usize) -> Self {
        self.start_line = line;
        self
    }

    /// Wraps long lines instead of scrolling horizontally (`wrap`).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Caps the height of the content, which then scrolls (`max-h-*`).
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Sets the style of the line at `index` (from 0).
    pub fn line(mut self, index: usize, line: Line) -> Self {
        if let Some((_, variant)) = self.lines.get_mut(index) {
            *variant = line;
        }
        self
    }

    /// Sets the style of every line with a function of its index and
    /// content.
    pub fn lines_with(mut self, f: impl Fn(usize, &str) -> Line) -> Self {
        for (index, (content, variant)) in self.lines.iter_mut().enumerate() {
            *variant = f(index, content);
        }
        self
    }

    /// Reads a unified diff: lines starting with `+` and `-` become
    /// [`Line::Added`] and [`Line::Removed`], and the first character of
    /// every line is dropped.
    pub fn diff(mut self) -> Self {
        for (content, variant) in &mut self.lines {
            *variant = match content.chars().next() {
                Some('+') => Line::Added,
                Some('-') => Line::Removed,
                _ => Line::Default,
            };

            let mut chars = content.chars();
            chars.next();
            *content = chars.as_str().to_owned();
        }
        self
    }
}

/// An action of the header, built once the variant is known.
enum Action<'a, Message> {
    Element(Element<'a, Message>),
    Copy(Message, bool),
}

/// A ghost `icon-xs` button taking the color of a destructive code block
/// (`text-destructive`), until hovered (`hover:text-foreground`).
fn destructive_action<'a, Message: Clone + 'a>(
    icon: Icon,
    on_press: Message,
) -> Element<'a, Message> {
    let size = ButtonSize::IconXs;

    Pressable::new(
        container(icon.sized(size.icon_size()))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(on_press)
    .width(size.height())
    .height(size.height())
    .press_offset(1.0)
    .clip_background(true)
    .style(move |theme, status| {
        let t = tokens(theme);
        let mut style = button::style(theme, status, ButtonVariant::Ghost);

        if !matches!(
            status,
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
        ) {
            style.text_color = t.palette.destructive;
        }

        style.border.radius = size.radius(&t).into();
        style
    })
    .into()
}

/// A ghost `icon-xs` button for the actions of a code block.
pub fn action<'a, Message: Clone + 'a>(icon: Icon, on_press: Message) -> Element<'a, Message> {
    icon_button(icon)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconXs)
        .on_press(on_press)
        .into()
}

/// A copy button (`CodeBlockCopyButton`); `copied` shows a check instead.
pub fn copy_button<'a, Message: Clone + 'a>(
    on_press: Message,
    copied: bool,
) -> Element<'a, Message> {
    action(if copied { lucide::CHECK } else { lucide::COPY }, on_press)
}

fn line_background(theme: &Theme, line: Line) -> Option<Background> {
    let p = tokens(theme).palette;

    match line {
        Line::Default => None,
        Line::Highlighted | Line::Added => Some(Background::Color(p.muted)),
        Line::Removed => Some(Background::Color(alpha(p.destructive, 0.1))),
    }
}

/// The `+` or `−` of a diff line (`after:start-1 after:text-muted-foreground`).
fn marker<'a, Message: 'a>(line: Line, size: Size) -> Option<Element<'a, Message>> {
    let symbol = match line {
        Line::Added => "+",
        Line::Removed => "−",
        _ => return None,
    };

    Some(
        container(
            text(symbol)
                .size(size.text_size())
                .font(font::mono())
                .line_height(Pixels(size.line_height()))
                .style(move |theme: &Theme| {
                    let p = tokens(theme).palette;
                    text::Style {
                        color: Some(if line == Line::Removed {
                            p.destructive
                        } else {
                            p.muted_foreground
                        }),
                    }
                }),
        )
        .padding(Padding::ZERO.left(4))
        .into(),
    )
}

impl<'a, Message: Clone + 'a> From<CodeBlock<'a, Message>> for Element<'a, Message> {
    fn from(block: CodeBlock<'a, Message>) -> Self {
        let size = block.size;
        let variant = block.variant;
        let inset = size.inset();
        let line_height = size.line_height();
        let has_header = block.title.is_some() || (!block.actions.is_empty() && !block.floating);
        let has_footer = block.footer.is_some();
        let numbers = block.line_numbers;
        let digits = (block.start_line + block.lines.len()).to_string().len();

        let number = move |index: usize| {
            // `before:min-w-6 before:text-end before:text-muted-foreground`
            container(
                text((block.start_line + index).to_string())
                    .size(size.text_size())
                    .font(font::mono())
                    .line_height(Pixels(line_height))
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            )
            .align_right(Length::Fixed(
                (digits as f32 * size.text_size() * 0.6).max(24.0),
            ))
        };

        let code = |content: String| {
            text(if content.is_empty() {
                " ".to_owned()
            } else {
                content
            })
            .size(size.text_size())
            .font(font::mono())
            .line_height(Pixels(line_height))
        };

        // `px-3 py-3`, `pt-2` under a header, `pb-2` above a footer
        // (`pt-1.5` / `py-2.5` when small).
        let top = match (size, has_header) {
            (Size::Default, true) => 8.0,
            (Size::Sm, true) => 6.0,
            _ => inset,
        };
        let bottom = if has_footer { 8.0 } else { inset };

        let content: Element<'a, Message> = if block.wrap {
            // Wrapped lines have varying heights: each one draws its own
            // background (`-mx-3 px-3`).
            let lines = block
                .lines
                .into_iter()
                .enumerate()
                .map(|(index, (content, line))| {
                    let row = Row::new()
                        .push(numbers.then(|| number(index)))
                        .push(code(content).width(Length::Fill))
                        .spacing(12);

                    let line_row = container(row)
                        .padding([0.0, inset])
                        .width(Length::Fill)
                        .style(move |theme| container::Style {
                            background: line_background(theme, line),
                            ..container::Style::default()
                        });

                    match marker(line, size) {
                        Some(marker) => Stack::new().push(line_row).push(marker).into(),
                        None => line_row.into(),
                    }
                });

            Column::with_children(lines)
                .width(Length::Fill)
                .padding(Padding::ZERO.top(top).bottom(bottom))
                .into()
        } else {
            // The text scrolls horizontally over full-width line
            // backgrounds (`-mx-3 px-3`).
            let backgrounds = Column::with_children(block.lines.iter().map(|(_, line)| {
                let line = *line;
                let background = container(space())
                    .width(Length::Fill)
                    .height(line_height)
                    .style(move |theme| container::Style {
                        background: line_background(theme, line),
                        ..container::Style::default()
                    });

                match marker(line, size) {
                    Some(marker) => Stack::new().push(background).push(marker).into(),
                    None => background.into(),
                }
            }))
            .padding(Padding::ZERO.top(top).bottom(bottom));

            let lines = Column::with_children(block.lines.into_iter().enumerate().map(
                |(index, (content, _))| {
                    Row::new()
                        .push(numbers.then(|| number(index)))
                        .push(code(content).wrapping(text::Wrapping::None))
                        .spacing(12)
                        .into()
                },
            ))
            .padding(Padding::new(inset).top(top).bottom(bottom));

            // `scrollbar-thin`
            let text = scrollable(lines)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new().width(6).scroller_width(6),
                ))
                .style(super::scroll_area::style)
                .width(Length::Fill);

            // The backgrounds come first: they are drawn below the text and
            // size the stack.
            Stack::new()
                .push(backgrounds)
                .push(text)
                .width(Length::Fill)
                .into()
        };

        let content: Element<'a, Message> = match block.max_height {
            Some(height) => container(
                scrollable(content)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new().width(6).scroller_width(6),
                    ))
                    .style(super::scroll_area::style)
                    .width(Length::Fill),
            )
            .max_height(height)
            .into(),
            None => content,
        };

        // `CodeBlockTitle`: `font-sans text-sm font-medium text-foreground`
        let title = block.title.map(|title| {
            Row::new()
                .push(block.title_icon.map(|icon| icon.sized(14)))
                .push(
                    text(title)
                        .size(size.text_size())
                        .line_height(Pixels(size.title_height()))
                        .font(font::medium())
                        .wrapping(text::Wrapping::None),
                )
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Fill)
        });

        // `CodeBlockActions`: `gap-1`
        let actions = (!block.actions.is_empty()).then(|| {
            Row::with_children(block.actions.into_iter().map(|action| match action {
                Action::Element(element) => element,
                Action::Copy(on_press, copied) => {
                    let icon = if copied { lucide::CHECK } else { lucide::COPY };

                    match variant {
                        Variant::Default => self::action(icon, on_press),
                        Variant::Destructive => destructive_action(icon, on_press),
                    }
                }
            }))
            .spacing(4)
            .align_y(Alignment::Center)
        });

        let (header_actions, floating_actions) = if block.floating && title.is_none() {
            (None, actions)
        } else {
            (actions, None)
        };

        // `gap-2 px-3 pt-3 text-muted-foreground`, actions pulled by
        // `-my-1 -me-1.5`.
        let header = has_header.then(|| {
            container(
                Row::new()
                    .push(title.map(|title| {
                        container(title).style(move |theme: &Theme| {
                            let p = tokens(theme).palette;
                            container::Style {
                                text_color: Some(match variant {
                                    Variant::Default => p.foreground,
                                    Variant::Destructive => p.destructive,
                                }),
                                ..container::Style::default()
                            }
                        })
                    }))
                    .push(space::horizontal())
                    .push(header_actions)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    // The actions are pulled out of the row (`-my-1`).
                    .height(size.title_height()),
            )
            .padding(Padding::new(inset).bottom(0).right(inset - 6.0))
            .style(move |theme: &Theme| {
                let p = tokens(theme).palette;
                container::Style {
                    text_color: Some(match variant {
                        Variant::Default => p.muted_foreground,
                        Variant::Destructive => p.destructive,
                    }),
                    ..container::Style::default()
                }
            })
        });

        // `gap-2 px-3 pb-3 text-muted-foreground`
        let footer = block.footer.map(|footer| {
            container(footer)
                .padding(Padding::new(inset).top(0))
                .center_x(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    text_color: Some(tokens(theme).palette.muted_foreground),
                    ..container::Style::default()
                })
        });

        let body: Element<'a, Message> = match floating_actions {
            // `absolute top-2 end-2`, buttons on `bg-background ring-1
            // ring-foreground/10`.
            Some(actions) => {
                let offset = if size == Size::Sm { 6.0 } else { 8.0 };
                Stack::new()
                    .push(content)
                    .push(
                        container(container(actions).style(|theme: &Theme| {
                            let t = tokens(theme);
                            container::Style {
                                background: Some(Background::Color(t.palette.background)),
                                border: style::ring_border(&t, t.radius_md().min(10.0)),
                                ..container::Style::default()
                            }
                        }))
                        .align_right(Length::Fill)
                        .padding(offset),
                    )
                    .into()
            }
            None => content,
        };

        container(
            Column::new()
                .push(header)
                .push(body)
                .push(footer)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        // Inside the 1px border.
        .padding(1)
        .clip(true)
        .style(move |theme: &Theme| self::style(theme, variant))
        .into()
    }
}

/// The style of a code block.
pub fn style(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let (background, text_color, border_color) = match variant {
        Variant::Default => (dark.then(|| alpha(p.input, 0.3)), p.foreground, p.input),
        Variant::Destructive => (
            None,
            p.destructive,
            alpha(p.destructive, if dark { 0.4 } else { 0.3 }),
        ),
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(text_color),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(t.radius_lg()),
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_diffs() {
        let block = code_block::<()>(" a\n-b\n+c\n").diff();
        let lines: Vec<_> = block.lines.iter().map(|(l, v)| (l.as_str(), *v)).collect();
        assert_eq!(
            lines,
            [
                ("a", Line::Default),
                ("b", Line::Removed),
                ("c", Line::Added)
            ]
        );
    }

    #[test]
    fn line_heights() {
        assert_eq!(Size::Default.line_height(), 22.75);
        assert_eq!(Size::Sm.line_height(), 19.5);
    }
}
