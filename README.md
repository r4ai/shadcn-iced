# iced-ui

Beautifully designed components for [iced](https://iced.rs), ported from
[shadcn/ui](https://ui.shadcn.com).

Use them as a regular library, **or copy their source into your project and
own it**, just like shadcn/ui:

```sh
cargo iced-ui add button dialog dropdown-menu
```

| | |
|---|---|
| ![Dropdown menu with a submenu, dark mode](docs/screenshots/dropdown-menu-dark.png) | ![Dialog](docs/screenshots/dialog-light.png) |
| ![Charts with the blue accent](docs/screenshots/chart-blue-light.png) | ![Card with the zinc base and violet accent](docs/screenshots/card-violet-dark.png) |

## Components

66 components, covering the whole shadcn/ui registry in its current default
style (**Base UI + Nova**, the `base-nova` preset of `shadcn init`):

accordion · alert · alert-dialog · aspect-ratio · attachment · avatar · badge ·
breadcrumb · bubble · button · button-group · calendar · card · carousel ·
chart · checkbox · code-block · collapsible · combobox · command ·
context-menu · date-picker · dialog · drawer · dropdown-menu · empty · field ·
hover-card · input · input-group · input-otp · item · kbd · label · marker ·
menu · menubar · message · message-scroller · native-select · navigation-menu ·
pagination · popover · progress · questionnaire · radio-group · resizable ·
scroll-area · select · separator · sheet · sidebar · skeleton · slider ·
sonner · spinner · switch · table · tabs · textarea · toast · toggle ·
toggle-group · tooltip · trace · typography

Plus everything shadcn/ui gets from its environment:

- **Themes**: the options of shadcn/ui's `/create` page (except the style):
  every base color (neutral, stone, zinc, mauve, olive, mist, taupe), theme
  (amber, blue, … yellow), chart color, radius preset, menu color (default,
  inverted, translucent) and menu accent (subtle, bold), in light and dark
  mode, generated from shadcn/ui's OKLCH tokens.
- **Fonts and icons**: Geist, Geist Mono, Inter and all 2,179
  [Lucide](https://lucide.dev) icons are bundled; the body and heading fonts
  are configurable.
- **Animations**: overlays, sheets, drawers, accordions, toasts and hover
  states animate like tw-animate-css.
- **Headless primitives** (the part Base UI plays for shadcn/ui): anchored
  floating content with flipping and collision handling, modal layers,
  listboxes with keyboard navigation, disclosures, resizable splits and
  focus rings.

See them all on the website (`site/`), where every
example runs live in the browser (iced compiled to WebAssembly) next to its
code, or run the gallery:

```sh
cargo run -p gallery     # every component, with a theme customizer
cargo run -p hello       # the example below
```

## Getting started

### As a library

```toml
[dependencies]
iced = "0.14"
iced_ui = { git = "https://github.com/r4ai/iced-ui" }
```

```rust
use iced_ui::prelude::*;
use iced_ui::button::Variant;

#[derive(Debug, Clone)]
enum Message {
    Save,
}

fn view(_state: &()) -> iced::Element<'_, Message> {
    card()
        .title("Create project")
        .description("Deploy your new project in one-click.")
        .footer(button("Deploy").on_press(Message::Save))
        .width(350)
        .into()
}

fn main() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: Message| {}, view)
        // Any shadcn/ui preset, e.g. zinc with the blue accent:
        .theme(|_: &()| iced_ui::theme::zinc(Mode::Dark).build())
        // Loads Geist, Inter and the Lucide icons.
        .iced_ui()
        .run()
}
```

### Copy and paste

Install the CLI:

```sh
cargo install --git https://github.com/r4ai/iced-ui cargo-iced-ui
```

Then, in your project:

```sh
cargo add iced --features advanced,canvas,image,svg
cargo add iced_ui_core --git https://github.com/r4ai/iced-ui
cargo iced-ui add button card dialog
```

The components (and the components they use) are written to `src/ui/`, and
`src/ui/mod.rs` is kept up to date. Declare the module with `mod ui;` and use
them like any of your own code:

```rust
mod ui;

fn view(state: &State) -> iced::Element<'_, Message> {
    ui::button::button("Click me").on_press(Message::Click).into()
}
```

Other commands:

```sh
cargo iced-ui list           # every component, with a description
cargo iced-ui add --all      # copy everything
cargo iced-ui diff           # which local copies differ from upstream
cargo iced-ui view dialog    # print the source of a component
```

## Theming

Components read shadcn/ui's design tokens from the iced theme:

```rust
use iced_ui::theme::{self, Accent, BaseColor, Mode};

let theme = theme::Config::new(BaseColor::Stone, Mode::Light)
    .accent(Accent::Orange)
    .radius(6.0)
    .palette(|palette| palette.ring = palette.primary)
    .build();
```

The other options of shadcn/ui's `/create` page are there too (the
gallery's **Customize** sheet shows the code for any combination):

```rust
use iced_ui::theme::{Accent, MenuAccent, MenuColor, Radius};

let theme = theme::Config::new(BaseColor::Neutral, Mode::Dark)
    .accent(Accent::Blue)
    .chart_color(Accent::Rose)
    .radius_preset(Radius::Large)
    .menu_accent(MenuAccent::Bold)
    .menu_color(MenuColor::Inverted)
    .build();

// Fonts (`font` and `fontHeading`): Geist by default, Inter is bundled too.
iced_ui::font::set_sans(iced_ui::font::INTER); // before `.iced_ui()`
iced_ui::font::set_heading(Some(iced_ui::font::GEIST));
```

`Config::build` returns a plain `iced::Theme` whose extended palette is also
derived from the tokens, so built-in iced widgets blend in. Inside a
component, `iced_ui::tokens(theme)` returns the full token set (`background`,
`primary`, `muted-foreground`, `ring`, `chart-1`…, `sidebar-*`, radii).
Plain iced themes (`iced::Theme::Dracula`, …) work too: their tokens are
derived from their palette.

## How it is built

```
crates/
  iced_ui_core/   tokens, themes, fonts, icons, headless primitives
  iced_ui/        the components, one self-contained file each
  cargo-iced-ui/  the copy-and-paste CLI (embeds the component sources)
examples/
  gallery/        a documentation-like showcase of every component
  gallery-web/    the gallery examples compiled to WebAssembly, for the website
site/             the website (Astro), with live previews of every example
```

Each file in `crates/iced_ui/src/components/` only depends on `iced`,
`iced_ui_core` and sibling components through `super::`. That rule is what
makes a component work identically inside the library and copied into your
project; `scripts/check-copy.sh` (run in CI) verifies it by copying every
component into a fresh project and compiling it with `-D warnings`.

### Visual QA

The gallery renders headlessly, so every page can be checked without a
display:

```sh
# Every page, light and dark, to target/snapshots/
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot

# One page, after interacting with it
cargo run -p gallery --bin snapshot -- dropdown-menu:dark --live --click "Open" --hover 700,538
cargo run -p gallery --bin snapshot -- dialog:light --click "Edit Profile"
cargo run -p gallery --bin snapshot -- chart:light --accent blue --chart rose
cargo run -p gallery --bin snapshot -- dropdown-menu:light --menu-color inverted --live --click "Open"

# A frame in the middle of an animation (animations are off by default)
cargo run -p gallery --bin snapshot -- dropdown-menu:light --animations --live --click "Open" --wait 70
```

`GALLERY_PAGE=sonner cargo run -p gallery` opens the gallery on a given page.

### Website

The website (`site/`) shows every example of the gallery with a Preview tab,
where the example runs live, and a Code tab, with its source:

- `cargo run -p gallery --bin site_data` lists the pages and where each
  `example(…)` is called; `pnpm data` turns that into the Code tabs, with the
  `use` declarations, helpers and bindings the example refers to.
- `examples/gallery-web` runs the examples of a page in a single
  `iced::daemon`, one window (canvas) per example, drawn with WebGPU or the
  tiny-skia fallback; `pnpm wasm` builds it to `site/public/wasm/`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <the version in Cargo.lock> --locked
cd site
pnpm install
pnpm data && pnpm wasm   # after changing the examples or the components
pnpm dev                 # http://localhost:4321
```

The `Site` workflow deploys it from `main` to Cloudflare Workers (static
assets, `site/wrangler.jsonc`); it needs the `CLOUDFLARE_API_TOKEN` and
`CLOUDFLARE_ACCOUNT_ID` secrets of the repository.

## Differences with shadcn/ui

iced is not a browser, so a few things differ:

- The style is fixed to Nova; the other styles of shadcn/ui (Vega, Maia,
  Lyra, Mira, Luma, Rhea, Sera) are not ported.
- No backdrop blur: modal backdrops and translucent menus are approximated
  without it.
- Text is clipped rather than ellipsized, and there is no syntax
  highlighting in code blocks.
- Animations are reimplemented natively rather than with CSS: overlays fade
  and zoom in and out (`animate-in`/`animate-out`), sheets and drawers slide
  from their edge, accordions and collapsibles expand, toasts slide in, the
  switch thumb slides and hover/press colors transition. Turn them off with
  `iced_ui::animation::set_enabled(false)` (e.g. for reduced motion).
- Keyboard focus: iced only focuses text inputs, so focus rings are shown on
  inputs, input groups and OTP fields; menus, selects and command palettes
  support <kbd>↑</kbd>/<kbd>↓</kbd>/<kbd>Enter</kbd>/<kbd>Esc</kbd>.
- Some state that Base UI keeps internally (dialogs, selected values, the
  current month of a calendar…) lives in your application state, the Elm way.
  Popovers, dropdown/context menus, selects, hover cards, tooltips,
  collapsibles and accordions manage themselves.
- `form` and `direction` are React-specific and have no counterpart; use
  `field` for forms.

## License

MIT. The design is ported from shadcn/ui (MIT); Geist and Inter are under the OFL and
Lucide under the ISC license. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

## 日本語

shadcn/ui のデザインを iced で再現した UI ライブラリです。
`iced_ui` をライブラリとして使うことも、`cargo iced-ui add button` で
コンポーネントのソースを自分のプロジェクトにコピーして自由に改変することも
できます。テーマ（全ベースカラー・アクセントカラー、ライト/ダーク）、
最新のデフォルトスタイル（Base UI + Nova）、/create 相当のテーマ設定、Geist / Inter フォント、Lucide アイコン、アニメーション、Base UI に相当するヘッドレスな
プリミティブ（ポップオーバーの配置、モーダル、キーボード操作できるリスト等）
も同梱しています。`cargo run -p gallery` で全コンポーネントを確認できます。
ウェブサイト（`site/`）では、WebAssembly にコンパイルした
iced で各サンプルをブラウザ上で動かしながら、そのコードを確認できます。
