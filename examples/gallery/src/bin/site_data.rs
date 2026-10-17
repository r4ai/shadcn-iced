//! Prints the pages of the gallery and their examples as JSON, for the
//! website (`site/`), which shows the source of each example next to its
//! live preview, and sizes the preview before it runs with the heights of
//! the example at a few widths (see [`WIDTHS`]).
//!
//! ```sh
//! cargo run -p gallery --bin site_data > site/src/data/gallery.json
//! ```
use gallery::{CHARTS, Gallery, PAGES, Page};
use shadcn_iced::Mode;

use iced::advanced::layout::Limits;
use iced::advanced::renderer::Headless;
use iced::advanced::widget::Tree;
use iced::widget::{container, space};
use iced::{Element, Fill, Size};
use iced_test::simulator::Simulator;
use std::borrow::Cow;

/// The widths of a preview (its frame, padding included) that the heights
/// of the examples are measured at: phones, the column of the docs (638px
/// from a tablet up) and the cards of the home page and the charts.
const WIDTHS: [f32; 11] = [
    300.0, 320.0, 340.0, 360.0, 380.0, 400.0, 440.0, 480.0, 560.0, 638.0, 720.0,
];

/// The padding of a preview (`PADDING` of `gallery-web`).
const PADDING: f32 = 40.0;

/// The emoji that `gallery-web` adds to the fonts of the gallery.
const EMOJI: &[u8] = include_bytes!("../../../../conformance/fonts/NotoEmoji-Regular.ttf");

/// A renderer to lay out the examples with the fonts of the previews.
fn renderer() -> iced::Renderer {
    let mut settings = gallery::settings();
    settings.fonts.push(Cow::Borrowed(EMOJI));
    let (font, size) = (settings.default_font, settings.default_text_size);

    // A simulator loads the fonts, for every renderer.
    let _ = Simulator::<'_, ()>::with_size(settings, (1.0, 1.0), space());

    // Only the layout matters: tiny-skia starts at once, wgpu takes a device.
    iced::futures::executor::block_on(iced::Renderer::new(font, size, Some("tiny-skia")))
        .expect("headless renderer")
}

/// The height of the frame of a preview of an example, `width` wide, as
/// `gallery-web` lays it out: centered, at its natural height (see its
/// `natural`). An example that fills its height has none.
fn height(
    renderer: &iced::Renderer,
    gallery: &Gallery,
    page: &Page,
    example: usize,
    width: f32,
) -> Option<f32> {
    let mut example: Element<'_, gallery::Message> =
        container(gallery::view_example(gallery, page, example))
            .center_x(Fill)
            .into();
    let mut tree = Tree::new(example.as_widget());
    let limits = Limits::new(Size::ZERO, Size::new(width - 2.0 * PADDING, f32::INFINITY));

    let height = example
        .as_widget_mut()
        .layout(&mut tree, renderer, &limits)
        .size()
        .height;

    height.is_finite().then(|| (height + 2.0 * PADDING).ceil())
}

fn heights(renderer: &iced::Renderer, gallery: &Gallery, page: &Page, example: usize) -> String {
    let heights: Vec<String> = WIDTHS
        .iter()
        .map(
            |width| match height(renderer, gallery, page, example, *width) {
                Some(height) => height.to_string(),
                None => "null".to_owned(),
            },
        )
        .collect();

    format!("[{}]", heights.join(","))
}

fn string(value: &str) -> String {
    let mut out = String::from("\"");

    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }

    out.push('"');
    out
}

fn pages(renderer: &iced::Renderer, gallery: &Gallery, pages: &[Page]) -> String {
    let pages: Vec<String> = pages
        .iter()
        .map(|page| {
            let examples: Vec<String> = gallery::examples(gallery, page)
                .iter()
                .enumerate()
                .map(|(index, example)| {
                    format!(
                        "{{\"title\":{},\"file\":{},\"line\":{},\"column\":{},\"heights\":{}}}",
                        string(&example.title),
                        string(&example.location.file().replace('\\', "/")),
                        example.location.line(),
                        example.location.column(),
                        heights(renderer, gallery, page, index),
                    )
                })
                .collect();

            format!(
                "{{\"id\":{},\"title\":{},\"description\":{},\"examples\":[{}]}}",
                string(page.id),
                string(page.title),
                string(page.description),
                examples.join(","),
            )
        })
        .collect();

    pages.join(",\n")
}

fn main() {
    // The first frame, as it settles.
    shadcn_iced::animation::set_enabled(false);

    let renderer = renderer();
    let gallery = Gallery::new(0, Mode::Light);
    let widths: Vec<String> = WIDTHS.iter().map(f32::to_string).collect();

    println!(
        "{{\"widths\":[{}],\"pages\":[\n{}\n],\"charts\":[\n{}\n]}}",
        widths.join(","),
        pages(&renderer, &gallery, PAGES),
        pages(&renderer, &gallery, CHARTS),
    );
}
