//! Mouse and touch input, told apart.
//!
//! iced moves the cursor to a finger while it touches the screen, so
//! `cursor.is_over(bounds)` works for both. But nothing moves the cursor away
//! when the finger lifts: without care, whatever was tapped last stays
//! hovered. On the web, `:hover` styles and `pointerenter` handlers do not
//! apply to touch either (`@media (hover: hover)`), and a press on a button
//! inside of a scrolling list scrolls the list when the finger moves.
//!
//! This module keeps track of the kind of the last pointer:
//!
//! - [`observe`] records it, from the events every primitive receives;
//! - [`hover`] hides the cursor from hover styles once a finger lifted;
//! - [`Press`] follows one finger (or the left mouse button) from its press,
//!   and turns it into a tap only when it barely moved, like a `click` on the
//!   web, so that a drag scrolls instead;
//! - [`LongPress`] opens what a right click or a hover opens with a mouse.
use iced::{Event, Point, mouse, touch};
use std::cell::Cell;
use std::time::{Duration, Instant};

/// How far a finger can move before a press stops being a tap (like the
/// slop of mobile browsers).
pub const SLOP: f32 = 10.0;

/// How long a finger has to rest for a long press (the 700ms of Radix's
/// context menu feel slow on iOS and Android, which use about 500ms).
pub const LONG_PRESS: Duration = Duration::from_millis(500);

/// The extra room given around small targets, like a resize handle, for a
/// finger (half of the 44px minimum of touch targets, minus the target).
pub const TOUCH_SLOP: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Mouse,
    /// A finger touches the screen.
    Down,
    /// The last finger lifted: the cursor rests where it was.
    Up,
}

thread_local! {
    static KIND: Cell<Kind> = const { Cell::new(Kind::Mouse) };
}

/// Records the kind of pointer an event comes from. Primitives call it at
/// the beginning of their `update`, so any of them keeps it up to date.
pub fn observe(event: &Event) {
    let kind = match event {
        Event::Touch(touch::Event::FingerPressed { .. } | touch::Event::FingerMoved { .. }) => {
            Kind::Down
        }
        Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
            Kind::Up
        }
        // A mouse moves the cursor without a button; touch emulation of
        // browsers may send a move along with a touch, but before it.
        Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::WheelScrolled { .. }) => {
            Kind::Mouse
        }
        _ => return,
    };

    KIND.with(|cell| cell.set(kind));
}

/// Whether the last pointer was a finger.
pub fn is_touch() -> bool {
    KIND.with(Cell::get) != Kind::Mouse
}

/// Whether a finger touches the screen right now.
pub fn is_touching() -> bool {
    KIND.with(Cell::get) == Kind::Down
}

/// The cursor hover styles should use: a finger hovers only while it touches
/// the screen, so the cursor levitates (it is over nothing of this layer)
/// once it lifted.
pub fn hover(cursor: mouse::Cursor) -> mouse::Cursor {
    if KIND.with(Cell::get) == Kind::Up {
        cursor.levitate()
    } else {
        cursor
    }
}

/// Whether `cursor` hovers over `bounds`, see [`hover`].
pub fn is_hovered(cursor: mouse::Cursor, bounds: iced::Rectangle) -> bool {
    hover(cursor).is_over(bounds)
}

/// The pointer that started a [`Press`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    Mouse,
    Finger(touch::Finger),
}

/// What happened to a [`Press`] on an event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// The pointer went down.
    Pressed(Point),
    /// The pointer of the press moved, by this much from where it pressed.
    Moved(Point),
    /// The finger moved too far: the press is a drag (or a scroll) now.
    Cancelled,
    /// The pointer lifted without moving more than [`SLOP`]: a tap.
    Tapped(Point),
    /// The pointer lifted after it moved, or the system took the finger.
    Released,
}

/// A press of the left mouse button or of one finger, until it is released.
///
/// Other fingers are ignored, so a second finger does not retarget or end
/// the press. A finger moving more than [`SLOP`] cancels the tap: like on
/// the web, a scroll that starts on a button does not press it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Press {
    pub pointer: Pointer,
    pub origin: Point,
    /// Whether the pointer moved more than [`SLOP`].
    pub moved: bool,
}

impl Press {
    /// Starts a press, if `event` presses the left mouse button or a finger
    /// at a position `accept`ed (usually, over the bounds of the widget).
    pub fn start(
        event: &Event,
        cursor: mouse::Cursor,
        accept: impl FnOnce(Point) -> bool,
    ) -> Option<Self> {
        let (pointer, position) = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                (Pointer::Mouse, cursor.position()?)
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                (Pointer::Finger(*id), *position)
            }
            _ => return None,
        };

        accept(position).then_some(Self {
            pointer,
            origin: position,
            moved: false,
        })
    }

    /// Follows the press on `event`; `None` when the event is not about its
    /// pointer. After [`Gesture::Tapped`] or [`Gesture::Released`], the
    /// press is over and should be dropped.
    pub fn update(&mut self, event: &Event, cursor: mouse::Cursor) -> Option<Gesture> {
        match (self.pointer, event) {
            (Pointer::Mouse, Event::Mouse(mouse::Event::CursorMoved { position })) => {
                self.moved |= position.distance(self.origin) > SLOP;
                Some(Gesture::Moved(*position))
            }
            (Pointer::Mouse, Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))) => {
                // A mouse may move a little between its press and release:
                // like a click, the release counts wherever it is.
                Some(Gesture::Tapped(cursor.position().unwrap_or(self.origin)))
            }
            (Pointer::Mouse, Event::Mouse(mouse::Event::CursorLeft)) => Some(Gesture::Released),
            (Pointer::Finger(finger), Event::Touch(touch::Event::FingerMoved { id, position }))
                if *id == finger =>
            {
                if !self.moved && position.distance(self.origin) > SLOP {
                    self.moved = true;
                    return Some(Gesture::Cancelled);
                }

                Some(Gesture::Moved(*position))
            }
            (
                Pointer::Finger(finger),
                Event::Touch(touch::Event::FingerLifted { id, position }),
            ) if *id == finger => Some(if self.moved {
                Gesture::Released
            } else {
                Gesture::Tapped(*position)
            }),
            (Pointer::Finger(finger), Event::Touch(touch::Event::FingerLost { id, .. }))
                if *id == finger =>
            {
                Some(Gesture::Released)
            }
            _ => None,
        }
    }

    /// Whether the press comes from a finger.
    pub fn is_touch(&self) -> bool {
        matches!(self.pointer, Pointer::Finger(_))
    }
}

