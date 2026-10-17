# Development Guide

This guide covers everything needed to build, test, inspect, and contribute to `shadcn-iced`.

---

## Prerequisites

| Tool | Requirement | Used For |
| :--- | :--- | :--- |
| **Rust** | `1.88+` (`2024` edition) | Building all workspace crates, CLI, gallery, and conformance runner |
| **Node.js & `pnpm`** | Node `20+`, `pnpm 10+` | Running the documentation site (`site/`) and conformance suite (`conformance/`) |
| **`wasm-pack`** | Latest (`wasm32-unknown-unknown` target) | Compiling interactive WebAssembly previews (`examples/gallery-web`) |
| **Python 3 / `uv`** | Optional | Running theme/icon/font generation and snapshot montage scripts in `scripts/` |

---

## Everyday Commands

Run these from the repository root before submitting a pull request:

```sh
# Format check
cargo fmt --all --check

# Lint all workspace crates and targets
cargo clippy --workspace --all-targets -- -D warnings

# Run unit tests and component doc-tests
cargo test --workspace

# Verify copy-and-own isolation (copies all 66 components into a fresh temp crate and checks compilation)
bash scripts/check-copy.sh
```

---

## Running the Desktop Gallery

The `gallery` package (`examples/gallery`) showcases all **65** component pages and **4** chart pages with an interactive theme customizer.

```sh
# Launch the desktop gallery (opens on the "button" page by default)
cargo run -p gallery

# Open directly to a specific page ID
GALLERY_PAGE=calendar cargo run -p gallery
GALLERY_PAGE=charts-area cargo run -p gallery

# Start with Inter instead of Geist as the body font
GALLERY_FONT=inter cargo run -p gallery
```

---

## Headless Snapshot Tool (`snapshot`)

You can render any gallery page to PNG files without opening a window using the `snapshot` binary (`examples/gallery/src/bin/snapshot.rs`) powered by `iced_test::Simulator` and `tiny-skia`.

```sh
# Render both light and dark snapshots of specific pages to target/snapshots/
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- button card:dark dialog:light

# Render every gallery page in both light and dark modes
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot
```

### Interaction & Theme Flags

Actions placed **before** `--live` dispatch messages through `Gallery::update` and rebuild the view. Actions placed **after** `--live` run directly on the final widget tree right before taking the screenshot (preserving internal widget state such as focus rings, hover tooltips, and self-managed overlays like `DropdownMenu` or `Select`).

```sh
# Open a dropdown menu and hover a submenu item before capturing
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot -- \
  dropdown-menu:dark --live --click "Open" --hover 450,310
```

| Flag | Description |
| :--- | :--- |
| `<page-id>[:light\|:dark]` | Page ID (e.g. `button`, `dialog:light`, `charts-bar:dark`). Omit `:mode` to render both. |
| `--size <WxH>` | Viewport size in logical pixels (default: `1100x820`). |
| `--suffix <STR>` | Appends `<STR>` to the output filename (`target/snapshots/<id><suffix>-<mode>-tiny-skia.png`). |
| `--base <NAME>` | Base palette: `neutral`, `stone`, `zinc`, `mauve`, `olive`, `mist`, `taupe`. |
| `--accent <NAME>` | Accent color: `amber`, `blue`, `cyan`, `emerald`, `fuchsia`, `green`, `indigo`, `lime`, `orange`, `pink`, `purple`, `red`, `rose`, `sky`, `teal`, `violet`, `yellow`. |
| `--chart <NAME>` | Chart palette preset (any base or accent color name). |
| `--radius <SIZE>` | Corner radius preset: `none`, `small`, `medium`, `large`, `default`. |
| `--menu-color <MODE>` | Menu surface style: `default`, `inverted`, `translucent`, `inverted-translucent`. |
| `--menu-accent <MODE>` | Menu highlight style: `subtle` (default) or `bold`. |
| `--heading <FONT>` | Heading font family: `geist`, `inter`, or `inherit`. |
| `--customize` | Opens the theme customizer popover in the gallery header. |
| `--animations` | Enables animations (disabled by default for deterministic screenshots). |
| `--live` | Switches subsequent action flags to operate on the live final widget tree. |
| `--click "<LABEL>"` | Clicks the visible text widget matching `<LABEL>`. |
| `--at <X,Y>` | Moves the cursor to `(X, Y)` and clicks. |
| `--hover <X,Y>` | Moves the cursor to `(X, Y)` without clicking. |
| `--type "<TEXT>"` | Types `<TEXT>` into the currently focused widget. |
| `--key <KEY>` | Taps a key (`escape`, `enter`, `tab`, `down`, `up`, `space`, or a character). |
| `--wait <MS>` | Renders a frame, sleeps `<MS>` milliseconds, and requests a redraw. |

---

## Working with the CLI (`cargo-shadcn-iced`)

Test CLI commands directly via `cargo run -p cargo-shadcn-iced --`:

```sh
# List all 66 available components and their descriptions
cargo run -p cargo-shadcn-iced -- list

# Inspect a component's embedded source code
cargo run -p cargo-shadcn-iced -- view button

# Copy components (plus any transitive sibling dependencies) into a target directory
cargo run -p cargo-shadcn-iced -- add dialog dropdown-menu --dir /tmp/test-ui

# Compare local files against the embedded upstream templates
cargo run -p cargo-shadcn-iced -- diff --dir /tmp/test-ui
```

### CLI Options Reference

