//! Accessible one-time password component with copy paste functionality.
//!
//! Port of <https://ui.shadcn.com/docs/components/input-otp>.
//!
//! ```no_run
//! # use shadcn_iced::input_otp::input_otp;
//! # #[derive(Clone)] enum Message { Code(String) }
//! # fn view(code: &str) -> iced::Element<'_, Message> {
//! input_otp(6, code, Message::Code).groups(&[3, 3]).into()
//! # }
//! ```
use shadcn_iced_core::primitive::Otp;
use shadcn_iced_core::primitive::otp::Status;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::advanced::Renderer as _;
use iced::advanced::renderer;
use iced::advanced::text::{self as advanced_text, Renderer as _};
use iced::{Background, Border, Color, Pixels, Point, Rectangle, Size, Theme, alignment, border};

/// `size-8`
const SLOT: f32 = 32.0;
/// A `size-4` minus icon, without any gap (`flex items-center`).
const SEPARATOR: f32 = 16.0;

/// A one-time password input. See the [module documentation](self).
pub struct InputOtp<'a, Message> {
    otp: Otp<'a, Message>,
    length: usize,
    groups: Vec<usize>,
    invalid: bool,
    disabled: bool,
}

/// Creates an [`InputOtp`] with `length` slots. Extra characters are
/// dropped.
pub fn input_otp<'a, Message: Clone + 'a>(
    length: usize,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> InputOtp<'a, Message> {
    InputOtp {
        otp: Otp::new(length, value, on_input),
        length,
        groups: vec![length],
        invalid: false,
        disabled: false,
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
        self.otp = self.otp.on_submit(message);
        self
    }

    /// Marks the input as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables the input (`has-disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.otp = self.otp.disabled(disabled);
        self.disabled = disabled;
        self
    }

    /// Sets the [`Id`](iced::widget::Id) of the underlying text input.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.otp = self.otp.id(id);
        self
    }
}

/// Where the slots are: `size-8` each, in groups separated by a
/// [`SEPARATOR`].
fn slots(length: usize, groups: &[usize]) -> Vec<Rectangle> {
    let mut slots = Vec::with_capacity(length);
    let mut x = 0.0;
    let mut remaining = length;

    for (index, &group) in groups.iter().enumerate() {
        let group = group.min(remaining);

        for _ in 0..group {
            slots.push(Rectangle::new(Point::new(x, 0.0), Size::new(SLOT, SLOT)));
            x += SLOT;
        }

        remaining -= group;

        if index + 1 < groups.len() && remaining > 0 {
            x += SEPARATOR;
        }
    }

    while slots.len() < length {
        slots.push(Rectangle::new(Point::new(x, 0.0), Size::new(SLOT, SLOT)));
        x += SLOT;
    }

    slots
}

impl<'a, Message: Clone + 'a> From<InputOtp<'a, Message>> for Element<'a, Message> {
    fn from(input: InputOtp<'a, Message>) -> Self {
        let InputOtp {
            otp,
            length,
            groups: groups_sizes,
            invalid,
            disabled,
        } = input;

        let slots = slots(length, &groups_sizes);
        let width = slots.last().map_or(0.0, |slot| slot.x + slot.width);

        otp.slots(slots.clone(), Size::new(width, SLOT))
            .draw(move |renderer, theme, bounds, status: Status<'_>| {
                draw(
                    renderer,
                    theme,
                    bounds,
                    &status,
                    &slots,
                    length,
                    &groups_sizes,
                    invalid,
                    disabled,
                );
            })
            .into()
    }
}

