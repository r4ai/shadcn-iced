# Development Guide

This guide covers building, testing, inspecting, and contributing to `shadcn-iced`.

---

## Prerequisites

| Tool | Requirement | Used For |
| :--- | :--- | :--- |
| **Rust** | `1.88+` (`2024` edition) | Building workspace crates, CLI,<br/>gallery, and conformance runner |
| **Node.js & `pnpm`** | Node `22+` (site), `24+` (conformance), `pnpm 10+` | Running the docs site (`site/`)<br/>and conformance suite (`conformance/`) |
| **`wasm32-unknown-unknown` & `wasm-bindgen-cli`** | `wasm-bindgen-cli` matching the<br/>`wasm-bindgen` version in `Cargo.lock` | Compiling interactive WebAssembly<br/>previews (`examples/gallery-web`) |
| **Python 3 / `uv`** | Optional | Running theme, icon, font, and<br/>snapshot scripts in `scripts/` |

---

## Everyday Commands

Run these from the repository root before opening a pull request:

```sh
# Format check
cargo fmt --all --check

# Lint all workspace crates and targets
cargo clippy --workspace --all-targets -- -D warnings

# Run unit tests and component doc-tests
cargo test --workspace

# Verify copy-and-own isolation across all 66 components
bash scripts/check-copy.sh
```

---

## Running the Desktop Gallery

The `gallery` package (`examples/gallery`) showcases all **65** component pages. The **4** chart pages (`charts-area`, `charts-bar`, `charts-line`, `charts-pie`) are only shown on the website, through `gallery-web`.

```sh
# The minimal app of the README quick start
cargo run -p hello

# Launch the desktop gallery (opens on "button" by default)
cargo run -p gallery

# Open directly to a specific page ID
GALLERY_PAGE=calendar cargo run -p gallery
GALLERY_PAGE=sonner cargo run -p gallery

# Start with Inter instead of Geist as the body font
GALLERY_FONT=inter cargo run -p gallery
```

---

## Running the Example Apps

Two mock apps (`examples/dashboard`, `examples/settings`) show the components working together. Each is a crate of its own: a library with the state, `update` and `view` of the app, and a `main.rs` that runs it. The gallery maps their views (`examples/gallery/src/pages/apps.rs`) for the `/examples` pages of the website, and the `snapshot` binary renders them by id.

```sh
# The dashboard-01 block of shadcn/ui: sidebar, metrics, chart and data table
cargo run -p dashboard

# An app's settings: profile, account, appearance, notifications and display
cargo run -p settings

# Render an app headlessly, like any gallery page
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- settings:dark --click Notifications --size 1280x1000
```

---

## Headless Snapshot Tool (`snapshot`)

Render any gallery page to PNG files without opening a window using `examples/gallery/src/bin/snapshot.rs`.

```sh
# Render light and dark snapshots of specific pages to target/snapshots/
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- button card:dark dialog:light

# Render every component page (not the chart pages or the apps) in both light and dark modes
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot
```

### Interaction & Theme Flags

- **Actions before `--live`**
  - Dispatch messages through `Gallery::update` and rebuild the view after each action
- **Actions after `--live`**
  - Run directly on the final widget tree right before capturing the snapshot
  - Preserves internal widget state (focus rings, hover tooltips, and self-managed overlays)

```sh
# Open a dropdown menu and hover a submenu item before capturing
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- \
  dropdown-menu:dark --live --click "Open" --hover 450,310

# Customize the theme and the chart colors
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- card:light --accent blue --chart rose

# Capture an in-progress animation frame
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot --   dropdown-menu:light --animations --live --click "Open" --wait 70
```

Unknown `--base` and `--accent` names are ignored, and unknown `--radius` and `--menu-color` values fall back to the default.

