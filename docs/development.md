# Development Guide

This guide covers building, testing, inspecting, and contributing to `shadcn-iced`.

---

## Prerequisites

| Tool | Requirement | Used For |
| :--- | :--- | :--- |
| **Rust** | `1.88+` (`2024` edition) | Building workspace crates, CLI,<br/>gallery, and conformance runner |
| **Node.js & `pnpm`** | Node `20+`, `pnpm 10+` | Running the docs site (`site/`)<br/>and conformance suite (`conformance/`) |
| **`wasm-pack`** | Latest (`wasm32-unknown-unknown`) | Compiling interactive WebAssembly<br/>previews (`examples/gallery-web`) |
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

The `gallery` package (`examples/gallery`) showcases all **65** component pages and **4** chart pages.

```sh
# Launch the desktop gallery (opens on "button" by default)
cargo run -p gallery

# Open directly to a specific page ID
GALLERY_PAGE=calendar cargo run -p gallery
GALLERY_PAGE=charts-area cargo run -p gallery

# Start with Inter instead of Geist as the body font
GALLERY_FONT=inter cargo run -p gallery
```

---

## Headless Snapshot Tool (`snapshot`)

Render any gallery page to PNG files without opening a window using `examples/gallery/src/bin/snapshot.rs`.

```sh
# Render light and dark snapshots of specific pages to target/snapshots/
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- button card:dark dialog:light

# Render every gallery page in both light and dark modes
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
```

| Flag | Description |
| :--- | :--- |
| `<page-id>[:light\|:dark]` | Page ID (e.g. `button`, `dialog:light`, `charts-bar:dark`).<br/>Omit `:mode` to render both light and dark. |
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
| `diff [<NAME>...]` | Shows a unified diff between local files in `--dir`<br/>and the embedded upstream templates. |
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
   - Re-export primary entrypoints in `crates/shadcn_iced/src/lib.rs` and `prelude`
3. **Add a Gallery Page**
   - Define a `Page` in `examples/gallery/src/pages/` and register it in `PAGES`
   - `extract_docs` uses `#[track_caller]` on `gallery::example(...)` to extract preview source code
4. **Add Conformance Fixtures (if applicable)**
   - Add the iced fixture in `conformance/iced/src/fixtures/`
   - Add the matching React fixture in `conformance/react/app/`
5. **Verify Copy Isolation & Tests**
   - Run `bash scripts/check-copy.sh` and `cargo test --workspace`

---

## Conformance Testing (`conformance/`)

See [`conformance/README.md`](../conformance/README.md) for full details.

```sh
cd conformance
pnpm install

# Full QA pipeline: renders iced snapshots, runs pixel diffs, and builds report.html
pnpm qa

# Render only specific iced fixtures (substring filter)
pnpm iced button dialog

# Run Playwright comparison for matching fixtures
pnpm test -g "button|dialog"

# Regenerate reference React PNG baselines via Playwright
pnpm baseline button
```

---

## Documentation Website (`site/`)

```sh
cd site
pnpm install

# 1. Extract component docstrings and gallery sources to site/src/data/components.json
pnpm data

# 2. Compile examples/gallery-web to site/public/wasm/ via wasm-pack
pnpm wasm

# 3. Start the Astro dev server (runs `pnpm data` automatically)
pnpm dev

# Type-check, lint, and format checks
pnpm check
pnpm lint
pnpm fmt:check

# Build static production site into site/dist/
pnpm build

# Run Playwright E2E tests against the built site
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
| `uv run --with pillow scripts/grid.py <pngs...>` | Tiles PNG snapshots into `3×3` contact sheets<br/>(`/tmp/grid-*.png`) for visual inspection. |
| `uv run --with pillow scripts/montage.py` | Combines light/dark snapshots in `target/snapshots/`<br/>into `2×2` contact sheets under `target/montage/`. |

---

## CI/CD Workflows (`.github/workflows/`)

- **`ci.yml`** (runs on push to `main` and pull requests)
  - **Rust Job**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `bash scripts/check-copy.sh`
  - **Site Job**: `pnpm fmt:check`, `pnpm lint`, `pnpm data`, `pnpm check`, and `pnpm build`
- **`deploy-docs.yml`** (runs on push to `main` or manual dispatch)
  - Runs `pnpm data`, `pnpm wasm`, and `pnpm build` in `site/`
  - Deploys `site/dist` to Cloudflare Pages (`shadcn-iced`)
