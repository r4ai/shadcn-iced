//! Momentum of touch scrolling (`shadcn_iced::primitive::wheel`).
//!
//! iced's scrollable follows a finger by itself; the wheel adds the fling
//! after the finger lifts. The velocity of the finger is measured in real
//! time (events carry no time), so the moves are spaced with short sleeps,
//! and the frames of the fling are simulated with redraw instants.
use iced::time::{Duration, Instant};
use iced::widget::{Column, scrollable, text};
use iced::{Element, Event, Point, mouse, touch, window};
use iced_test::simulator::Simulator;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// The animation switch is global.
static ANIMATIONS: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
enum Message {
    Scrolled(f32),
}

fn view(live: Arc<AtomicU32>) -> Element<'static, Message> {
    let items = (0..100).map(|i| text(format!("Item {i}")).into());

    shadcn_iced::primitive::wheel(
        scrollable(Column::with_children(items))
            .width(300)
            .height(200)
            .on_scroll(move |viewport| {
                let offset = viewport.absolute_offset().y;
                live.store(offset.to_bits(), Ordering::Relaxed);
                Message::Scrolled(offset)
            }),
    )
    .into()
}

struct Touch {
    ui: Simulator<'static, Message>,
    frames: u64,
    start: Instant,
    /// The offset of the scrollable, as reported last.
    live: Arc<AtomicU32>,
}

const FINGER: touch::Finger = touch::Finger(0);

impl Touch {
    fn new() -> Self {
        let live = Arc::new(AtomicU32::new(0));

        Self {
            ui: Simulator::with_size(
                iced::Settings::default(),
                (400.0, 300.0),
                view(live.clone()),
            ),
            live,
            frames: 0,
            start: Instant::now(),
        }
    }

    fn touch(&mut self, event: impl FnOnce(Point) -> touch::Event, at: (f32, f32)) {
        self.ui.point_at(at);
        let _ = self
            .ui
            .simulate([Event::Touch(event(Point::new(at.0, at.1)))]);
    }

    fn press(&mut self, y: f32) {
        self.touch(
            |position| touch::Event::FingerPressed {
                id: FINGER,
                position,
            },
            (150.0, y),
        );
    }

    fn move_to(&mut self, y: f32) {
        self.touch(
            |position| touch::Event::FingerMoved {
                id: FINGER,
                position,
            },
            (150.0, y),
        );
    }

    fn lift(&mut self, y: f32) {
        self.touch(
            |position| touch::Event::FingerLifted {
                id: FINGER,
                position,
            },
            (150.0, y),
        );
    }

    /// Drags the finger up from 190 by `step` every `every` ms, `moves`
    /// times, and lifts it.
    fn drag(&mut self, step: f32, moves: u32, every: u64) -> f32 {
        let mut y = 190.0;

        self.press(y);

        for _ in 0..moves {
            std::thread::sleep(std::time::Duration::from_millis(every));
            y -= step;
            self.move_to(y);
        }

        self.lift(y);
        y
    }

    /// Lets `count` frames pass (16ms apart).
    fn frames(&mut self, count: u64) {
        let (start, from) = (self.start, self.frames);

        self.frames += count;

        // The first frame is after the lift, whatever the real time.
        let _ = self.ui.simulate((from + 1..=self.frames).map(|frame| {
            Event::Window(window::Event::RedrawRequested(
                start + Duration::from_millis(16 * frame),
            ))
        }));
    }

    /// The offset of the scrollable now.
    fn offset(&self) -> f32 {
        f32::from_bits(self.live.load(Ordering::Relaxed))
    }

    /// The offsets reported so far.
    fn offsets(self) -> Vec<f32> {
        self.ui
            .into_messages()
            .map(|Message::Scrolled(y)| y)
            .collect()
    }
}

/// A flick of 20px every 10ms (2px/ms), well over 0.3px/ms.
fn flick(touch: &mut Touch) -> f32 {
    touch.drag(20.0, 4, 10)
}

#[test]
fn a_fast_flick_keeps_scrolling_after_the_lift() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    // What the finger alone scrolled.
    let mut still = Touch::new();
    let _ = flick(&mut still);
    let dragged = still.offset();

    let mut flung = Touch::new();
    let _ = flick(&mut flung);
    flung.start = Instant::now();
    flung.frames(120);
    let offset = flung.offset();

    assert!(dragged >= 79.0, "the drag did not scroll: {dragged}");
    assert!(
        offset > dragged + 200.0,
        "no momentum: {dragged} -> {offset}"
    );
}