| Flag | Description |
| :--- | :--- |
| `<page-id>[:light\|:dark]` | Page ID (e.g. `button`, `dialog:light`, `settings:dark`).<br/>Omit `:mode` to render both light and dark. |
| `--size <WxH>` | Viewport size in logical pixels (default: `1100x820`). |
| `--suffix <STR>` | Appends `<STR>` to the output filename<br/>(`target/snapshots/<id><suffix>-<mode>-tiny-skia.png`). |
| `--base <NAME>` | Base palette: `neutral`, `stone`, `zinc`,<br/>`mauve`, `olive`, `mist`, `taupe`. |
| `--accent <NAME>` | Accent color: `amber`, `blue`, `cyan`, `emerald`, `fuchsia`,<br/>`green`, `indigo`, `lime`, `orange`, `pink`, `purple`,<br/>`red`, `rose`, `sky`, `teal`, `violet`, `yellow`. |
| `--chart <NAME>` | Chart palette preset (any base or accent color name). |
| `--radius <SIZE>` | Corner radius preset: `none`, `small`, `medium`, `large`, `default`. |
| `--menu-color <MODE>` | Menu surface style: `default`, `inverted`,<br/>`translucent`, `inverted-translucent`. |
| `--menu-accent <MODE>` | Menu highlight style: `subtle` (default) or `bold`. |
| `--heading <FONT>` | Heading font family: `geist`, `inter`, or `inherit`. |
| `--customize` | Opens the theme customizer popover in the gallery header. |
| `--animations` | Enables animations (disabled by default for determinism). |
| `--live` | Runs subsequent action flags on the live final widget tree. |
| `--click "<LABEL>"` | Clicks the visible text widget matching `<LABEL>`. |
| `--at <X,Y>` | Moves the cursor to `(X, Y)` and clicks. |
| `--hover <X,Y>` | Moves the cursor to `(X, Y)` without clicking. |
| `--type "<TEXT>"` | Types `<TEXT>` into the currently focused widget. |
| `--key <KEY>` | Taps a key (`escape`, `enter`, `tab`,<br/>`down`, `up`, `space`, or a character). |
| `--wait <MS>` | Renders a frame, sleeps `<MS>` ms, and requests a redraw. |

---

## Working with the CLI (`cargo-shadcn-iced`)

Test CLI commands directly via `cargo run -p cargo-shadcn-iced --`:

```sh
# List all 66 available components and their descriptions
cargo run -p cargo-shadcn-iced -- list

# Inspect a component's embedded source code
cargo run -p cargo-shadcn-iced -- view button

# Copy components (plus transitive siblings) into a target directory
cargo run -p cargo-shadcn-iced -- add dialog dropdown-menu --dir /tmp/test-ui

# Compare local files against the embedded upstream templates
cargo run -p cargo-shadcn-iced -- diff --dir /tmp/test-ui
```

### CLI Options Reference

| Command / Flag | Description |
| :--- | :--- |
| `list` | Prints all available components and their one-line summaries. |
| `add <NAME>...` | Copies `<NAME>` and its transitive sibling dependencies<br/>into `--dir` and updates `mod.rs` (`kebab-case` or `snake_case`). |
| `add --all` | Copies all 66 components into `--dir`. |
| `diff [<NAME>...]` | Prints `same`, `modified` (with the number of differing lines),<br/>or `missing` for each component in `--dir` against the embedded templates. |
| `view <NAME>` | Prints the embedded Rust source of `<NAME>` to stdout. |
| `--dir <PATH>` | Target module directory (default: `src/ui`). |
| `--overwrite` | Overwrites existing `<name>.rs` files when running `add`. |

---

## Adding or Modifying a Component

