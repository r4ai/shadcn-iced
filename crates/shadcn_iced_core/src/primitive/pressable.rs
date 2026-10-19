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
//! It can also take the keyboard focus ([`Pressable::focusable`]), like a
//! `<button>`: <kbd>Tab</kbd> (iced's focus operations) focuses it,
//! <kbd>Space</kbd> and <kbd>Enter</kbd> press it, and its style knows
//! whether the focus is visible (`:focus-visible`): focus gained with the
//! keyboard is, focus gained with the pointer is not until a key is pressed.
//!
//! It reuses [`button::Style`] and [`button::Status`], so any iced button
//! style function works with it.
use super::focus::{Focus, Ring};
use crate::animation::{self, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};

use iced::keyboard::{self, key::Named};
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
    style: Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>,
    ring: Option<Box<dyn Fn(&Theme, Status, bool) -> Option<Ring> + 'a>>,
    outline: Option<Box<dyn Fn(&Theme, Status, bool) -> Option<Ring> + 'a>>,
    focusable: bool,
    tab_stop: bool,
    enter: bool,
    id: Option<Id>,
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
            style: Box::new(|_, _, _| Style {
                background: None,
                text_color: Color::BLACK,
                ..Style::default()
            }),
            ring: None,
            outline: None,
            focusable: false,
            tab_stop: true,
            enter: true,
            id: None,
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
        self.style = Box::new(move |theme, status, _| style(theme, status));
        self
    }

    /// Sets the style function, which also receives whether the
    /// [`Pressable`] shows its keyboard focus (`focus-visible:`).
    pub fn focus_style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }

    /// Sets a ring drawn *outside* of the bounds (`ring-3`), given the
    /// status and whether the keyboard focus is visible.
    pub fn ring(mut self, ring: impl Fn(&Theme, Status, bool) -> Option<Ring> + 'a) -> Self {
        self.ring = Some(Box::new(ring));
        self
    }

    /// Sets an outline drawn *outside* of the bounds, over the ring
    /// (`outline-1`), given the status and whether the keyboard focus is
    /// visible.
    pub fn outline(mut self, outline: impl Fn(&Theme, Status, bool) -> Option<Ring> + 'a) -> Self {
        self.outline = Some(Box::new(outline));
        self
    }

    /// Makes the [`Pressable`] take the keyboard focus, like a `<button>`.
    ///
    /// It is then focused by iced's focus operations (<kbd>Tab</kbd>) and by
    /// a press, loses the focus when the pointer presses elsewhere, and is
    /// pressed by <kbd>Space</kbd> or <kbd>Enter</kbd> while focused. A
    /// disabled [`Pressable`] is never focused.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Sets whether a focusable [`Pressable`] is reached with
    /// <kbd>Tab</kbd> (`tabindex="0"`, the default), or only by the arrow
    /// keys of its group and by its label (`tabindex="-1"`), for roving
    /// focus (see [`Roving`](super::roving::Roving)).
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Sets whether <kbd>Enter</kbd> presses a focused [`Pressable`], in
    /// addition to <kbd>Space</kbd>. Defaults to `true`; checkboxes and
    /// radios only react to <kbd>Space</kbd>.
    pub fn activate_on_enter(mut self, enter: bool) -> Self {
        self.enter = enter;
        self
    }

    /// Sets the [`Id`] reported to focus operations.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Debug, Default)]
struct State {
    is_pressed: bool,
    is_hovered: bool,
    status: Option<Status>,
    /// The style the transition to `status` starts from.
    origin: Option<Origin>,
    transition: Transition,
    focus: Focus,
}

/// A style on screen: the style of `from` mixed into the style of `to` by
/// `t`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Origin {
    from: Status,
    to: Status,
    t: f32,
}

impl Origin {
    /// The style of `status` alone.
    fn settled(status: Status) -> Self {
        Self {
            from: status,
            to: status,
            t: 0.0,
        }
    }