#[test]
fn the_fling_slows_down_and_stops() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    let mut touch = Touch::new();
    let _ = flick(&mut touch);
    touch.start = Instant::now();
    touch.frames(310);

    let offsets = touch.offsets();
    let steps: Vec<f32> = offsets.windows(2).map(|w| w[1] - w[0]).collect();

    assert!(steps.iter().all(|step| *step >= 0.0), "{offsets:?}");

    // The steps of the fling shrink.
    let largest = steps.iter().copied().fold(0.0, f32::max);
    assert!(
        steps.last().unwrap() * 4.0 < largest,
        "not slowing down: {steps:?}"
    );

    // And it stopped on its own long before the last frame (a frame that
    // does not scroll reports nothing).
    assert!(offsets.len() < 200, "never stopped: {}", offsets.len());
}

#[test]
fn a_slow_drag_does_not_fling() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    // 1px every 20ms: 0.05px/ms.
    let mut touch = Touch::new();
    let _ = touch.drag(1.0, 8, 20);
    touch.start = Instant::now();
    let mut still = Touch::new();
    let _ = still.drag(1.0, 8, 20);
    let dragged = still.offset();

    touch.frames(60);

    let offset = touch.offset();
    assert!(dragged >= 7.0, "the drag did not scroll: {dragged}");
    assert!((offset - dragged).abs() < 1.0, "{dragged} -> {offset}");
}

#[test]
fn a_finger_that_rested_does_not_fling() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    let mut touch = Touch::new();
    let y = {
        let mut y = 190.0;
        touch.press(y);
        for _ in 0..3 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            y -= 20.0;
            touch.move_to(y);
        }
        y
    };

    // It rests for longer than the time the velocity is measured over.
    std::thread::sleep(std::time::Duration::from_millis(200));
    touch.lift(y);
    touch.start = Instant::now();
    touch.frames(60);

    assert!((touch.offset() - 60.0).abs() < 1.0);
}

#[test]
fn a_press_stops_the_fling() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    let mut touch = Touch::new();
    let _ = flick(&mut touch);
    touch.start = Instant::now();
    touch.frames(5);

    let flinging = touch.offset();
    touch.frames(5);
    assert!(touch.offset() > flinging + 10.0, "not flinging");

    // The finger lands again: no more scrolling.
    touch.press(100.0);
    let landed = touch.offset();
    touch.frames(60);

    assert!(
        (touch.offset() - landed).abs() < 1.0,
        "kept scrolling: {landed} -> {}",
        touch.offset()
    );
}

#[test]
fn a_click_or_a_wheel_stops_the_fling() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    let stops = |event: Event| {
        let mut touch = Touch::new();
        let _ = flick(&mut touch);
        touch.start = Instant::now();
        touch.frames(5);
        touch.ui.point_at((150.0, 100.0));

        let _ = touch.ui.simulate([event]);
        let landed = touch.offset();
        touch.frames(60);

        (touch.offset() - landed).abs() < 1.0
    };

    assert!(stops(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))));

    // A wheel that scrolls nothing (0px) still takes over.
    assert!(stops(Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 },
    })));
}

#[test]
fn a_lost_finger_does_not_fling() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(true);

    let mut touch = Touch::new();
    let mut y = 190.0;
    touch.press(y);
    for _ in 0..4 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        y -= 20.0;
        touch.move_to(y);
    }
    touch.touch(
        |_| touch::Event::FingerLost {
            id: FINGER,
            position: Point::new(150.0, y),
        },
        (150.0, y),
    );
    touch.start = Instant::now();
    touch.frames(60);

    assert!((touch.offset() - 80.0).abs() < 1.0);
}

#[test]
fn no_momentum_without_animations() {
    let _guard = ANIMATIONS.lock().unwrap_or_else(|e| e.into_inner());
    shadcn_iced::animation::set_enabled(false);

    let mut touch = Touch::new();
    let _ = flick(&mut touch);
    touch.start = Instant::now();
    touch.frames(120);
    let offset = touch.offset();

    shadcn_iced::animation::set_enabled(true);

    // Only what the finger scrolled.
    assert!((offset - 80.0).abs() < 1.0, "{offset}");
}
