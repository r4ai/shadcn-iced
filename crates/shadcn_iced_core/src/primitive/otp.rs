//! The input behind one-time password slots, like
//! [input-otp](https://input-otp.rodz.dev).
//!
//! A one-time password is typed into a single invisible text input, which
//! keeps the editing, the selection and pasting of a text input, while the
//! slots show its characters. [`Otp`] is that input: it keeps the value to
//! its length without whitespace, tracks the focus, tells which slot is
//! active (the one the next character goes to) and blinks the caret. The
//! slots are drawn by you, from a [`Status`].
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::otp::{Otp, Status};
//! # use iced::{Point, Rectangle, Size};
//! # #[derive(Clone)] enum Message { Code(String) }
//! # fn view(code: &str) -> iced::Element<'_, Message> {
//! let slots: Vec<Rectangle> = (0..6)
//!     .map(|i| Rectangle::new(Point::new(i as f32 * 32.0, 0.0), Size::new(32.0, 32.0)))
//!     .collect();
//!
//! Otp::new(6, code, Message::Code)
//!     .slots(slots, Size::new(192.0, 32.0))
//!     .draw(|renderer, theme, bounds, status: Status| {
//!         // Draw the slots in `bounds`, the characters of `status.value`,
//!         // a ring around `status.active` and the caret if `status.caret`.
//!     })
//!     .into()
//! # }
//! ```
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell};
use iced::time::{Duration, Instant};
use iced::widget::text_input::{self, TextInput};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme,
    mouse, window,
};

/// How long the caret stays visible, then hidden.
const BLINK: Duration = Duration::from_millis(500);

/// What the slots of an [`Otp`] show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status<'a> {
    /// The characters of the value, one per slot.
    pub value: &'a [char],
    /// Whether the input is focused.
    pub focused: bool,
    /// The slot the next character goes to (the last one when they are all
    /// filled), while focused.
    pub active: Option<usize>,
    /// Whether the caret is visible in the active slot, if it is empty.
    pub caret: bool,
}

/// The input of one-time password slots. See the [module
/// documentation](self).
pub struct Otp<'a, Message> {
    input: TextInput<'a, Message>,
    value: Vec<char>,
    length: usize,
    slots: Vec<Rectangle>,
    size: Size,
    #[allow(clippy::type_complexity)]
    draw: Option<Box<dyn Fn(&mut Renderer, &Theme, Rectangle, Status<'_>) + 'a>>,
}

impl<'a, Message: Clone + 'a> Otp<'a, Message> {
    /// Creates an input of `length` characters showing `value`. The
    /// whitespace is removed from what is typed or pasted, and the extra
    /// characters are dropped.
    pub fn new(length: usize, value: &str, on_input: impl Fn(String) -> Message + 'a) -> Self {
        let value: String = value.chars().take(length).collect();

        Self {
            input: TextInput::new("", &value)
                .on_input(move |value| {
                    on_input(
                        value
                            .chars()
                            .filter(|c| !c.is_whitespace())
                            .take(length)
                            .collect(),
                    )
                })
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
            value: value.chars().collect(),
            length,
            slots: Vec::new(),
            size: Size::ZERO,
            draw: None,
        }
    }

    /// Sets where the slots are, relative to the input, and the size of the
    /// input. The characters are reported as texts of the slots to
    /// operations.
    pub fn slots(mut self, slots: Vec<Rectangle>, size: Size) -> Self {
        self.slots = slots;
        self.size = size;
        self
    }

    /// Draws the slots in the bounds of the input.
    pub fn draw(
        mut self,
        draw: impl Fn(&mut Renderer, &Theme, Rectangle, Status<'_>) + 'a,
    ) -> Self {
        self.draw = Some(Box::new(draw));
        self
    }

    /// Sets the message produced when <kbd>Enter</kbd> is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.input = self.input.on_submit(message);
        self
    }

    /// Disables the input: it cannot be typed into anymore.
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.input = self.input.on_input_maybe(None::<fn(String) -> Message>);
        }

        self
    }

    /// Sets the [`Id`] of the text input.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.input = self.input.id(id);
        self
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

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.0 |= state.is_focused();
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Otp<'a, Message> {
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
        vec![Tree::new(
            &self.input as &dyn Widget<Message, Theme, Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.input as &dyn Widget<Message, Theme, Renderer>]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let input = Widget::<Message, Theme, Renderer>::layout(
            &mut self.input,
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(self.size, self.size),
        );

        layout::Node::with_children(self.size, vec![input])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        Widget::<Message, Theme, Renderer>::operate(
            &mut self.input,
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );

        // The characters of the slots are texts of their own.
        let offset = layout.bounds().position() - Point::ORIGIN;
        let mut buffer = [0; 4];

        for (slot, c) in self.slots.iter().zip(&self.value) {
            operation.text(None, *slot + offset, c.encode_utf8(&mut buffer));
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let input_layout = layout.children().next().unwrap();

        Widget::<Message, Theme, Renderer>::update(
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
        Widget::<Message, Theme, Renderer>::operate(
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
            shell.request_redraw_at(*now + BLINK);
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let Some(draw) = &self.draw else {
            return;
        };

        let state = tree.state.downcast_ref::<State>();
        let active = state
            .focused
            .then(|| self.value.len().min(self.length.saturating_sub(1)));
        let caret = active.is_some()
            && (state.since.elapsed().as_millis() / BLINK.as_millis()).is_multiple_of(2);

        draw(
            renderer,
            theme,
            layout.bounds(),
            Status {
                value: &self.value,
                focused: state.focused,
                active,
                caret,
            },
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: Clone + 'a> From<Otp<'a, Message>> for Element<'a, Message> {
    fn from(otp: Otp<'a, Message>) -> Self {
        Element::new(otp)
    }
}
