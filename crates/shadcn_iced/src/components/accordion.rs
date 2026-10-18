//! A vertically stacked set of interactive headings that each reveal a
//! section of content.
//!
//! Port of <https://ui.shadcn.com/docs/components/accordion>.
//!
//! By default, items open and close on their own (`multiple`). Use
//! [`Accordion::single`] to only allow one open item, controlled by your
//! application.
//!
//! ```no_run
//! # use shadcn_iced::accordion::accordion;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! accordion()
//!     .item("Is it accessible?", iced::widget::text("Yes."))
//!     .item("Is it styled?", iced::widget::text("Yes."))
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::{Disclosure, Pressable};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Color, Event, Length, Rectangle, Size, Theme, mouse};

struct Item<'a, Message> {
    title: String,
    content: Element<'a, Message>,
    open: bool,
    disabled: bool,
}

/// An accordion. See the [module documentation](self).
pub struct Accordion<'a, Message> {
    items: Vec<Item<'a, Message>>,
    single: Option<(Option<usize>, Box<dyn Fn(Option<usize>) -> Message + 'a>)>,
    bordered: bool,
    width: Length,
}

/// Creates an empty [`Accordion`].
pub fn accordion<'a, Message: Clone + 'a>() -> Accordion<'a, Message> {
    Accordion {
        items: Vec::new(),
        single: None,
        bordered: false,
        width: Length::Fill,
    }
}

impl<'a, Message: Clone + 'a> Accordion<'a, Message> {
    fn push(
        mut self,
        title: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
        open: bool,
        disabled: bool,
    ) -> Self {
        self.items.push(Item {
            title: title.into(),
            content: content.into(),
            open,
            disabled,
        });
        self
    }

    /// Appends an item.
    pub fn item(self, title: impl Into<String>, content: impl Into<Element<'a, Message>>) -> Self {
        self.push(title, content, false, false)
    }

    /// Appends an item that starts open.
    pub fn open_item(
        self,
        title: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        self.push(title, content, true, false)
    }

    /// Appends an item that cannot be toggled. Like in shadcn/ui, it looks
    /// like the others.
    pub fn disabled_item(
        self,
        title: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        self.push(title, content, false, true)
    }

    /// Only allows one item to be open at a time. `open` is the index of the
    /// open item.
    pub fn single(
        mut self,
        open: Option<usize>,
        on_change: impl Fn(Option<usize>) -> Message + 'a,
    ) -> Self {
        self.single = Some((open, Box::new(on_change)));
        self
    }

    /// Draws a border around the accordion and pads its items, like the
    /// "Borders" example (`rounded-lg border`, items `border-b px-4`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

/// `AccordionTrigger`: `rounded-lg py-2.5 text-left text-sm font-medium
/// hover:underline`, with a muted `size-4` chevron.
fn trigger<'a, Message: Clone + 'a>(
    title: &str,
    open: bool,
    disabled: bool,
) -> Element<'a, Message> {
    // `aria-disabled:opacity-50` never applies: Base UI disables the trigger
    // with the `disabled` attribute instead, so it looks enabled.
    let opacity = 1.0;

    let row = Row::new()
        .push(
            text(title.to_owned())
                .size(style::text::SM)
                .font(font::medium())
                .line_height(iced::Pixels(20.0))
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(tokens(theme).palette.foreground, opacity)),
                }),
        )
        .push(space().width(Length::Fill))
        .push(
            (if open {
                lucide::CHEVRON_UP
            } else {
                lucide::CHEVRON_DOWN
            })
            .sized(16)
            .style(move |theme: &Theme| text::Style {
                color: Some(alpha(tokens(theme).palette.muted_foreground, opacity)),
            }),
        )
        .align_y(Alignment::Start);

    // `py-2.5`
    let row = container(row).padding([10, 0]).width(Length::Fill);

    let row: Element<'a, Message> = if disabled {
        row.into()
    } else {
        Element::new(HoverUnderline {
            content: row.into(),
            underlined: std::cell::Cell::new(false),
        })
    };

    // The trigger takes the keyboard focus; the disclosure toggles on
    // Enter and Space. `rounded-lg border border-transparent`,
    // `focus-visible:border-ring focus-visible:ring-3
    // focus-visible:ring-ring/50`
    Pressable::new(row)
        .disabled(disabled)
        .focusable(true)
        .width(Length::Fill)
        .padding(1)
        .interaction(mouse::Interaction::Pointer)
        .focus_style(|theme, _status, focused| {
            let t = tokens(theme);

            iced::widget::button::Style {
                border: style::border(
                    if focused {
                        t.palette.ring
                    } else {
                        Color::TRANSPARENT
                    },
                    t.radius_lg(),
                ),
                ..iced::widget::button::Style::default()
            }
        })
        .ring(|theme, _status, focused| {
            focused.then(|| {
                let t = tokens(theme);

                style::ring(&t, t.radius_lg())
            })
        })
        .into()
}

