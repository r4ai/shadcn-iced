//! A control that allows the user to toggle between checked and not checked.
//!
//! Port of <https://ui.shadcn.com/docs/components/switch>.
//!
//! ```no_run
//! # use shadcn_iced::switch::switch;
//! # #[derive(Clone)] enum Message { Airplane(bool) }
//! # fn view<'a>(on: bool) -> iced::Element<'a, Message> {
//! switch(on).label("Airplane Mode").on_toggle(Message::Airplane).into()
//! # }
//! ```
use shadcn_iced_core::animation::{self, Transition};
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::theme::{alpha, mix};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::time::Instant;
use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Color, Event, Length, Rectangle, Theme, border, mouse, window};

/// The size of a [`Switch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-[14px] w-[24px]`, with a `size-3` thumb.
    Sm,
    /// `h-[18.4px] w-[32px]`, with a `size-4` thumb.
    #[default]
    Default,
}

/// A switch. See the [module documentation](self).
pub struct Switch<'a, Message> {
    checked: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    disabled: bool,
    invalid: bool,
    size: Size,
}

/// Creates a [`Switch`].
pub fn switch<'a, Message: Clone + 'a>(checked: bool) -> Switch<'a, Message> {
    Switch {
        checked,
        label: None,
        on_toggle: None,
        disabled: false,
        invalid: false,
        size: Size::Default,
    }
}

impl<'a, Message: Clone + 'a> Switch<'a, Message> {
    /// Sets the label, displayed on the right.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the message produced when the switch is toggled.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Disables the switch.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the switch as invalid (`aria-invalid`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Switch<'a, Message>> for Element<'a, Message> {
    fn from(switch: Switch<'a, Message>) -> Self {
        let checked = switch.checked;
        let disabled = switch.disabled || switch.on_toggle.is_none();

        let (width, height, thumb) = match switch.size {
            Size::Default => (32.0, 18.4, 16.0),
            Size::Sm => (24.0, 14.0, 12.0),
        };

        let track = Track {
            checked,
            disabled,
            invalid: switch.invalid,
            size: iced::Size::new(width, height),
            thumb,
        };

        let label = switch.label.map(|label| {
            text(label)
                .size(style::text::SM)
                .font(font::medium())
                .line_height(iced::Pixels(14.0))
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(
                        tokens(theme).palette.foreground,
                        if disabled {
                            style::DISABLED_OPACITY
                        } else {
                            1.0
                        },
                    )),
                })
        });

        let on_press = switch.on_toggle.map(|f| f(!checked));

        Pressable::new(
            Row::new()
                .push(track)
                .push(label)
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .on_press_maybe(if disabled { None } else { on_press })
        .disabled(disabled)
        .width(Length::Shrink)
        .interaction(if disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Pointer
        })
        .style(|_, _| iced::widget::button::Style::default())
        .into()
    }
}

/// The style of the track of a switch.
pub fn track(theme: &Theme, checked: bool, disabled: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let background = match (checked, t.is_dark()) {
        (true, _) => p.primary,
        (false, false) => p.input,
        (false, true) => alpha(p.input, 0.8),
    };

    container::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: border::radius(f32::MAX),
        },
        ..container::Style::default()
    }
}

/// The style of the thumb of a switch.
pub fn thumb(theme: &Theme, checked: bool, disabled: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let color = match (t.is_dark(), checked) {
        (false, _) => p.background,
        (true, true) => p.primary_foreground,
        (true, false) => p.foreground,
    };

    container::Style {
        background: Some(Background::Color(shadcn_iced_core::theme::over(
            alpha(color, opacity),
            alpha(p.background, 1.0 - opacity),
        ))),
        border: border::rounded(f32::MAX),
        ..container::Style::default()
    }
}

/// The track and thumb of a switch, animating the thumb between both ends.
struct Track {
    checked: bool,
    disabled: bool,
    invalid: bool,
    size: iced::Size,
    thumb: f32,
}

#[derive(Debug)]
struct TrackState {
    position: Transition,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Track {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TrackState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TrackState {
            position: Transition::new(if self.checked { 1.0 } else { 0.0 }),
        })
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(
            Length::Fixed(self.size.width),
            Length::Fixed(self.size.height),
        )
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<TrackState>();

        state.position.go(
            if self.checked { 1.0 } else { 0.0 },
            animation::FAST,
            animation::ease_in_out,
            Instant::now(),
        );
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(self.size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && tree
                .state
                .downcast_ref::<TrackState>()
                .position
                .is_animating(*now)
        {
            shell.request_redraw();
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
        use iced::advanced::Renderer as _;

        let position = tree
            .state
            .downcast_ref::<TrackState>()
            .position
            .value(Instant::now());
        let bounds = layout.bounds();

        let off = self::track(theme, false, self.disabled);
        let on = self::track(theme, true, self.disabled);
        let color = |style: &container::Style| match style.background {
            Some(Background::Color(color)) => color,
            _ => Color::TRANSPARENT,
        };

        let t = tokens(theme);
        let opacity = if self.disabled {
            style::DISABLED_OPACITY
        } else {
            1.0
        };

        // `aria-invalid:ring-3 aria-invalid:ring-destructive/20`
        if self.invalid {
            let ring = style::invalid_outset_ring(&t, bounds.height / 2.0);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds.expand(ring.width),
                    border: iced::Border {
                        color: alpha(ring.color, opacity),
                        width: ring.width,
                        radius: border::radius(f32::MAX),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                // `border border-transparent`, or
                // `aria-invalid:border-destructive dark:aria-invalid:border-destructive/50`
                border: iced::Border {
                    color: if self.invalid {
                        alpha(
                            t.palette.destructive,
                            opacity * if t.is_dark() { 0.5 } else { 1.0 },
                        )
                    } else {
                        Color::TRANSPARENT
                    },
                    width: 1.0,
                    radius: border::radius(f32::MAX),
                },
                ..renderer::Quad::default()
            },
            Background::Color(mix(color(&off), color(&on), position)),
        );

        let travel = bounds.width - self.thumb - 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + 1.0 + travel * position,
                    y: bounds.center_y() - self.thumb / 2.0,
                    width: self.thumb,
                    height: self.thumb,
                },
                border: border::rounded(f32::MAX),
                ..renderer::Quad::default()
            },
            Background::Color(mix(
                color(&self::thumb(theme, false, self.disabled)),
                color(&self::thumb(theme, true, self.disabled)),
                position,
            )),
        );
    }
}

impl<'a, Message: 'a> From<Track> for Element<'a, Message> {
    fn from(track: Track) -> Self {
        Element::new(track)
    }
}