/// A finger resting on a widget, until it rests long enough.
///
/// Start it on a [`touch::Event::FingerPressed`] and ask for a redraw at its
/// [`deadline`](Self::deadline); [`update`](Self::update) reports when the
/// press became long. Like on mobile browsers, it ends when the finger moves
/// more than [`SLOP`] or lifts before.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongPress {
    pub finger: touch::Finger,
    pub origin: Point,
    pub deadline: Instant,
}

/// What happened to a [`LongPress`] on an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    /// The finger still rests.
    Waiting,
    /// The finger rested long enough.
    Long,
    /// The finger moved or lifted before: no long press.
    Ended,
}

impl LongPress {
    /// Starts a long press if `event` presses a finger at a position
    /// `accept`ed.
    pub fn start(event: &Event, accept: impl FnOnce(Point) -> bool) -> Option<Self> {
        match event {
            Event::Touch(touch::Event::FingerPressed { id, position }) if accept(*position) => {
                Some(Self {
                    finger: *id,
                    origin: *position,
                    deadline: Instant::now() + LONG_PRESS,
                })
            }
            _ => None,
        }
    }

    /// Follows the long press on `event`.
    pub fn update(&self, event: &Event) -> Hold {
        match event {
            Event::Window(iced::window::Event::RedrawRequested(now)) if *now >= self.deadline => {
                Hold::Long
            }
            Event::Touch(touch::Event::FingerMoved { id, position })
                if *id == self.finger && position.distance(self.origin) > SLOP =>
            {
                Hold::Ended
            }
            Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) if *id == self.finger => Hold::Ended,
            _ => Hold::Waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finger(event: fn(touch::Finger, Point) -> touch::Event, x: f32) -> Event {
        Event::Touch(event(touch::Finger(0), Point::new(x, 0.0)))
    }

    fn pressed(id: touch::Finger, position: Point) -> touch::Event {
        touch::Event::FingerPressed { id, position }
    }

    fn moved(id: touch::Finger, position: Point) -> touch::Event {
        touch::Event::FingerMoved { id, position }
    }

    fn lifted(id: touch::Finger, position: Point) -> touch::Event {
        touch::Event::FingerLifted { id, position }
    }

    #[test]
    fn a_finger_that_barely_moves_taps() {
        let cursor = mouse::Cursor::Unavailable;
        let mut press = Press::start(&finger(pressed, 0.0), cursor, |_| true).unwrap();

        assert_eq!(
            press.update(&finger(moved, 4.0), cursor),
            Some(Gesture::Moved(Point::new(4.0, 0.0)))
        );
        assert_eq!(
            press.update(&finger(lifted, 4.0), cursor),
            Some(Gesture::Tapped(Point::new(4.0, 0.0)))
        );
    }

    #[test]
    fn a_finger_that_moves_away_does_not_tap() {
        let cursor = mouse::Cursor::Unavailable;
        let mut press = Press::start(&finger(pressed, 0.0), cursor, |_| true).unwrap();

        assert_eq!(
            press.update(&finger(moved, 20.0), cursor),
            Some(Gesture::Cancelled)
        );
        assert_eq!(
            press.update(&finger(moved, 0.0), cursor),
            Some(Gesture::Moved(Point::ORIGIN))
        );
        assert_eq!(
            press.update(&finger(lifted, 0.0), cursor),
            Some(Gesture::Released)
        );
    }

    #[test]
    fn other_fingers_are_ignored() {
        let cursor = mouse::Cursor::Unavailable;
        let mut press = Press::start(&finger(pressed, 0.0), cursor, |_| true).unwrap();
        let other = Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::ORIGIN,
        });

        assert_eq!(press.update(&other, cursor), None);
    }

    #[test]
    fn a_lifted_finger_stops_hovering() {
        let cursor = mouse::Cursor::Available(Point::new(5.0, 5.0));
        let bounds = iced::Rectangle::new(Point::ORIGIN, iced::Size::new(10.0, 10.0));

        observe(&finger(pressed, 5.0));
        assert!(is_hovered(cursor, bounds));

        observe(&finger(lifted, 5.0));
        assert!(!is_hovered(cursor, bounds));
        assert!(is_touch());

        observe(&Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(5.0, 5.0),
        }));
        assert!(is_hovered(cursor, bounds));
        assert!(!is_touch());
    }

    #[test]
    fn a_resting_finger_presses_long() {
        let hold = LongPress::start(&finger(pressed, 0.0), |_| true).unwrap();
        let redraw = |at| Event::Window(iced::window::Event::RedrawRequested(at));

        assert_eq!(hold.update(&finger(moved, 3.0)), Hold::Waiting);
        assert_eq!(hold.update(&redraw(Instant::now())), Hold::Waiting);
        assert_eq!(hold.update(&redraw(hold.deadline)), Hold::Long);
        assert_eq!(hold.update(&finger(moved, 30.0)), Hold::Ended);
    }
}
