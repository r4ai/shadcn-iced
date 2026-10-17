//! A clickable surface whose look depends on its interaction status.
//!
//! [`Pressable`] is similar to [`iced::widget::Button`], with two differences
//! that matter for a component library:
//!
//! - It is only disabled when explicitly asked to. A pressable without an
//!   `on_press` message still reacts to hover, which is what triggers of
//!   self-managed overlays (dropdown menus, popovers, ...) need.
//! - It never draws the focus ring of the browser; components decide.
//!
//! It reuses [`button::Style`] and [`button::Status`], so any iced button
//! style function works with it.
use crate::animation::{self, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};

use iced::time::Instant;
use iced::widget::button::{Status, Style};
use iced::{
    Background, Color, Element, Event, Length, Padding, Rectangle, Size, Vector, mouse, touch,
    window,
};

/// A clickable surface.
pub struct Pressable<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<OnPress<'a, Message>>,
    disabled: bool,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    interaction: Option<mouse::Interaction>,
    press_offset: f32,
    clip_background: bool,
    style: Box<dyn Fn(&Theme, Status) -> Style + 'a>,
}

enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<Message: Clone> OnPress<'_, Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
            OnPress::Closure(f) => f(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Pressable<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates a new [`Pressable`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            content,
            on_press: None,
            disabled: false,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: Padding::ZERO,
            clip: false,
            interaction: None,
            press_offset: 0.0,
            clip_background: false,
            style: Box::new(|_, _| Style {
                background: None,
                text_color: Color::BLACK,
                ..Style::default()
            }),
        }
    }

    /// Moves the whole pressable down by `offset` while it is pressed
    /// (`active:translate-y-px`).
    pub fn press_offset(mut self, offset: f32) -> Self {
        self.press_offset = offset;
        self
    }

    /// Paints the background inside the border only (`bg-clip-padding`), so
    /// that a transparent border leaves a gap around it.
    pub fn clip_background(mut self, clip: bool) -> Self {
        self.clip_background = clip;
        self
    }

    /// Sets the message produced when the [`Pressable`] is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    /// Sets the message produced when the [`Pressable`] is pressed, if any.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    /// Sets a closure producing the message when the [`Pressable`] is
    /// pressed.
    pub fn on_press_with(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(f)));
        self
    }

    /// Disables the [`Pressable`].
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Clips the content to the bounds of the [`Pressable`].
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Overrides the mouse cursor shown on hover.
    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    /// Sets the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }
}

#[derive(Debug, Default)]
struct State {
    is_pressed: bool,
    is_hovered: bool,
    status: Option<Status>,
    previous: Option<Status>,
    transition: Transition,
}

impl<'a, Message, Theme, Renderer> Pressable<'a, Message, Theme, Renderer> {
    fn status(&self, state: &State, cursor: mouse::Cursor, bounds: Rectangle) -> Status {
        if self.disabled {
            Status::Disabled
        } else if cursor.is_over(bounds) {
            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        }
    }
}

