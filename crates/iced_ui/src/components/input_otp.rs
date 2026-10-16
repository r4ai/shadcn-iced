//! Accessible one-time password component with copy paste functionality.
//!
//! Port of <https://ui.shadcn.com/docs/components/input-otp>.
//!
//! ```no_run
//! # use iced_ui::input_otp::input_otp;
//! # #[derive(Clone)] enum Message { Code(String) }
//! # fn view(code: &str) -> iced::Element<'_, Message> {
//! input_otp(6, code, Message::Code).groups(&[3, 3]).into()
//! # }
//! ```
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::{self as advanced_text, Renderer as _};
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::time::Instant;
use iced::widget::text_input::{self, TextInput};
use iced::{
    Background, Border, Color, Event, Length, Pixels, Point, Rectangle, Size, Theme, alignment,
    border, mouse, window,
};

const SLOT: f32 = 36.0;
const SEPARATOR: f32 = 32.0;

/// A one-time password input. See the [module documentation](self).
pub struct InputOtp<'a, Message> {
    input: TextInput<'a, Message>,
    value: String,
    length: usize,
    groups: Vec<usize>,
    invalid: bool,
}

/// Creates an [`InputOtp`] with `length` slots. Extra characters are
/// dropped.
pub fn input_otp<'a, Message: Clone + 'a>(
    length: usize,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> InputOtp<'a, Message> {
    let value: String = value.chars().take(length).collect();

    InputOtp {
        input: TextInput::new("", &value)
            .on_input(move |value| on_input(value.chars().filter(|c| !c.is_whitespace()).take(length).collect()))
            .size(1)
            .padding(0)
            .style(|_, _| text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: Color::TRANSPARENT,
                placeholder: Color::TRANSPARENT,
                value: Color::TRANSPARENT,
                selection: Color::TRANSPARENT,
            }),
        value,
        length,
        groups: vec![length],
        invalid: false,
    }
}

impl<'a, Message: Clone + 'a> InputOtp<'a, Message> {
    /// Splits the slots into groups separated by a dash, e.g. `&[3, 3]`.
    pub fn groups(mut self, groups: &[usize]) -> Self {
        self.groups = groups.to_vec();
        self
    }

    /// Sets the message produced when all the slots are filled and
    /// <kbd>Enter</kbd> is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.input = self.input.on_submit(message);
        self
    }

    /// Marks the input as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the [`Id`] of the underlying text input.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.input = self.input.id(id);
        self
    }

    fn slots(&self) -> Vec<Rectangle> {
        let mut slots = Vec::with_capacity(self.length);
        let mut x = 0.0;
        let mut remaining = self.length;

        for (index, &group) in self.groups.iter().enumerate() {
            let group = group.min(remaining);

            for _ in 0..group {
                slots.push(Rectangle::new(Point::new(x, 0.0), Size::new(SLOT, SLOT)));
                x += SLOT - 1.0;
            }

            remaining -= group;
            x += 1.0;

            if index + 1 < self.groups.len() && remaining > 0 {
                x += SEPARATOR;
            }
        }

        while slots.len() < self.length {
            slots.push(Rectangle::new(Point::new(x, 0.0), Size::new(SLOT, SLOT)));
            x += SLOT - 1.0;
        }

        slots
    }

    fn width(&self) -> f32 {
        self.slots().last().map_or(0.0, |slot| slot.x + slot.width)
    }
}

#[derive(Debug)]
struct State {
    focused: bool,
    since: Instant,
}

struct FocusQuery(bool);