1. **Implement `crates/shadcn_iced/src/components/<name>.rs`**
   - Import only from `iced`, `shadcn_iced_core::*`, and sibling components (`use super::<sibling>::*;`)
   - Never use `crate::*` (breaks `cargo shadcn-iced add` in user projects)
   - Add a `//!` module doc summary and a `//! # Example` (` ```no_run `) block
2. **Export the component**
   - Add `pub mod <name>;` to `crates/shadcn_iced/src/components/mod.rs`
   - Re-export its builder functions in `pub mod prelude` of the same `components/mod.rs`<br/>(`crates/shadcn_iced/src/lib.rs` re-exports everything with globs)
3. **Add a Gallery Page**
   - Define a `Page` in `examples/gallery/src/pages/` and register it in `PAGES`
   - The `site_data` binary (`pnpm data`) uses `#[track_caller]` on `gallery::example(...)` to extract preview source code
4. **Add Conformance Scenarios (if applicable)**
   - Add the scenarios in `conformance/scenarios/<name>.json`
   - Add the iced fixtures in `conformance/iced/src/fixtures/<name>.rs` and register them in `mod.rs`
   - Add the matching React fixtures in `conformance/src/fixtures/<name>.tsx`
5. **Verify Copy Isolation & Tests**
   - Run `bash scripts/check-copy.sh` and `cargo test --workspace`

---

## Conformance Testing (`conformance/`)

See [`conformance/README.md`](../conformance/README.md) for full details.

```sh
cd conformance
pnpm install
pnpm exec playwright install chromium

# Full QA pipeline: renders iced snapshots, compares them with shadcn/ui, and writes out/report/index.html
pnpm qa

# Render only specific iced scenarios (substring filter on component/scenario)
pnpm iced button dialog

# Run the Playwright comparison for matching scenarios
pnpm test -g "button|dialog"

# Record the scores of the last run in baseline.json (commit it with the fix)
pnpm baseline button/
```

---

## Documentation Website (`site/`)

```sh
# One-time setup for the WebAssembly previews
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <version-in-Cargo.lock> --locked

cd site
pnpm install

# 1. Extract component docstrings and gallery sources to site/src/data/components.json
pnpm data

# 2. Compile examples/gallery-web to site/public/wasm/ (cargo, wasm-bindgen, wasm-opt)
pnpm wasm

# 3. Start the Vite dev server (TanStack Start + Solid); re-run 1 and 2 after changing examples or components
pnpm dev

# Type-check, lint, and format checks
pnpm check
pnpm lint
pnpm fmt:check

# Build static production site into site/dist/client/
pnpm build

# Run Playwright E2E tests against the built site (run `pnpm build` first)
pnpm test:e2e
```

---

## Asset & Code Generation Scripts (`scripts/`)

| Script | Purpose |
| :--- | :--- |
| `bash scripts/check-copy.sh` | Verifies `cargo shadcn-iced add --all` compiles<br/>cleanly with `-D warnings` in an isolated crate. |
| `python3 scripts/gen_themes.py <shadcn-ui-dir>` | Converts OKLCH tokens from `apps/v4/registry/themes.ts`<br/>and writes `crates/shadcn_iced_core/src/theme/presets.rs`. |
| `python3 scripts/gen_icons.py <lucide-dir>` | Reads `font/codepoints.json` from `lucide-static` and<br/>writes `crates/shadcn_iced_core/src/icon/lucide.rs`. |
| `uv run --with fonttools scripts/gen_fonts.py` | Builds `ShadcnIcedSymbols-Regular.ttf` and<br/>`conformance/fonts/NotoEmoji-Regular.ttf`. |
| `uv run --with pillow scripts/grid.py <out-prefix> <pngs...>` | Tiles PNG snapshots into `3×3` contact sheets<br/>(`<out-prefix><n>.png`) for visual inspection. |
| `uv run --with pillow scripts/montage.py <out.png> <pngs...>` | Combines up to 4 snapshots into one `2×2`<br/>contact sheet at half resolution. |

---

## CI/CD Workflows (`.github/workflows/`)

- **`ci.yml`** (push to `main`, pull requests, manual dispatch)
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `./scripts/check-copy.sh`
  - Renders every component page with `snapshot` (`tiny-skia`) and uploads them as the `snapshots` artifact
- **`site.yml`** (push to `main`, pull requests touching `site/`, `examples/`, or `crates/`, manual dispatch)
  - **Build Job**: `pnpm lint`, `pnpm fmt:check`, `pnpm data`, `pnpm wasm`, `pnpm build`, `pnpm check`, and `pnpm test:e2e`
  - **Deploy Job** (`main` only): deploys `site/dist/client` to Cloudflare Workers static assets (`site/wrangler.jsonc`)<br/>with the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets of the `production` environment
- **`conformance.yml`** (push to `main`, pull requests touching `crates/`, `conformance/`, or `Cargo.lock`, weekly on Monday, manual dispatch)
  - Runs `pnpm typecheck`, `pnpm iced`, `pnpm test`, and `pnpm report` against `baseline.json`, and uploads the report with the `baseline.json` of the run, ready to be committed
  - The weekly run (and a manual run with `update`) first updates shadcn/ui to its latest release with `pnpm shadcn:update`
