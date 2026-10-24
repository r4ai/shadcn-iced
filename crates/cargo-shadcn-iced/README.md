# cargo-shadcn-iced

[![crates.io](https://img.shields.io/crates/v/cargo-shadcn-iced.svg)](https://crates.io/crates/cargo-shadcn-iced)
[![MIT licensed](https://img.shields.io/crates/l/cargo-shadcn-iced.svg)](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE)

Copy [shadcn-iced](https://github.com/r4ai/shadcn-iced) components into your project and own the code, like `npx shadcn add` for [shadcn/ui](https://ui.shadcn.com).

**[Browse the components →](https://shadcn-iced.r4ai.dev)**

## The crates

| Crate | What it is |
| --- | --- |
| [`shadcn-iced`](https://crates.io/crates/shadcn-iced) | The components, ready to use as a library. |
| [`shadcn_iced_core`](https://crates.io/crates/shadcn_iced_core) | The tokens, fonts, icons and primitives the components are built on. Copied components depend on it. |
| [`cargo-shadcn-iced`](https://crates.io/crates/cargo-shadcn-iced) (this crate) | A cargo subcommand that copies the source of the components into your project. |

The CLI embeds the components of the `shadcn-iced` release of the same version, so it works offline.

## Installation

```sh
cargo install cargo-shadcn-iced
```

Or download a prebuilt binary with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall cargo-shadcn-iced
```

## Usage

Add the dependencies of the components to your project:

```sh
cargo add iced --features advanced,canvas,image,svg
cargo add shadcn_iced_core
```

Then copy the components you need. The components they use are copied too:

```sh
cargo shadcn-iced add button card dialog dropdown-menu
```

They land in `src/ui/`, one file per component, and `src/ui/mod.rs` declares them. Declare the module, import what you need, and load the fonts:

```rust,ignore
mod ui;

use shadcn_iced_core::{ApplicationExt, Mode, theme};
use ui::button::button;

#[derive(Debug, Clone)]
enum Message {
    Save,
}

fn view(_state: &()) -> iced::Element<'_, Message> {
    iced::widget::center(button("Save").on_press(Message::Save)).into()
}

fn main() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: Message| {}, view)
        .theme(|_: &()| theme::zinc(Mode::Dark).build())
        .shadcn_iced()
        .run()
}
```

### Commands

```text
cargo shadcn-iced list                List the available components
cargo shadcn-iced add <NAME>...       Copy components (and the components they use)
cargo shadcn-iced add --all           Copy every component
cargo shadcn-iced diff <NAME>...      Show which local copies differ from the registry
cargo shadcn-iced view <NAME>         Print the source of a component

Options:
    --dir <PATH>                      Where components go [default: src/ui]
    --overwrite                       Replace files that already exist
```

Component names accept both `dropdown-menu` and `dropdown_menu`.

## Minimum supported Rust version

Rust 1.88 (edition 2024).

## Links

- [Website](https://shadcn-iced.r4ai.dev): previews and code of every component
- [Repository](https://github.com/r4ai/shadcn-iced)
- [`shadcn_iced_core` API documentation](https://docs.rs/shadcn_iced_core)

## License

Licensed under the [MIT License](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE).
