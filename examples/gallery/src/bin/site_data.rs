//! Prints the pages of the gallery and their examples as JSON, for the
//! website (`site/`), which shows the source of each example next to its
//! live preview.
//!
//! ```sh
//! cargo run -p gallery --bin site_data > site/src/data/gallery.json
//! ```
use gallery::{Gallery, PAGES};
use shadcn_iced::Mode;

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

fn main() {
    let gallery = Gallery::new(0, Mode::Light);

    let pages: Vec<String> = PAGES
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let examples: Vec<String> = gallery::examples(&gallery, index)
                .iter()
                .map(|example| {
                    format!(
                        "{{\"title\":{},\"file\":{},\"line\":{},\"column\":{}}}",
                        string(&example.title),
                        string(&example.location.file().replace('\\', "/")),
                        example.location.line(),
                        example.location.column(),
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

    println!("{{\"pages\":[\n{}\n]}}", pages.join(",\n"));
}