    /// The style on screen when a transition from `self` to the style of
    /// `target` is at `t`, so that a new transition starts from it, like CSS
    /// transitions start from the current value.
    ///
    /// It is exact when `target` is one of `from` and `to`, which covers
    /// going back and forth between two statuses; otherwise it is the
    /// nearest of them.
    fn at(self, target: Status, t: f32) -> Self {
        if self.to == target {
            Self {
                t: self.t + (1.0 - self.t) * t,
                ..self
            }
        } else if self.from == target {
            Self {
                t: self.t * (1.0 - t),
                ..self
            }
        } else {
            Self {
                from: if self.t < 0.5 { self.from } else { self.to },
                to: target,
                t,
            }
        }
    }
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
    // Mixed with premultiplied alpha, like CSS transitions: a translucent
    // white turning into a translucent gray must not go through a lighter
    // gray on the way.
    let color = |a: Color, b: Color| {
        let alpha = a.a + (b.a - a.a) * t;

        if alpha <= f32::EPSILON {
            return Color { a: 0.0, ..b };
        }

        let channel = |x: f32, y: f32| (x * a.a + (y * b.a - x * a.a) * t) / alpha;

        Color {
            r: channel(a.r, b.r),
            g: channel(a.g, b.g),
            b: channel(a.b, b.b),
            a: alpha,
        }
    };
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
        if self.focusable && !self.disabled {
            let state = tree.state.downcast_mut::<State>();
            let bounds = layout.bounds();

            operation.custom(self.id.as_ref(), bounds, &mut state.focus);

            // Out of the tab order (`tabindex="-1"`) unless focused, so that
            // focus operations can still move the focus away.
            if self.tab_stop || state.focus.is_focused() {
                operation.focusable(self.id.as_ref(), bounds, &mut state.focus);
            }
        }

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
        // An activation requested by a label or a roving group.
        if tree.state.downcast_mut::<State>().focus.take_activation()
            && !self.disabled
            && let Some(on_press) = &self.on_press
        {
            shell.publish(on_press.get());
            shell.request_redraw();
        }

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
                    // A status changed mid-transition starts from the
                    // style on screen, not from the previous one.
                    state.origin = Some(match state.origin {
                        Some(origin) if state.transition.is_animating(now) => {
                            origin.at(current, state.transition.value(now))
                        }
                        _ => Origin::settled(current),
                    });
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

        if self.focusable {
            if self.disabled {
                state.focus.blur();
            } else {
                match event {
                    // Like text inputs, a press focuses the pressable or
                    // takes the focus away from it.
                    Event::Mouse(mouse::Event::ButtonPressed(_))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        if is_hovered {
                            state.focus.focus_with_pointer();
                            shell.request_redraw();
                        } else if state.focus.is_focused() {
                            state.focus.blur();
                            shell.request_redraw();
                        }
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                        if state.focus.is_focused() =>
                    {
                        if !state.focus.is_visible() {
                            state.focus.key_pressed();
                            shell.request_redraw();
                        }

                        let activates = match key {
                            keyboard::Key::Named(Named::Space) => true,
                            keyboard::Key::Named(Named::Enter) => self.enter,
                            _ => false,
                        };

                        if activates
                            && !shell.is_event_captured()
                            && let Some(on_press) = &self.on_press
                        {
                            shell.publish(on_press.get());
                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
            }
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

        let focused = state.focus.is_visible();

        let style = match (state.origin, state.status) {
            (Some(origin), Some(current))
                if current == status && state.transition.is_animating(now) =>
            {
                let from = (self.style)(theme, origin.from, focused);
                let from = if origin.t > 0.0 {
                    mix_styles(&from, &(self.style)(theme, origin.to, focused), origin.t)
                } else {
                    from
                };

                mix_styles(
                    &from,
                    &(self.style)(theme, current, focused),
                    state.transition.value(now),
                )
            }
            _ => (self.style)(theme, status, focused),
        };

        let rings = [&self.ring, &self.outline]
            .into_iter()
            .flatten()
            .filter_map(|ring| ring(theme, status, focused))
            .collect::<Vec<_>>();

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        let draw = |renderer: &mut Renderer| {
            for ring in &rings {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: bounds.expand(ring.width),
                        border: ring.border(),
                        snap: style.snap,
                        ..renderer::Quad::default()
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }

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

        let draw = |renderer: &mut Renderer| {
            if status == Status::Pressed && self.press_offset != 0.0 {
                renderer.with_translation(Vector::new(0.0, self.press_offset), draw);
            } else {
                draw(renderer);
            }
        };

        // With a ring, the widget is drawn in a layer of its own, above the
        // siblings drawn later, like the `focus-visible:z-10` of grouped
        // items (button groups, toggle groups), which overlap their borders.
        let spread = rings.iter().map(|ring| ring.width).fold(0.0, f32::max);

        match bounds
            .expand(spread + self.press_offset.abs())
            .intersection(&viewport)
        {
            Some(clip) if !rings.is_empty() => renderer.with_layer(clip, draw),
            _ => draw(renderer),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_mid_hover_starts_from_the_style_on_screen() {
        // Hovered 40% of the way, then left.
        let origin = Origin::settled(Status::Active).at(Status::Hovered, 0.4);

        assert_eq!(
            origin,
            Origin {
                from: Status::Active,
                to: Status::Hovered,
                t: 0.4,
            }
        );

        // Hovered again halfway back to `Active`.
        let origin = origin.at(Status::Active, 0.5);

        assert_eq!(origin.from, Status::Active);
        assert_eq!(origin.to, Status::Hovered);
        assert!((origin.t - 0.2).abs() < 1e-6);
    }

    #[test]
    fn going_on_towards_the_same_status_keeps_the_progress() {
        let origin = Origin {
            from: Status::Active,
            to: Status::Hovered,
            t: 0.4,
        }
        .at(Status::Hovered, 0.5);

        assert!((origin.t - 0.7).abs() < 1e-6);
    }
}
