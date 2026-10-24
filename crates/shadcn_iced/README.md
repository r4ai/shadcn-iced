# shadcn-iced

[![crates.io](https://img.shields.io/crates/v/shadcn-iced.svg)](https://crates.io/crates/shadcn-iced)
[![docs.rs](https://img.shields.io/docsrs/shadcn-iced)](https://docs.rs/shadcn-iced)
[![MIT licensed](https://img.shields.io/crates/l/shadcn-iced.svg)](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE)

[shadcn/ui](https://ui.shadcn.com) components for [iced](https://iced.rs) 0.14: 66 components, from buttons, forms, overlays and data tables to charts and AI agent interfaces, with every shadcn/ui theme preset.

**[Live WebAssembly previews and docs →](https://shadcn-iced.r4ai.dev)**

![dashboard-01 recreated with shadcn-iced](https://raw.githubusercontent.com/r4ai/shadcn-iced/main/docs/screenshots/dashboard-dark.png)

## The crates

| Crate | What it is |
| --- | --- |
| [`shadcn-iced`](https://crates.io/crates/shadcn-iced) (this crate) | The components, ready to use as a library. |
| [`shadcn_iced_core`](https://crates.io/crates/shadcn_iced_core) | The foundation the components are built on: theme tokens, bundled fonts and icons, and headless primitives. Re-exported here as `shadcn_iced::core`. |
| [`cargo-shadcn-iced`](https://crates.io/crates/cargo-shadcn-iced) | A cargo subcommand that copies the source of these components into your project, shadcn/ui style, so you own and edit the code. |

Every component lives in its own file and depends only on `iced`, `shadcn_iced_core` and sibling components. Use this crate when you want the components as a dependency; use `cargo shadcn-iced add` when you want to customize them.

## Installation

```sh
cargo add iced
cargo add shadcn-iced
```

The library is imported as `shadcn_iced`.

## Usage

Import the prelude, pick a theme, and call `.shadcn_iced()` on your application to load the bundled fonts and icons:

```rust,no_run
use shadcn_iced::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Save,
}

fn view(_state: &()) -> iced::Element<'_, Message> {
    // `tab_order` moves the focus with Tab, like in a browser.
    tab_order(iced::widget::center(
        card()
            .title("Create project")
            .description("Deploy your new project in one-click.")
            .footer(button("Deploy").on_press(Message::Save))
            .width(350),
    ))
    .into()
}

fn main() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: Message| {}, view)
        .theme(|_: &()| shadcn_iced::theme::zinc(Mode::Dark).build())
        .shadcn_iced()
        .run()
}
```

This is [`examples/hello`](https://github.com/r4ai/shadcn-iced/tree/main/examples/hello) in the repository (`cargo run -p hello`).

### Themes

Combine any of the 7 base palettes, 17 accent colors, 5 radius presets and the menu styles of shadcn/ui:

```rust
use shadcn_iced::theme::{self, Accent, MenuAccent, MenuColor, Mode, Radius};

let theme: iced::Theme = theme::zinc(Mode::Dark)
    .accent(Accent::Violet)
    .radius_preset(Radius::Medium)
    .menu_color(MenuColor::DefaultTranslucent)
    .menu_accent(MenuAccent::Bold)
    .build();
```

## Feature flags

The features choose which fonts are bundled into your binary; they are forwarded to `shadcn_iced_core`.

| Feature | Default | What it bundles |
| --- | :---: | --- |
| `lucide` | yes | The [Lucide](https://lucide.dev) icon font (~900 KB). |
| `geist` | yes | Geist and Geist Mono, the fonts of shadcn/ui's default preset. |
| `symbols` | yes | The keyboard symbols and arrows Geist lacks (⌘ ⌥ ⌃ ⏎ …), from Noto Sans Symbols (~55 KB). |
| `inter` | no | Inter (~710 KB), an alternative to Geist. |

To ship your own fonts, turn off the defaults and keep what you need:

```sh
cargo add shadcn-iced --no-default-features --features lucide
```

## Minimum supported Rust version

Rust 1.88 (edition 2024).

## Links

- [Website](https://shadcn-iced.r4ai.dev): live previews, props and code for every component
- [API documentation](https://docs.rs/shadcn-iced)
- [Repository](https://github.com/r4ai/shadcn-iced)
- [Changelog](https://github.com/r4ai/shadcn-iced/blob/main/CHANGELOG.md)

## License

Licensed under the [MIT License](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE). The bundled fonts and icons have their own licenses, listed in [`THIRD_PARTY_NOTICES.md`](https://github.com/r4ai/shadcn-iced/blob/main/THIRD_PARTY_NOTICES.md).
