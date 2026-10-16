//! Renders every page of the gallery headlessly, in light and dark mode,
//! to catch layout panics and broken views.
use gallery::{Gallery, PAGES};
use iced_ui::Mode;

use iced_test::simulator::Simulator;

#[test]
fn every_page_renders() {
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
