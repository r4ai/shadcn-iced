# shadcn_iced_core

[![crates.io](https://img.shields.io/crates/v/shadcn_iced_core.svg)](https://crates.io/crates/shadcn_iced_core)
[![docs.rs](https://img.shields.io/docsrs/shadcn_iced_core)](https://docs.rs/shadcn_iced_core)
[![MIT licensed](https://img.shields.io/crates/l/shadcn_iced_core.svg)](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE)

The foundation of [shadcn-iced](https://github.com/r4ai/shadcn-iced), the [shadcn/ui](https://ui.shadcn.com) components for [iced](https://iced.rs) 0.14:

- **Theme tokens** ported from shadcn/ui: every base palette, accent color, radius preset and menu style, built into an `iced::Theme`.
- **Fonts and icons**: Geist, Geist Mono, Inter, the [Lucide](https://lucide.dev) icon font and keyboard symbols, bundled and loaded with one call.
- **Headless primitives**: anchored overlays, modal layers, focus and tab order, roving focus, listboxes, box shadows, animation, and the other building blocks of the components.

**[Live WebAssembly previews and docs →](https://shadcn-iced.r4ai.dev)**

## The crates

| Crate | What it is |
| --- | --- |
| [`shadcn-iced`](https://crates.io/crates/shadcn-iced) | The components, ready to use as a library. It re-exports this crate as `shadcn_iced::core`. |
| [`shadcn_iced_core`](https://crates.io/crates/shadcn_iced_core) (this crate) | The tokens, fonts, icons and primitives the components are built on. |
| [`cargo-shadcn-iced`](https://crates.io/crates/cargo-shadcn-iced) | A cargo subcommand that copies the source of the components into your project. |

The components depend only on `iced` and this crate, which is what makes them copy-pasteable. Depend on `shadcn_iced_core` directly when you copy components with `cargo shadcn-iced add`, or when you build your own widgets on the shadcn/ui tokens. If you use the `shadcn-iced` library, you don't need to add this crate yourself.

## Installation

```sh
cargo add iced --features advanced,canvas,image,svg
cargo add shadcn_iced_core
```

## Usage

Build a theme, load the fonts with `.shadcn_iced()`, and read the tokens of the current theme to style your widgets:

```rust,no_run
use iced::widget::{container, row, text};
use shadcn_iced_core::icon::{icon, lucide};
use shadcn_iced_core::{ApplicationExt, Mode, theme};

fn view(_state: &()) -> iced::Element<'_, ()> {
    let badge = container(row![icon(lucide::CHECK), text("Saved").size(14)].spacing(6))
        .padding([4, 10])
        .style(|theme: &iced::Theme| {
            let tokens = theme::tokens(theme);

            container::Style::default()
                .background(tokens.palette.primary)
                .color(tokens.palette.primary_foreground)
                .border(iced::border::rounded(tokens.radius_md()))
        });

    container(badge).center(iced::Fill).into()
}

fn main() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: ()| {}, view)
        .theme(|_: &()| theme::zinc(Mode::Dark).build())
        .shadcn_iced()
        .run()
}
```

Themes combine any of the 7 base palettes, 17 accent colors, 5 radius presets and the menu styles of shadcn/ui:

```rust
use shadcn_iced_core::theme::{self, Accent, MenuAccent, MenuColor, Mode, Radius};

let theme: iced::Theme = theme::zinc(Mode::Dark)
    .accent(Accent::Violet)
    .radius_preset(Radius::Medium)
    .menu_color(MenuColor::DefaultTranslucent)
    .menu_accent(MenuAccent::Bold)
    .build();
```

## Feature flags

The features choose which fonts are bundled into your binary. Every bundled font is also copied into memory when it is loaded, so turn off the ones you don't use.

| Feature | Default | What it bundles |
| --- | :---: | --- |
| `lucide` | yes | The [Lucide](https://lucide.dev) icon font (~900 KB). |
| `geist` | yes | Geist and Geist Mono, the fonts of shadcn/ui's default preset. |
| `symbols` | yes | The keyboard symbols and arrows Geist lacks (⌘ ⌥ ⌃ ⏎ …), from Noto Sans Symbols 2, Noto Sans Symbols and Noto Sans Math (~55 KB). |
| `inter` | no | Inter (~710 KB), an alternative to Geist. |

## Minimum supported Rust version

Rust 1.88 (edition 2024).

## Links

- [Website](https://shadcn-iced.r4ai.dev)
- [API documentation](https://docs.rs/shadcn_iced_core)
- [Repository](https://github.com/r4ai/shadcn-iced)
- [Architecture](https://github.com/r4ai/shadcn-iced/blob/main/docs/architecture.md): the token pipeline and the headless primitives

## License

Licensed under the [MIT License](https://github.com/r4ai/shadcn-iced/blob/main/LICENSE). The bundled fonts and icons have their own licenses, included in the package next to them (`assets/fonts/*-LICENSE.txt`) and listed in [`THIRD_PARTY_NOTICES.md`](https://github.com/r4ai/shadcn-iced/blob/main/THIRD_PARTY_NOTICES.md).
