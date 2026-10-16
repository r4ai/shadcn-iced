//! Renders every page of the gallery headlessly, in light and dark mode,
//! to catch layout panics and broken views.
use gallery::{Gallery, PAGES};
use iced_ui::Mode;

use iced_test::simulator::Simulator;

#[test]
fn every_page_renders() {
    iced_ui::animation::set_enabled(false);

    for (index, page) in PAGES.iter().enumerate() {
        for mode in [Mode::Light, Mode::Dark] {
            let gallery = Gallery::new(index, mode);
            let theme = gallery.theme();
            let mut ui = Simulator::with_size(gallery::settings(), (1100.0, 820.0), gallery.view());

            ui.snapshot(&theme)
                .unwrap_or_else(|error| panic!("{} ({mode:?}): {error}", page.id));
        }
    }
}

#[test]
fn page_ids_are_unique() {
    let mut ids: Vec<_> = PAGES.iter().map(|page| page.id).collect();
    ids.sort_unstable();
    ids.dedup();

    assert_eq!(ids.len(), PAGES.len());
}

#[test]
fn toasts_expire_after_their_duration() {
    use gallery::Message;
    use gallery::pages::DemoMessage;
    use iced::time::{Duration, Instant};
    use iced::{Event, window};

    iced_ui::animation::set_enabled(false);

    let mut gallery = Gallery::new(Gallery::page_by_id("sonner").unwrap(), Mode::Light);
    let _ = gallery.update(Message::Demo(DemoMessage::Toast(
        iced_ui::sonner::Kind::Success,
    )));

    let start = Instant::now();
    let mut ui = Simulator::with_size(gallery::settings(), (1100.0, 820.0), gallery.view());

    let redraw = |at| Event::Window(window::Event::RedrawRequested(at));
    let _ = ui.simulate([redraw(start), redraw(start + Duration::from_secs(2))]);
    let early: Vec<_> = ui.into_messages().collect();
    assert!(early.is_empty(), "dismissed too early: {early:?}");

    let mut ui = Simulator::with_size(gallery::settings(), (1100.0, 820.0), gallery.view());
    let _ = ui.simulate([redraw(start), redraw(start + Duration::from_secs(5))]);
    let messages: Vec<_> = ui.into_messages().collect();

    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Demo(DemoMessage::DismissToast(0)))),
        "{messages:?}"
    );
}