impl<'a, Message: Clone + 'a> From<Accordion<'a, Message>> for Element<'a, Message> {
    fn from(accordion: Accordion<'a, Message>) -> Self {
        let count = accordion.items.len();
        let single = accordion.single;
        let bordered = accordion.bordered;

        let items = accordion
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                // `text-sm`, `pt-0 pb-2.5`
                let content = container(item.content)
                    .padding(iced::Padding::ZERO.bottom(10))
                    .width(Length::Fill)
                    .style(|theme: &Theme| container::Style {
                        text_color: Some(tokens(theme).palette.foreground),
                        ..container::Style::default()
                    });

                let mut disclosure = Disclosure::new(
                    trigger(&item.title, false, item.disabled),
                    trigger(&item.title, true, item.disabled),
                    content,
                )
                .default_open(item.open)
                .disabled(item.disabled);

                if let Some((open, on_change)) = &single {
                    let is_open = *open == Some(index);
                    let message = on_change(if is_open { None } else { Some(index) });

                    disclosure = disclosure.open(is_open).on_toggle(move |_| message.clone());
                }

                let item = container(disclosure).padding(if bordered { [0, 16] } else { [0, 0] });

                // `not-last:border-b`
                if index + 1 < count {
                    Column::new()
                        .push(item)
                        .push(super::separator::separator())
                        .into()
                } else {
                    Element::from(item)
                }
            });

        let column = Column::with_children(items).width(accordion.width);

        if bordered {
            // The items sit inside the border.
            container(column)
                .width(accordion.width)
                .padding(1)
                .style(|theme: &Theme| {
                    let t = tokens(theme);

                    container::Style {
                        border: style::border(t.palette.border, t.radius_lg()),
                        ..container::Style::default()
                    }
                })
                .into()
        } else {
            column.into()
        }
    }
}

/// Underlines the title of a trigger while hovered (`hover:underline`).
///
/// The content is the padded trigger row, whose first child is the title.
struct HoverUnderline<'a, Message> {
    content: Element<'a, Message>,
    /// Whether the underline was drawn, to redraw only when the cursor
    /// enters or leaves the trigger.
    underlined: std::cell::Cell<bool>,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for HoverUnderline<'_, Message> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
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
        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event
            && cursor.is_over(layout.bounds()) != self.underlined.get()
        {
            shell.request_redraw();
        }

        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        use iced::advanced::Renderer as _;

        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);

        let underlined = cursor.is_over(layout.bounds());
        self.underlined.set(underlined);

        if !underlined {
            return;
        }

        // container > row > title
        let Some(title) = layout
            .children()
            .next()
            .and_then(|row| row.children().next())
        else {
            return;
        };

        let bounds = title.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + 16.5,
                    width: bounds.width,
                    height: 1.0,
                },
                ..renderer::Quad::default()
            },
            Background::Color(tokens(theme).palette.foreground),
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
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
}