impl Operation for FocusQuery {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn operation::Focusable) {
        self.0 |= state.is_focused();
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, iced::Renderer> for InputOtp<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            focused: false,
            since: Instant::now(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.input as &dyn Widget<Message, Theme, iced::Renderer>)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.input as &dyn Widget<Message, Theme, iced::Renderer>]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, _limits: &layout::Limits) -> layout::Node {
        let size = Size::new(self.width(), SLOT);
        let input = Widget::<Message, Theme, iced::Renderer>::layout(
            &mut self.input,
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(size, size),
        );

        layout::Node::with_children(size, vec![input])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        Widget::<Message, Theme, iced::Renderer>::operate(
            &mut self.input,
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
        let input_layout = layout.children().next().unwrap();

        Widget::<Message, Theme, iced::Renderer>::update(
            &mut self.input,
            &mut tree.children[0],
            event,
            input_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let mut query = FocusQuery(false);
        Widget::<Message, Theme, iced::Renderer>::operate(
            &mut self.input,
            &mut tree.children[0],
            input_layout,
            renderer,
            &mut query,
        );

        let state = tree.state.downcast_mut::<State>();

        if state.focused != query.0 {
            state.focused = query.0;
            state.since = Instant::now();
            shell.request_redraw();
        }

        if state.focused
            && let Event::Window(window::Event::RedrawRequested(now)) = event
        {
            shell.request_redraw_at(*now + std::time::Duration::from_millis(500));
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let t = tokens(theme);
        let p = &t.palette;
        let origin = layout.bounds().position();
        let radius = t.radius_md();
        let slots = self.slots();
        let chars: Vec<char> = self.value.chars().collect();
        let active = state.focused.then(|| chars.len().min(self.length.saturating_sub(1)));

        let mut groups = Vec::new();
        let mut index = 0;
        for &group in &self.groups {
            groups.push((index, (index + group).min(self.length)));
            index += group;
        }

        let draw_slot = |i: usize, renderer: &mut iced::Renderer| {
            let bounds = slots[i] + (origin - Point::ORIGIN);
            let (first, last) = groups
                .iter()
                .find(|(start, end)| (*start..*end).contains(&i))
                .map(|(start, end)| (i == *start, i + 1 == *end))
                .unwrap_or((true, true));

            let mut corners = border::radius(0);
            if first {
                corners = corners.top_left(radius).bottom_left(radius);
            }
            if last {
                corners = corners.top_right(radius).bottom_right(radius);
            }

            let is_active = active == Some(i);

            if is_active {
                let ring = if self.invalid {
                    style::invalid_outset_ring(&t, radius)
                } else {
                    style::ring(&t, radius)
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: bounds.expand(ring.width),
                        border: Border {
                            color: ring.color,
                            width: ring.width,
                            radius: border::Radius {
                                top_left: if corners.top_left > 0.0 { corners.top_left + ring.width } else { 0.0 },
                                top_right: if corners.top_right > 0.0 { corners.top_right + ring.width } else { 0.0 },
                                bottom_right: if corners.bottom_right > 0.0 { corners.bottom_right + ring.width } else { 0.0 },
                                bottom_left: if corners.bottom_left > 0.0 { corners.bottom_left + ring.width } else { 0.0 },
                            },
                        },
                        ..renderer::Quad::default()
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: if self.invalid {
                            p.destructive
                        } else if is_active {
                            p.ring
                        } else {
                            p.input
                        },
                        width: 1.0,
                        radius: corners,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(if t.is_dark() { alpha(p.input, 0.3) } else { p.background }),
            );

            if let Some(c) = chars.get(i) {
                renderer.fill_text(
                    advanced_text::Text {
                        content: c.to_string(),
                        bounds: bounds.size(),
                        size: Pixels(style::text::SM),
                        line_height: advanced_text::LineHeight::Absolute(Pixels(20.0)),
                        font: font::SANS,
                        align_x: advanced_text::Alignment::Center,
                        align_y: alignment::Vertical::Center,
                        shaping: advanced_text::Shaping::Basic,
                        wrapping: advanced_text::Wrapping::None,
                    },
                    bounds.center(),
                    p.foreground,
                    bounds,
                );
            } else if is_active && (state.since.elapsed().as_millis() / 500) % 2 == 0 {
                let center = bounds.center();

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(Point::new(center.x - 0.5, center.y - 8.0), Size::new(1.0, 16.0)),
                        ..renderer::Quad::default()
                    },
                    Background::Color(p.foreground),
                );
            }
        };

        for i in 0..self.length {
            if active != Some(i) {
                draw_slot(i, renderer);
            }
        }

        if let Some(i) = active {
            draw_slot(i, renderer);
        }

        // Group separators (`MinusIcon`).
        for (_, end) in groups.iter().take(groups.len().saturating_sub(1)) {
            if *end == 0 || *end >= self.length {
                continue;
            }

            let previous = slots[end - 1] + (origin - Point::ORIGIN);
            let center = Point::new(previous.x + previous.width + SEPARATOR / 2.0, previous.center_y());

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(Point::new(center.x - 5.0, center.y - 1.0), Size::new(10.0, 2.0)),
                    border: border::rounded(1),
                    ..renderer::Quad::default()
                },
                Background::Color(p.foreground),
            );
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: Clone + 'a> From<InputOtp<'a, Message>> for Element<'a, Message> {
    fn from(otp: InputOtp<'a, Message>) -> Self {
        Element::new(otp)
    }
}
