# Architecture

This document describes the internal structure of `shadcn-iced`, how components work both as a library and as copied source files, and where the implementation differs from the web version of [shadcn/ui](https://ui.shadcn.com).

## Workspace Layout

```text
crates/
  shadcn_iced_core/   Design tokens, themes, fonts, icons, animations, and headless primitives
  shadcn_iced/        UI components (one self-contained module per component)
  cargo-shadcn-iced/  CLI tool that embeds and copies component sources into user projects
examples/
  gallery/            Native desktop showcase and headless snapshot generator
  gallery-web/        WebAssembly runtime powering the live previews on the website
  hello/              Minimal starter application
site/                 Documentation website (TanStack Start, Solid, Vite)
conformance/          Comparison with the latest shadcn/ui: a shadcn/ui app, Playwright
                      tests and a headless iced runner (`iced/`) sharing scenarios
scripts/
  check-copy.sh       CI script verifying that copied components compile standalone
```

## Library and Copy-and-Paste Model

`shadcn-iced` is structured so that every component works identically whether it is imported from the `shadcn_iced` crate or copied into a user's repository via `cargo shadcn-iced add`.

### Shared Foundation (`shadcn_iced_core`)

Both consumption modes depend on `shadcn_iced_core`, which provides the non-visual and foundational building blocks:

- **Design tokens and themes** (`theme`)
- **Shared style constants** (`style`)
- **Bundled fonts and icons** (`font`, `icon`)
- **Transition and animation utilities** (`animation`)
- **Headless widget primitives** (`primitive`)

### Self-Contained Component Modules (`shadcn_iced`)

Each component in `crates/shadcn_iced/src/components/` lives in a single `.rs` file and follows a strict dependency rule:

- A component may only import from `iced`, `shadcn_iced_core`, and sibling components via `super::<component>`.
- A component never references `crate::` or `shadcn_iced::` in code.

Because of this constraint, copying a component file (along with any `super::` siblings it references) into a `src/ui/` directory produces a valid module hierarchy that compiles without the `shadcn_iced` crate.

### CLI Registry (`cargo-shadcn-iced`)

At build time (`crates/cargo-shadcn-iced/build.rs`), the CLI scans `crates/shadcn_iced/src/components/` and embeds every component's source string into the binary. When `cargo shadcn-iced add <name>` runs:

1. It parses `super::<sibling>` references to resolve transitive component dependencies.
2. It rewrites `//! ```no_run` doctest attributes to `//! ```ignore` so that copied files do not require `shadcn_iced` during `cargo test --doc`.
3. It writes each resolved `.rs` file into the target directory (`src/ui` by default) and ensures `pub mod <name>;` is declared in `src/ui/mod.rs`.

## Core Subsystems

### Theme and Token Bridge

`shadcn/ui` defines its appearance through CSS variables (`--background`, `--foreground`, `--primary`, `--muted`, `--accent`, `--destructive`, `--border`, `--input`, `--ring`, `--sidebar-*`, `--chart-1..5`, and `--radius`). `shadcn_iced_core::theme` mirrors these variables as a `Palette` struct paired with a corner radius (`Tokens`).

- **OKLCH Presets**: All base colors, accent themes, and chart palettes are generated from `shadcn/ui`'s OKLCH token definitions across light and dark modes.
- **`iced::Theme` Integration**: `theme::Config::build()` returns a standard `iced::Theme` (`Theme::Custom`). Its `iced` extended palette is derived from the `shadcn/ui` tokens so built-in `iced` widgets match the surrounding theme, while the full `Tokens` struct is stored in an internal registry keyed by the theme.
- **Token Retrieval**: Components call `shadcn_iced::tokens(theme)` during rendering to read the active `Tokens`. If a standard built-in theme (such as `iced::Theme::Dracula`) is passed instead, `tokens(theme)` derives a compatible `Tokens` set from that theme's extended palette.

### Headless Primitives

The `shadcn_iced_core::primitive` module provides the behavioral layer analogous to Base UI / Radix UI in the web ecosystem:

| Primitive | Purpose |
| :--- | :--- |
| `anchored` | Positions floating content (`Popover`, `DropdownMenu`, `Select`, `Tooltip`, `HoverCard`) relative to a trigger widget with configurable `Side` and `Align`, viewport collision detection, and automatic flipping. |
| `layer` | Renders modal backdrops and overlay stacks (`Dialog`, `AlertDialog`, `Sheet`, `Drawer`, `Toaster`) above the main layout tree. |
| `listbox` | Manages keyboard navigation (<kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Enter</kbd>, <kbd>Esc</kbd>) and item selection for menus, selects, comboboxes, and command palettes. |
| `disclosure` | Handles expand/collapse state and layout transitions for `Accordion` and `Collapsible`. |
| `focus` | Detects when a widget subtree holds keyboard focus (`focus_within`) to draw focus rings around composite inputs (`InputGroup`, `InputOtp`). |
| `pressable` | Provides pointer press, release, and hover interaction tracking for custom interactive surfaces. |
| `split` | Implements draggable, resizable pane dividers used by `Resizable`. |
| `toaster` | Stacks, animates, expands, times out and swipes away the toasts of `Sonner` and `Toast`, porting sonner's transitions (`translateY`, `scale`, height and opacity, `400ms ease`). A `Motion` sets how toasts enter, leave and peek out of the collapsed stack, which is all that differs between the two. |
| `plot` | The geometry of `Chart`, like the parts of d3 that Recharts uses: band and point scales, nice ticks, stacking, bar layout, natural curves, pies, hit testing and tooltip placement. Not a widget; the chart keeps its configuration and its drawing. |
| `track` | Lays out the items of a `Carousel` on a track that slides to the current item and can be dragged (swiped). |
| `animated_width` | Animates the width of its content when it changes, like the `Sidebar` collapsing to icons. |
| `wheel` | Wraps a whole view to make wheel scrolling smooth (wheel notches are spread over a few frames) and horizontal with <kbd>Shift</kbd>, for every scrollable under it. |

`anchored` also keeps a submenu open while the pointer crosses the triangle between its trigger and the submenu (a pointer grace area, or "safe triangle", as in Radix), so that the pointer can move diagonally to it.

### Native Animations

Because `iced` does not use a CSS engine, animations from `tw-animate-css` are implemented natively in `shadcn_iced_core::animation`:

- **Stateful Transitions**: Animated widgets store a `Transition` in their internal widget state and request redraws until the transition completes, evaluated using CSS `cubic-bezier` timing functions.
- **Spatial Transforms**: Scale (`zoom-in-95`) and translation (`slide-in-from-*` for overlays, sheets, drawers, and toasts) are applied at draw time via `iced::Transformation`.
- **Opacity (`Fade`)**: Fade transitions render a widget subtree using a derived copy of the active theme with alpha-adjusted color tokens.
- **Global Toggle**: Calling `shadcn_iced::animation::set_enabled(false)` disables all transitions globally for reduced-motion preferences or deterministic visual testing.

## WebAssembly Previews (`examples/gallery-web`)

The documentation website (`site/`) renders live interactive widgets rather than static images:

- `examples/gallery-web` compiles the `gallery` examples to `wasm32-unknown-unknown`.
- A single `iced::daemon` instance runs per documentation page. Each example on the page is mounted as an independent `iced` window backed by its own HTML `<canvas>`, drawn with WebGPU (or the `tiny-skia` software renderer as a fallback).
- The web page communicates with the WASM runtime via exported bindings (`mount`, `unmount`, `set_theme`, `theme_code`), while the WASM runtime notifies the page of natural content height changes (`shadcnIcedPreviewResized`) and, during the dispatch of each wheel event, whether an example used it (`shadcnIcedPreviewWheel`), so the browser scrolls the page natively over a preview unless the example scrolls itself.

## Differences from Web `shadcn/ui`

The [conformance suite](../conformance/README.md) measures how far every component is from the latest `shadcn/ui`, in snapshots and behavior; its `baseline.json` lists the accepted scores and the known behavioral differences. Text is shaped with kerning (iced's `advanced-shaping` feature, enabled by `shadcn_iced_core`) so that it measures like in browsers.

Because `iced` is a native retained/immediate-mode GUI library rather than a browser DOM environment, several aspects differ from the React implementation of `shadcn/ui`:

- **Style Preset**: Only the default **Nova** (`base-nova`) style is implemented; alternative `shadcn/ui` visual styles (Vega, Maia, Lyra, Mira, Luma, Rhea, Sera) are not ported.
- **Application vs. Widget State**: Following `iced`'s Elm architecture, application-level state—such as whether a `Dialog` is open, the selected value of a form control, or the visible month of a `Calendar`—is owned by your application state and updated via messages. Transient interaction state for `Popover`, `DropdownMenu`, `ContextMenu`, `Select`, `HoverCard`, `Tooltip`, `Collapsible`, and `Accordion` is managed internally by the widgets.
- **Backdrop Blur**: `iced` does not support real-time backdrop blur filters; modal backdrops and translucent menus approximate the appearance using semi-transparent fills.
- **Text Overflow and Highlighting**: Overflowing text is clipped rather than ellipsized, and `code_block` displays monospace code without built-in syntax highlighting.
- **Keyboard Focus**: `iced` tracks keyboard focus primarily on text inputs, so focus rings are displayed on `input`, `input_group`, and `input_otp`. Open menus, selects, and command palettes handle arrow-key and <kbd>Enter</kbd>/<kbd>Esc</kbd> navigation directly.
- **React-Specific APIs**: `form` (React Hook Form integration) and `direction` have no equivalent; use `field`, `field_set`, and `field_separator` to structure forms.