/// Draws the slots, their characters, the caret and the separators.
#[allow(clippy::too_many_arguments)]
fn draw(
    renderer: &mut iced::Renderer,
    theme: &Theme,
    bounds: Rectangle,
    status: &Status<'_>,
    slots: &[Rectangle],
    length: usize,
    groups_sizes: &[usize],
    invalid: bool,
    disabled: bool,
) {
    let faded = disabled
        .then(|| shadcn_iced_core::theme::faded(theme, style::DISABLED_OPACITY))
        .flatten();
    let theme = faded.as_ref().unwrap_or(theme);
    let t = tokens(theme);
    let p = &t.palette;
    let origin = bounds.position();
    let radius = t.radius_lg();
    let chars = status.value;
    let active = status.active;

    let mut groups = Vec::new();
    let mut index = 0;
    for &group in groups_sizes {
        groups.push((index, (index + group).min(length)));
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
            let ring = if invalid {
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
                            top_left: if corners.top_left > 0.0 {
                                corners.top_left + ring.width
                            } else {
                                0.0
                            },
                            top_right: if corners.top_right > 0.0 {
                                corners.top_right + ring.width
                            } else {
                                0.0
                            },
                            bottom_right: if corners.bottom_right > 0.0 {
                                corners.bottom_right + ring.width
                            } else {
                                0.0
                            },
                            bottom_left: if corners.bottom_left > 0.0 {
                                corners.bottom_left + ring.width
                            } else {
                                0.0
                            },
                        },
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        // `dark:bg-input/30`
        if t.is_dark() {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: border::rounded(corners),
                    ..renderer::Quad::default()
                },
                Background::Color(alpha(p.input, 0.3)),
            );
        }

        // `border-y border-r first:border-l`: the left border of the
        // other slots is the right border of the previous one.
        let frame = if first {
            bounds
        } else {
            Rectangle {
                x: bounds.x - 1.0,
                width: bounds.width + 1.0,
                ..bounds
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: frame,
                border: Border {
                    color: if invalid {
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
            Background::Color(Color::TRANSPARENT),
        );

        // The content box, inside the borders.
        let content = Rectangle {
            x: bounds.x + if first { 1.0 } else { 0.0 },
            width: bounds.width - if first { 2.0 } else { 1.0 },
            ..bounds
        };

        if let Some(c) = chars.get(i) {
            renderer.fill_text(
                advanced_text::Text {
                    content: c.to_string(),
                    bounds: bounds.size(),
                    size: Pixels(style::text::SM),
                    line_height: advanced_text::LineHeight::Absolute(Pixels(20.0)),
                    font: font::sans(),
                    align_x: advanced_text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: advanced_text::Shaping::Basic,
                    wrapping: advanced_text::Wrapping::None,
                },
                content.center(),
                p.foreground,
                bounds,
            );
        } else if is_active && status.caret {
            let center = content.center();

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(center.x - 0.5, center.y - 8.0),
                        Size::new(1.0, 16.0),
                    ),
                    ..renderer::Quad::default()
                },
                Background::Color(p.foreground),
            );
        }
    };

    // `has-aria-invalid:ring-3 has-aria-invalid:ring-destructive/20` on
    // each group.
    if invalid {
        let ring = style::invalid_outset_ring(&t, radius);

        for &(start, end) in &groups {
            if start >= end {
                continue;
            }

            let first = slots[start] + (origin - Point::ORIGIN);
            let last = slots[end - 1] + (origin - Point::ORIGIN);
            let bounds = Rectangle::new(
                first.position(),
                Size::new(last.x + last.width - first.x, first.height),
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds.expand(ring.width),
                    border: Border {
                        color: ring.color,
                        width: ring.width,
                        radius: border::radius(radius + ring.width),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
    }

    for i in 0..length {
        if active != Some(i) {
            draw_slot(i, renderer);
        }
    }

    if let Some(i) = active {
        draw_slot(i, renderer);
    }

    // Group separators (`MinusIcon`).
    for (_, end) in groups.iter().take(groups.len().saturating_sub(1)) {
        if *end == 0 || *end >= length {
            continue;
        }

        let previous = slots[end - 1] + (origin - Point::ORIGIN);
        let center = Point::new(
            previous.x + previous.width + SEPARATOR / 2.0,
            previous.center_y(),
        );

        renderer.fill_quad(
            renderer::Quad {
                // Lucide's `minus` at 16px.
                bounds: Rectangle::new(
                    Point::new(center.x - 14.0 / 3.0 - 0.67, center.y - 0.67),
                    Size::new(28.0 / 3.0 + 1.33, 1.33),
                ),
                border: border::rounded(0.67),
                ..renderer::Quad::default()
            },
            Background::Color(p.foreground),
        );
    }
}
