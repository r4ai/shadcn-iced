//! Renders gallery pages headlessly to PNG files, for visual QA.
//!
//! ```sh
//! cargo run -p gallery --bin snapshot -- [page-id[:mode]...] [ACTIONS] [--live ACTIONS]
//! ```
//!
//! Actions:
//! - `--click "Label"`: clicks the text widget with that content,
//! - `--at X,Y`: clicks at a logical position,
//! - `--hover X,Y`: moves the cursor to a logical position,
//! - `--type "text"`: types some text,
//! - `--key escape|enter|tab|down|up`: taps a key.
//!
//! Actions before `--live` produce messages that update the gallery (the view
//! is rebuilt after each one). Actions after `--live` are performed on the
//! final interface right before the snapshot, so internal widget state (focus,
//! hover, self-managed overlays) is captured.
//!
//! Snapshots are written to `target/snapshots/`.
use gallery::{Gallery, Message, PAGES};
use iced_ui::Mode;

use iced::keyboard::{Key, key::Named};
use iced::{Point, mouse};
use iced_test::simulator::{self, Simulator};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum Action {
    Click(String),
    At(Point),
    Hover(Point),
    Type(String),
    Key(Key),
}

fn point(value: &str) -> Point {
    let (x, y) = value.split_once(',').expect("X,Y");
    Point::new(x.trim().parse().unwrap(), y.trim().parse().unwrap())
}

fn key(value: &str) -> Key {
    match value {
        "escape" => Key::Named(Named::Escape),
        "enter" => Key::Named(Named::Enter),
        "tab" => Key::Named(Named::Tab),
        "down" => Key::Named(Named::ArrowDown),
        "up" => Key::Named(Named::ArrowUp),
        "space" => Key::Named(Named::Space),
        other => Key::Character(other.into()),
    }
}

fn perform(ui: &mut Simulator<'_, Message>, action: &Action) {
    match action {
        Action::Click(label) => {
            if let Err(error) = ui.click(label.as_str()) {
                eprintln!("click {label:?}: {error}");
            }
        }
        Action::At(position) => {
            ui.point_at(*position);
            let _ = ui.simulate(simulator::click());
        }
        Action::Hover(position) => {
            ui.point_at(*position);
            let _ = ui.simulate([iced::Event::Mouse(mouse::Event::CursorMoved {
                position: *position,
            })]);
        }
        Action::Type(text) => {
            let _ = ui.typewrite(text);
        }
        Action::Key(key) => {
            let _ = ui.tap_key(key.clone());
        }
    }
}

fn main() {
    let mut pages = Vec::new();
    let mut before = Vec::new();
    let mut live = Vec::new();
    let mut is_live = false;
    let mut size = (1100.0, 820.0);
    let mut suffix = String::new();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        let action = match arg.as_str() {
            "--click" => Some(Action::Click(args.next().expect("label"))),
            "--at" => Some(Action::At(point(&args.next().expect("X,Y")))),
            "--hover" => Some(Action::Hover(point(&args.next().expect("X,Y")))),
            "--type" => Some(Action::Type(args.next().expect("text"))),
            "--key" => Some(Action::Key(key(&args.next().expect("key")))),
            "--live" => {
                is_live = true;
                None
            }
            "--size" => {
                let value = args.next().expect("WxH");
                let (w, h) = value.split_once('x').expect("WxH");
                size = (w.parse().unwrap(), h.parse().unwrap());
                None
            }
            "--suffix" => {
                suffix = args.next().expect("suffix");
                None
            }
            _ => {
                pages.push(arg);
                None
            }
        };

        if let Some(action) = action {
            if is_live { live.push(action) } else { before.push(action) }
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

            for action in &before {
                let messages: Vec<Message> = {
                    let mut ui = Simulator::with_size(gallery::settings(), size, gallery.view());
                    perform(&mut ui, action);
                    ui.into_messages().collect()
                };

                for message in messages {
                    let _ = gallery.update(message);
                }
            }

            let theme = gallery.theme();
            let mut ui = Simulator::with_size(gallery::settings(), size, gallery.view());

            for action in &live {
                perform(&mut ui, action);
            }

            let snapshot = ui.snapshot(&theme).expect("snapshot");

            let mode = match mode {
                Mode::Light => "light",
                Mode::Dark => "dark",
            };
            let name = format!("{id}{suffix}-{mode}");
            let written = out.join(format!("{name}-tiny-skia.png"));
            let _ = std::fs::remove_file(&written);
            snapshot.matches_image(out.join(format!("{name}.png"))).expect("write snapshot");
            println!("{}", written.display());
        }
    }
}