/// Linearly interpolates two button styles, for `transition-colors`.
pub fn mix_styles(from: &Style, to: &Style, t: f32) -> Style {
    let color = |a: Color, b: Color| crate::theme::mix(a, b, t);
    let background = |background: Option<Background>| match background {
        Some(Background::Color(color)) => Some(color),
        _ => None,
    };

    let background = match (background(from.background), background(to.background)) {
        (Some(a), Some(b)) => Some(Background::Color(color(a, b))),
        (None, Some(b)) => Some(Background::Color(crate::theme::alpha(b, t))),
        (Some(a), None) => Some(Background::Color(crate::theme::alpha(a, 1.0 - t))),
        (None, None) => to.background,
    };

    Style {
        background,
        text_color: color(from.text_color, to.text_color),
        border: iced::Border {
            color: color(from.border.color, to.border.color),
            ..to.border
        },
        shadow: iced::Shadow {
            color: color(from.shadow.color, to.shadow.color),
            ..to.shadow
        },
        snap: to.snap,
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Pressable<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
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

        let bounds = layout.bounds();
        let is_hovered = cursor.is_over(bounds);

        {
            let status = self.status(tree.state.downcast_ref::<State>(), cursor, bounds);
            let state = tree.state.downcast_mut::<State>();
            let now = Instant::now();

            match state.status {
                Some(current) if current != status => {
                    state.previous = Some(current);
                    state.transition.set(0.0);
                    state
                        .transition
                        .go(1.0, animation::FAST, animation::ease_in_out, now);
                    shell.request_redraw();
                }
                None => state.transition.set(1.0),
                _ => {}
            }

            state.status = Some(status);

            if let Event::Window(window::Event::RedrawRequested(_)) = event
                && state.transition.is_animating(now)
            {
                shell.request_redraw();
            }
        }

        let state = tree.state.downcast_mut::<State>();

        if state.is_hovered != is_hovered {
            state.is_hovered = is_hovered;
            shell.request_redraw();
        }

        if shell.is_event_captured() || self.disabled {
            state.is_pressed = false;
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if is_hovered {
                    state.is_pressed = true;
                    shell.request_redraw();

                    if self.on_press.is_some() {
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if state.is_pressed {
                    state.is_pressed = false;
                    shell.request_redraw();

                    if let Some(on_press) = &self.on_press {
                        if is_hovered {
                            shell.publish(on_press.get());
                        }

                        shell.capture_event();
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. })
            | Event::Mouse(mouse::Event::CursorLeft) => {
                state.is_pressed = false;
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let status = self.status(state, cursor, bounds);
        let now = Instant::now();

        let style = match (state.previous, state.status) {
            (Some(previous), Some(current))
                if current == status && state.transition.is_animating(now) =>
            {
                mix_styles(
                    &(self.style)(theme, previous),
                    &(self.style)(theme, current),
                    state.transition.value(now),
                )
            }
            _ => (self.style)(theme, status),
        };

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        let draw = |renderer: &mut Renderer| {
            let inset = if self.clip_background {
                style.border.width
            } else {
                0.0
            };

            if inset > 0.0 {
                // The border and shadow, then the background inside the border.
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style.border,
                        shadow: style.shadow,
                        snap: style.snap,
                    },
                    Background::Color(Color::TRANSPARENT),
                );

                if let Some(background) = style.background {
                    let radius = style.border.radius;
                    let inner = |r: f32| (r - inset).max(0.0);

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: bounds.shrink(inset),
                            border: iced::Border {
                                radius: iced::border::Radius {
                                    top_left: inner(radius.top_left),
                                    top_right: inner(radius.top_right),
                                    bottom_right: inner(radius.bottom_right),
                                    bottom_left: inner(radius.bottom_left),
                                },
                                ..iced::Border::default()
                            },
                            snap: style.snap,
                            ..renderer::Quad::default()
                        },
                        background,
                    );
                }
            } else if style.background.is_some()
                || style.border.width > 0.0
                || style.shadow.color.a > 0.0
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style.border,
                        shadow: style.shadow,
                        snap: style.snap,
                    },
                    style
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
            }

            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color,
                },
                layout.children().next().unwrap(),
                cursor,
                &viewport,
            );
        };

        if status == Status::Pressed && self.press_offset != 0.0 {
            renderer.with_translation(Vector::new(0.0, self.press_offset), draw);
        } else {
            draw(renderer);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        );

        if content != mouse::Interaction::None {
            return content;
        }

        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        if self.disabled {
            mouse::Interaction::NotAllowed
        } else {
            self.interaction.unwrap_or(mouse::Interaction::Pointer)
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Pressable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(pressable: Pressable<'a, Message, Theme, Renderer>) -> Self {
        Element::new(pressable)
    }
}

/// Creates a new [`Pressable`].
pub fn pressable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Pressable<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Pressable::new(content)
}