| Command / Flag | Description |
| :--- | :--- |
| `list` | Prints all available components and their one-line doc summaries. |
| `add <NAME>...` | Copies `<NAME>` and its transitive sibling dependencies into `--dir` and updates `mod.rs`. Accepts both `kebab-case` and `snake_case`. |
| `add --all` | Copies all 66 components into `--dir`. |
| `diff [<NAME>...]` | Shows a unified diff between local files in `--dir` and the embedded templates. |
| `view <NAME>` | Prints the embedded Rust source of `<NAME>` to stdout. |
| `--dir <PATH>` | Target module directory (default: `src/ui`). |
| `--overwrite` | Overwrites existing `<name>.rs` files when running `add`. |

---

## Adding or Modifying a Component

When creating or updating a component, follow these conventions to keep the library, CLI, gallery, conformance suite, and docs site in sync:

1. **Implement the component in `crates/shadcn_iced/src/components/<name>.rs`**:
   - **Portability rule**: Import only from `iced`, `shadcn_iced_core::*`, and sibling components (`use super::<sibling>::*;`). **Never** use `crate::*`, or `cargo shadcn-iced add` will fail in user projects.
   - Include a module-level `//!` doc comment at the top of the file:
     - First paragraph: concise summary (used by `cargo shadcn-iced list` and the documentation site).
     - `//! # Example` section with a ` ```no_run ` Rust code block (`cargo shadcn-iced add` automatically rewrites `no_run` to `ignore` when copying into user crates).
2. **Export the component**:
   - Add `pub mod <name>;` to `crates/shadcn_iced/src/components/mod.rs`.
   - Re-export primary entrypoints in `crates/shadcn_iced/src/lib.rs` (and `prelude` if appropriate).
3. **Add a Gallery Page**:
   - Add a `Page` definition under `examples/gallery/src/pages/` and register it in `PAGES` in `examples/gallery/src/pages/mod.rs`.
   - Note: `extract_docs` uses `#[track_caller]` on `gallery::example(title, content)` to extract the exact Rust source code for every preview on the documentation website.
4. **Add Conformance Fixtures (if applicable)**:
   - Add the iced fixture in `conformance/iced/src/fixtures/` and the matching React fixture in `conformance/react/app/` (see [`conformance/README.md`](../conformance/README.md)).
5. **Verify Copy Isolation & Tests**:
   - Run `bash scripts/check-copy.sh` and `cargo test --workspace`.

---

## Conformance Testing (`conformance/`)

The `conformance/` directory runs pixel-diff comparisons between `shadcn-iced` and an official Next.js + React `shadcn/ui` application at `2x` device scale factor. See [`conformance/README.md`](../conformance/README.md) for architectural details.

```sh
cd conformance
pnpm install

# Full QA pipeline: renders iced snapshots, runs Playwright pixel diffs, and builds report.html
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

The documentation website (`site/`) is built with Astro, Starlight, and Tailwind CSS v4. It embeds live WebAssembly previews compiled from `examples/gallery-web`.

```sh
cd site
pnpm install

# 1. Extract component docstrings and gallery example sources to site/src/data/components.json
pnpm data

# 2. Compile examples/gallery-web to site/public/wasm/ via wasm-pack
pnpm wasm

# 3. Start the Astro dev server (automatically runs `pnpm data` first)
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

| Script | Usage | Purpose |
| :--- | :--- | :--- |
| `scripts/check-copy.sh` | `bash scripts/check-copy.sh` | Verifies `cargo shadcn-iced add --all` compiles cleanly with `-D warnings` in an isolated crate. |
| `scripts/gen_themes.py` | `python3 scripts/gen_themes.py <path-to-shadcn-ui>` | Reads `apps/v4/registry/themes.ts` from a `shadcn-ui` checkout, converts OKLCH tokens to sRGB, and writes `crates/shadcn_iced_core/src/theme/presets.rs`. |
| `scripts/gen_icons.py` | `python3 scripts/gen_icons.py <path-to-lucide-static>` | Reads `font/codepoints.json` from `lucide-static` and generates `crates/shadcn_iced_core/src/icon/lucide.rs`. |
| `scripts/gen_fonts.py` | `uv run --with fonttools scripts/gen_fonts.py` | Builds `ShadcnIcedSymbols-Regular.ttf` (subsetting keyboard/math glyphs from Noto Sans Symbols) and `conformance/fonts/NotoEmoji-Regular.ttf`. |
| `scripts/grid.py` | `uv run --with pillow scripts/grid.py <pngs...>` | Tiles PNG snapshots into `3×3` contact sheets (`/tmp/grid-*.png`) for visual inspection. |
| `scripts/montage.py` | `uv run --with pillow scripts/montage.py` | Combines light/dark snapshots in `target/snapshots/` into `2×2` contact sheets under `target/montage/`. |

---

## CI/CD Workflows (`.github/workflows/`)

- **`ci.yml`** (runs on every push to `main` and pull request):
  - **Rust Job**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `bash scripts/check-copy.sh`.
  - **Site Job**: `pnpm fmt:check`, `pnpm lint`, `pnpm data`, `pnpm check`, builds a stub WASM bundle for fast verification, and runs `pnpm build`.
- **`deploy-docs.yml`** (runs on push to `main` or manual dispatch):
  - Installs Rust + `wasm-pack`, runs `pnpm data`, `pnpm wasm`, and `pnpm build` inside `site/`, and deploys `site/dist` to Cloudflare Pages (`shadcn-iced`).
