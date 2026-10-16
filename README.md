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

62 components, covering the shadcn/ui registry (`new-york-v4`):

accordion · alert · alert-dialog · aspect-ratio · attachment · avatar · badge ·
breadcrumb · bubble · button · button-group · calendar · card · carousel ·
chart · checkbox · collapsible · combobox · command · context-menu ·
date-picker · dialog · drawer · dropdown-menu · empty · field · hover-card ·
input · input-group · input-otp · item · kbd · label · marker · menu · menubar ·
message · message-scroller · native-select · navigation-menu · pagination ·
popover · progress · radio-group · resizable · scroll-area · select ·
separator · sheet · sidebar · skeleton · slider · sonner · spinner · switch ·
table · tabs · textarea · toggle · toggle-group · tooltip · typography

Plus everything shadcn/ui gets from its environment:

- **Themes**: every base color (neutral, stone, zinc, mauve, olive, mist,
  taupe) and accent (amber, blue, … yellow) in light and dark mode, generated
  from shadcn/ui's OKLCH tokens, and a configurable radius.
- **Fonts and icons**: Geist, Geist Mono and all 2,179
  [Lucide](https://lucide.dev) icons are bundled.
- **Headless primitives** (the part Radix UI plays for shadcn/ui): anchored
  floating content with flipping and collision handling, modal layers,
  listboxes with keyboard navigation, disclosures, resizable splits and
  focus rings.

Run the gallery to see them all:

```sh
cargo run -p gallery     # every component
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
cargo run -p gallery --bin snapshot -- chart:light --accent blue

# A frame in the middle of an animation (animations are off by default)
cargo run -p gallery --bin snapshot -- dropdown-menu:light --animations --live --click "Open" --wait 70
```

`GALLERY_PAGE=sonner cargo run -p gallery` opens the gallery on a given page.

## Differences with shadcn/ui

iced is not a browser, so a few things differ:

- Animations are reimplemented natively rather than with CSS: overlays fade
  and zoom in and out (`animate-in`/`animate-out`), sheets and drawers slide
  from their edge, accordions and collapsibles expand, toasts slide in, the
  switch thumb slides and hover/press colors transition. Turn them off with
  `iced_ui::animation::set_enabled(false)` (e.g. for reduced motion).
- Keyboard focus: iced only focuses text inputs, so focus rings are shown on
  inputs, input groups and OTP fields; menus, selects and command palettes
  support <kbd>↑</kbd>/<kbd>↓</kbd>/<kbd>Enter</kbd>/<kbd>Esc</kbd>.
- Some state that Radix keeps internally (dialogs, selected values, the
  current month of a calendar…) lives in your application state, the Elm way.
  Popovers, dropdown/context menus, selects, hover cards, tooltips,
  collapsibles and accordions manage themselves.
- `form` and `direction` are React-specific and have no counterpart; use
  `field` for forms.

## License

MIT. The design is ported from shadcn/ui (MIT); Geist is under the OFL and
Lucide under the ISC license. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

## 日本語

shadcn/ui のデザインを iced で再現した UI ライブラリです。
`iced_ui` をライブラリとして使うことも、`cargo iced-ui add button` で
コンポーネントのソースを自分のプロジェクトにコピーして自由に改変することも
できます。テーマ（全ベースカラー・アクセントカラー、ライト/ダーク）、
Geist フォント、Lucide アイコン、アニメーション、Radix UI に相当するヘッドレスな
プリミティブ（ポップオーバーの配置、モーダル、キーボード操作できるリスト等）
も同梱しています。`cargo run -p gallery` で全コンポーネントを確認できます。
