# Architecture

`shadcn-iced` ports the design system and component model of [shadcn/ui](https://ui.shadcn.com) to [`iced`](https://github.com/iced-rs/iced) (`0.14`).

---

## High-Level Overview

`shadcn-iced` separates its core foundation from its component catalog so every component file stays self-contained and portable.

```mermaid
flowchart TD
    subgraph Workspace["shadcn-iced Workspace"]
        Core["shadcn_iced_core (Tokens, Themes, Fonts, Primitives)"]
        Lib["shadcn_iced (66 Styled Components)"]
        CLI["cargo-shadcn-iced (CLI Source Copier)"]
        Apps["examples/dashboard, examples/settings (Example Apps)"]
        Gallery["examples/gallery (Desktop Showcase & Snapshot CLI)"]
        Web["examples/gallery-web (WASM Multi-Canvas Previews)"]
        Conf["conformance/iced (Scenario Runner vs. React shadcn/ui)"]
    end

    Core --> Lib
    Lib --> Apps
    Lib --> Gallery
    Apps --> Gallery
    Lib --> Web
    Gallery --> Web
    Lib -. "embeds components/*.rs" .-> CLI
    Lib --> Conf
```

### Two Consumption Modes

Every file in `crates/shadcn_iced/src/components/*.rs` imports only from `iced`, `shadcn_iced_core`, and sibling components (`super::<name>`).

```mermaid
flowchart LR
    subgraph Mode1["1. Library Mode"]
        App1["Your App"] --> ShadcnIced["shadcn_iced"]
        ShadcnIced --> Core1["shadcn_iced_core"]
    end

    subgraph Mode2["2. Copy-and-Own Mode"]
        CLI2["cargo shadcn-iced add"] -. "copies .rs files" .-> LocalUI["src/ui/*.rs"]
        App2["Your App"] --> LocalUI
        LocalUI --> Core2["shadcn_iced_core"]
    end
```

- **Library Mode**
  - Depend on `shadcn_iced` directly in `Cargo.toml`
  - Import components via `use shadcn_iced::prelude::*;`
- **Copy-and-Own Mode (CLI)**
  - Depend only on `shadcn_iced_core` and `iced`
  - Copy component source files into `src/ui/` with `cargo shadcn-iced add <name>`
  - Customize layout, styles, or behavior directly in your repository

---

## Workspace Layout

| Path | Package / Role | Description |
| :--- | :--- | :--- |
| `crates/shadcn_iced_core` | `shadcn_iced_core` | Design tokens, OKLCH-converted palettes, theme builder,<br/>bundled fonts & icons, animations, and 17 headless primitives. |
| `crates/shadcn_iced` | `shadcn_iced` | All 66 styled components (`src/components/*.rs`),<br/>re-exports of `shadcn_iced_core`, and `shadcn_iced::prelude`. |
| `crates/cargo-shadcn-iced` | `cargo-shadcn-iced` | CLI binary (`cargo shadcn-iced`) that embeds `components/*.rs`<br/>and copies selected components with transitive siblings. |
| `examples/hello` | `hello` | Minimal single-file app (`cargo run -p hello`)<br/>demonstrating the `README.md` quick start. |
| `examples/dashboard` | `dashboard` | The `dashboard-01` block of `shadcn/ui` (sidebar, metrics, chart,<br/>data table) as a library (state, `update`, `view`) and a `main.rs`. |
| `examples/settings` | `settings` | An app's settings (profile, account, appearance, notifications,<br/>display), structured like `dashboard`. |
| `examples/gallery` | `gallery` | Desktop showcase of 65 component pages, plus 4 chart pages and<br/>the 2 example apps for the website, `snapshot` (PNG renderer),<br/>and `site_data` (site JSON). |
| `examples/gallery-web` | `gallery-web` | WebAssembly entrypoint (`wasm32-unknown-unknown`) driving<br/>all website component previews in a single `iced::daemon`. |
| `conformance/` | `conformance` (`conformance/iced`)<br/>+ Vite + React app | Runs the same scenarios against `shadcn-iced` (headless)<br/>and the latest `shadcn/ui` (Playwright), and compares them. |
| `site/` | TanStack Start + Solid + Vite | Documentation website (`https://shadcn-iced.r4ai.dev`)<br/>consuming `site_data` JSON and the `gallery-web` WASM bundle. |
| `scripts/` | Developer Scripts | Maintenance utilities (`check-copy.sh`, `gen_themes.py`,<br/>`gen_icons.py`, `gen_fonts.py`, `grid.py`, `montage.py`). |

---

## Theme & Design Token Pipeline

`iced` style closures receive `&iced::Theme`. `shadcn_iced_core::theme` bridges full `shadcn/ui` tokens onto `iced::Theme::Custom` without requiring a custom theme type in user applications.

```mermaid
flowchart LR
    Builder["theme::Config (base, accent, radius, menu)"]
    Build["Config::build()"]
    Registry["Global REGISTRY (RwLock HashMap)"]
    IcedTheme["iced::Theme::Custom (+ derived Extended palette)"]
    TokensFn["tokens(&theme)"]
    Tokens["Tokens (Palette, Radius, Menu, Mode)"]

    Builder --> Build
    Build -- "registers (name, tokens)" --> Registry
    Build --> IcedTheme
    IcedTheme --> TokensFn
    Registry -. "on cache miss" .-> TokensFn
    TokensFn --> Tokens
```

### How `tokens(&theme)` Resolves Tokens

1. **Theme Builder (`theme::Config`)**
   - Created via `theme::light()`, `theme::dark()`, `theme::zinc(mode)`, or `theme::Config::new(base, mode)`
   - `base`: `BaseColor` (`Neutral`, `Stone`, `Zinc`, `Mauve`, `Olive`, `Mist`, `Taupe`)
   - `accent`: `Accent` (`Amber`, `Blue`, `Cyan`, `Emerald`, `Fuchsia`, `Green`, `Indigo`, `Lime`, `Orange`, `Pink`, `Purple`, `Red`, `Rose`, `Sky`, `Teal`, `Violet`, `Yellow`)
   - `chart_color`: `ThemeColor` (`Base(BaseColor)` or `Accent(Accent)`)
   - `radius` / `radius_preset`: `Radius` (`None`, `Small`, `Medium`, `Large`, `Default`) or `f32` pixels
   - `menu_color`: `MenuColor` (`Default`, `Inverted`, `DefaultTranslucent`, `InvertedTranslucent`)
   - `menu_accent`: `MenuAccent` (`Subtle`, `Bold`)
   - `palette(|p| ...)`: Direct mutation closure over `Palette`
2. **Registration & `Extended` Palette Synthesis**
   - `Config::build()` hashes the token configuration into a unique theme name
   - Inserts `(name, tokens)` into the global `REGISTRY` (`RwLock<HashMap<String, Tokens>>`)
   - Builds `Theme::custom_with_fn(...)` with a synthesized `iced::theme::palette::Extended` so built-in `iced` widgets match
3. **Thread-Local Fast Path (`LAST`)**
   - `tokens(&theme)` checks a thread-local `(Arc<Custom>, Tokens)` cache using `Arc::ptr_eq`
   - Repeated lookups within a frame resolve in $O(1)$ without locking
4. **Built-in Theme Fallback (`derive`)**
   - Standard `iced::Theme` variants (e.g. `Theme::Dracula`) derive `Tokens` from `theme.extended_palette()`
5. **Opacity Fading (`theme::faded`)**
   - Quantizes opacity into 24 steps and caches faded `Theme` instances per thread for animated overlays

### Palette Conversion

- Presets in `crates/shadcn_iced_core/src/theme/presets.rs` are generated by `scripts/gen_themes.py`
- Source values come from `shadcn/ui`'s official OKLCH registry (`apps/v4/registry/themes.ts`)
- OKLCH colors are converted through Oklab and linear sRGB into `iced::Color` at generation time

---

## Component Anatomy & State Management

Every component in `crates/shadcn_iced/src/components/<name>.rs` exposes a builder function that converts into `iced::Element<'a, Message>`:

```rust
pub fn button<'a, Message: Clone + 'a>(label: impl Into<String>) -> Button<'a, Message>
```

### Where Widget State Lives

| State Category | Stored In | Examples |
| :--- | :--- | :--- |
| **Visual & Transient** | Widget tree (`tree.state`) | Hover/press states, focus rings, animations, and open/close state of<br/>`DropdownMenu`, `ContextMenu`, `Menubar`, `Popover`, `HoverCard`,<br/>`Tooltip`, `Select`, `Combobox`, `Collapsible`, and `Accordion`. |
| **Controlled Values** | Caller model (`&str`, `bool`, `Option<T>`) | `input`, `textarea`, `checkbox`, `switch`, `slider`,<br/>`radio_group`, `tabs`, `calendar`, `date_picker`, `pagination`. |
| **Shared Collections** | Caller model (`Vec<Toast>`, `State`) | `sonner::Toast`, `toast::Toast`, `questionnaire::State`,<br/>and `sidebar` open state. |

### Self-Contained Portability Contract

- Component files under `crates/shadcn_iced/src/components/` never import from `crate::*`
- Shared utilities come from `shadcn_iced_core` paths (e.g. `use shadcn_iced_core::primitive::Pressable;`)
- Sibling components are imported via `use super::<sibling>::*;`
- `scripts/check-copy.sh` verifies this in CI by running `cargo shadcn-iced add --all` in an isolated crate

---

## Headless Primitives (`shadcn_iced_core::primitive`)

`shadcn_iced_core::primitive` implements **17 headless primitives**: 16 widgets on top of `iced::advanced::Widget` and `iced::advanced::Overlay`, plus `plot`, the plain geometry (functions and types) behind charts.

```mermaid
flowchart TD
    Builder["Component Builder (dropdown_menu, select, chart, ...)"]
    Primitive["Headless Primitive (shadcn_iced_core::primitive)"]
    IcedWidget["iced::advanced::Widget & Overlay"]

    Builder --> Primitive
    Primitive --> IcedWidget
```

| Primitive | Responsibility | Used By |
| :--- | :--- | :--- |
| `anchored` | Floating content next to an anchor: positioning on a side with an alignment,<br/>flipping and shifting to stay inside the window, opening (controlled, click,<br/>hover, right click, combobox), dismissal (`Escape`, outside click), and menubar groups. | `ContextMenu`, `DropdownMenu`, `HoverCard`,<br/>`Menubar`, `NavigationMenu`, `Popover`,<br/>`Tooltip`, `Select`, `Combobox`, `DatePicker` |
| `animated_width` | Smoothly interpolates container width between two target lengths. | `Sidebar` |
| `breakpoint` | Responsive variants (`SM = 640px`, `MD = 768px`) for the content of overlays,<br/>laid out with the window size (`measure`, `either`). | `Dialog`, `AlertDialog` |
| `disclosure` | Animated height reveal/collapse container that clips content<br/>during transitions and hides it when closed. | `Accordion`, `Collapsible`, `Trace` |
| `focus` | Tracks direct (`Focus`) or descendant (`FocusWithin`) focus and draws<br/>the focus ring (`Ring`, outside the bounds like CSS `ring-[3px]`). | `Input`, `Textarea`, `InputGroup`, `Checkbox`,<br/>`RadioGroup`, `Slider`, `Tabs`, `NativeSelect`,<br/>`Combobox`, `Card`, `Avatar` |
| `labelled` | HTML `<label>` behavior that focuses and activates<br/>the first interactive child control when clicked. | `Field`, `Label`, `Checkbox`,<br/>`RadioGroup`, `Switch` |
| `layer` | Content on top of the whole window (centered, against an edge, or in a corner),<br/>with an optional backdrop (`bg-black/10` by default) that blocks the rest of the interface. | `AlertDialog`, `Dialog`, `Sheet`,<br/>`Drawer`, `Sonner`, `Toast` |
| `listbox` | Vertical list with a highlighted item that follows the mouse and<br/>the keys (`ArrowUp`, `ArrowDown`, `Home`, `End`); `Enter` or a click selects it. | `Select`, `Combobox`, `Command`, menus |
| `plot` | Chart geometry (not a widget): category and linear scales, nice ticks,<br/>stacked baselines, natural curves, pie arcs, and pointer hit-testing. | `Chart` (`Area`, `Bar`, `Line`, `Pie`) |
| `pressable` | Clickable surface exposing hover, press, and focus status without button chrome;<br/>reacts to hover without `on_press` and can take the keyboard focus. | `Button`, `Toggle`, `Checkbox`, `Switch`,<br/>`Accordion`, `Tabs`, `Select`, `Calendar`,<br/>`Pagination`, `Sidebar`, `Badge`, `Breadcrumb`,<br/>`Item`, `Trace`, and others |
| `roving` | Roving focus: arrow keys, `Home`, and `End` move the focus<br/>between the items of a group, which is a single `Tab` stop. | `RadioGroup`, `ToggleGroup`, `Tabs` |
| `split` | Two panes separated by a divider that can be dragged or moved with the keys,<br/>self-managed unless controlled with `ratio` and `on_resize`. | `Resizable` |
| `toaster` | Animated corner stack manager handling hover-to-expand,<br/>dismiss gestures, auto-dismiss timers, and stack transforms. | `Sonner`, `Toast` |
| `track` | Horizontal/vertical sliding viewport with drag gestures<br/>and smooth index snapping. | `Carousel` |
| `tracked` | Character-by-character text layout implementing CSS<br/>`letter-spacing` while supporting widget text lookup. | Shortcuts in `Command` and menus |
| `underline` | Underlines text while hovered (`hover:underline`). | `Button` (`Variant::Link`) |
| `wheel` | Browser-like wheel scrolling: smooth, with `Shift + Wheel` scrolling horizontally.<br/>Wraps a whole application view. | Application views (`gallery`, `gallery-web`) |

---

## Fonts, Icons & Cargo Features

`shadcn_iced_core` embeds fonts and icons into the binary for consistent rendering across desktop and WebAssembly.

| Feature | Default | Bundled Asset | Size | Purpose |
| :--- | :---: | :--- | :---: | :--- |
| `lucide` | Yes | `fonts/lucide.ttf` | ~900 KB | 2,179 [Lucide](https://lucide.dev) icons exposed in `shadcn_iced::icon::lucide`<br/>(also `shadcn_iced::prelude::lucide`). |
| `geist` | Yes | `Geist-*.ttf`,<br/>`GeistMono-*.ttf` | ~810 KB | Default sans-serif (`Geist`: `Regular`, `Medium`, `SemiBold`, `Bold`)<br/>and monospace (`Geist Mono`: `Regular`, `Medium`) typefaces. |
| `inter` | Yes | `Inter-*.ttf` | ~710 KB | Alternative sans-serif typeface (`Inter`), selectable<br/>at runtime via `font::set_sans(font::INTER)`. |
| `symbols` | Yes | `ShadcnIcedSymbols-Regular.ttf` | ~55 KB | Subset of Noto Sans Symbols, Symbols 2, and Math covering<br/>keyboard shortcut glyphs (`⌘ ⌥ ⌃ ⇧ ⏎ ⌫ ⎋ ⇥ ␣`) and arrows. |

- Call `.shadcn_iced()` on `iced::application` or `iced::daemon` to register all enabled fonts and set the default font
- Alternatively, register fonts manually via `shadcn_iced::font::all()` and `shadcn_iced::font::sans()`

---

## Animation System

`crates/shadcn_iced_core/src/animation.rs` ports the `tw-animate-css` utilities used by `shadcn/ui`. Animated widgets keep a `Transition` in their state and redraw until it settles, timed with `iced::time::Instant`:

- **Durations**
  - `FAST` (`150ms`), `NORMAL` (`200ms`), `SLOW` (`300ms`), and `SLOWER` (`500ms`)
- **Easings**
  - `linear`, `ease`, `ease_in`, `ease_out`, `css_ease_out`, `ease_in_out`, and `cubic_bezier(x1, y1, x2, y2, t)` for custom curves
- **Effects**
  - Transforms (`zoom-in-95`, `slide-in-from-top-2`, sheets sliding in) through `iced::Transformation`
  - Opacity (`fade-in-0`) by drawing a subtree with a faded copy of the theme (`Fade`, `theme::faded`)
- **`shadcn_iced::animation::set_enabled(false)`**
  - Snaps all animations immediately to their target value for deterministic snapshots or reduced motion

---

## Multi-Window WebAssembly Architecture (`gallery-web`)

`examples/gallery-web` uses `iced::daemon` with `window::settings::PlatformSpecific` to render all `<canvas>` previews on a documentation page from a single WASM instance.

```mermaid
sequenceDiagram
    participant Page as Site Doc Page
    participant WASM as gallery-web Daemon
    participant Canvas as iced Canvas Window

    Page->>WASM: mount(target, page, example)
    WASM->>Canvas: window::open(target)
    Canvas-->>WASM: window::Id
    WASM->>WASM: gallery::view_example(gallery, page, example)
    WASM-->>Page: shadcnIcedPreviewResized(target, height)
    WASM-->>Page: shadcnIcedPreviewWheel(captured)
    Page->>WASM: setTheme(dark, base, accent, radius)
    WASM->>WASM: Update theme across all windows
    Page->>WASM: themeCode(dark, base, accent, radius)
    WASM-->>Page: Rust code of the theme
    Page->>WASM: unmount(target)
    WASM->>Canvas: window::close
```

---

## Known Web (`shadcn/ui`) vs. Native (`iced`) Differences

- **Font Hinting & Anti-Aliasing**
  - Browser text rendering differs slightly in glyph weight and kerning from `cosmic-text` + `tiny-skia` / `wgpu`
- **Backdrop Blur**
  - Translucent menus (`MenuColor::DefaultTranslucent` / `InvertedTranslucent`) use the same translucent surface (`bg-popover/70`) without the CSS `backdrop-filter: blur(...)`, which iced cannot do
- **CSS Box Shadows**
  - Multi-layer CSS shadows (`shadow-xs`, `shadow-md`, `shadow-lg`) map to single-layer `iced::Shadow`
- **DOM Portals vs. `iced::advanced::Overlay`**
  - Floating menus and modals render via `iced::advanced::Overlay` within window bounds rather than `document.body` portals
