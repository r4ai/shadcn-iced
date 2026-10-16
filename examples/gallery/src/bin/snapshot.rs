//! Renders gallery pages headlessly to PNG files, for visual QA.
//!
//! ```sh
//! cargo run -p gallery --bin snapshot -- [page-id[:mode]...] [--click "Label"]...
//! ```
//!
//! Snapshots are written to `target/snapshots/`.
use gallery::{Gallery, Message, PAGES};
use iced_ui::Mode;

use iced_test::simulator::Simulator;
use std::path::PathBuf;

fn main() {
    let mut pages = Vec::new();
    let mut clicks = Vec::new();
    let mut size = (1100.0, 820.0);
    let mut args = std::env::args().skip(1);
    let mut suffix = String::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--click" => clicks.push(args.next().expect("label")),
            "--size" => {
                let value = args.next().expect("WxH");
                let (w, h) = value.split_once('x').expect("WxH");
                size = (w.parse().unwrap(), h.parse().unwrap());
            }
            "--suffix" => suffix = args.next().expect("suffix"),
            _ => pages.push(arg),
        }
    }

    if pages.is_empty() {
        pages = PAGES.iter().map(|page| page.id.to_owned()).collect();
    }

    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/snapshots");
    std::fs::create_dir_all(&out).unwrap();

    for spec in pages {
        let (id, modes) = match spec.split_once(':') {
            Some((id, "dark")) => (id.to_owned(), vec![Mode::Dark]),
            Some((id, "light")) => (id.to_owned(), vec![Mode::Light]),
            _ => (spec.clone(), vec![Mode::Light, Mode::Dark]),
        };

        let Some(index) = Gallery::page_by_id(&id) else {
            eprintln!("unknown page: {id}");
            continue;
        };

        for mode in modes {
            let mut gallery = Gallery::new(index, mode);

            for label in &clicks {
                let messages: Vec<Message> = {
                    let mut ui = Simulator::with_size(gallery::settings(), size, gallery.view());
                    if let Err(error) = ui.click(label.as_str()) {
                        eprintln!("click {label:?}: {error}");
                    }
                    ui.into_messages().collect()
                };

                for message in messages {
                    let _ = gallery.update(message);
                }
            }

            let theme = gallery.theme();
            let mut ui = Simulator::with_size(gallery::settings(), size, gallery.view());
            let snapshot = ui.snapshot(&theme).expect("snapshot");

            let mode = match mode {
                Mode::Light => "light",
                Mode::Dark => "dark",
            };
            let path = out.join(format!("{id}{suffix}-{mode}.png"));
            let _ = std::fs::remove_file(path.with_file_name(format!("{id}{suffix}-{mode}-tiny-skia.png")));
            snapshot.matches_image(&path).expect("write snapshot");
            println!("{}", path.with_file_name(format!("{id}{suffix}-{mode}-tiny-skia.png")).display());
        }
    }
}
