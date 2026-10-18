# Development Guide

This guide covers building, testing, inspecting, and contributing to `shadcn-iced`.

---

## Prerequisites

| Tool | Requirement | Used For |
| :--- | :--- | :--- |
| **Rust** | `1.88+` (`2024` edition), via `rustup` | Building workspace crates, CLI,<br/>gallery, and conformance runner |
| **[mise](https://mise.jdx.dev)** | `2026.10+` | Installing the pinned Node.js, `pnpm`,<br/>`uv`, and workflow linters of `mise.toml` |
| **`wasm-bindgen-cli`** | The version of `Cargo.lock` (`wasm32-unknown-unknown`) | Compiling interactive WebAssembly<br/>previews (`examples/gallery-web`) |

`mise install` installs the tools of [`mise.toml`](../mise.toml) (Node.js, `pnpm`, `uv`, `actionlint`, `zizmor`, `pinact`), checked against the checksums of `mise.lock`. Rust is not managed by mise: install it with `rustup`.

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

The `gallery` package (`examples/gallery`) showcases all **65** component pages. The **7** chart pages (`charts-area`, `charts-bar`, `charts-line`, `charts-pie`, `charts-radar`, `charts-radial`, `charts-tooltip`) are only shown on the website, through `gallery-web`, and can be rendered on their own by `snapshot` (e.g. `charts-radar:dark --size 440x4600`).

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
| `uv run --project scripts scripts/gen_themes.py <shadcn-ui-dir>` | Converts OKLCH tokens from `apps/v4/registry/themes.ts`<br/>and writes `crates/shadcn_iced_core/src/theme/presets.rs`. |
| `uv run --project scripts scripts/gen_icons.py <lucide-dir>` | Reads `font/codepoints.json` from `lucide-static` and<br/>writes `crates/shadcn_iced_core/src/icon/lucide.rs`. |
| `uv run --project scripts scripts/gen_fonts.py` | Builds `ShadcnIcedSymbols-Regular.ttf` and<br/>`conformance/fonts/NotoEmoji-Regular.ttf`. |
| `uv run --project scripts scripts/grid.py <out-prefix> <pngs...>` | Tiles PNG snapshots into `3×3` contact sheets<br/>(`<out-prefix><n>.png`) for visual inspection. |
| `uv run --project scripts scripts/montage.py <out.png> <pngs...>` | Combines up to 4 snapshots into one `2×2`<br/>contact sheet at half resolution. |
| `bash scripts/prepare-publish.sh` | Copies the components into `crates/cargo-shadcn-iced/components/`<br/>for `cargo publish` (see [Releasing](#releasing)). |

The Python dependencies are locked in `scripts/uv.lock` (`scripts/pyproject.toml`); uv only resolves releases at least 7 days old.

---

## CI/CD Workflows (`.github/workflows/`)

`ci.yml` (**CI**) runs on pull requests and on `main`. It looks at the files a pull request changes and calls the workflows of the parts they touch (every part on `main`), all in parallel. Its **Result** job is the single required status check of `main`.

| Workflow | Jobs | Runs when |
| :--- | :--- | :--- |
| `rust.yml` (**Rust**) | Format, Clippy, Test, Snapshots,<br/>Copy components, Package | Rust code, `Cargo.*` or `scripts/` change |
| `site.yml` (**Site**) | Lint, Data, Live previews, Build (+ e2e),<br/>Deploy (`main` only, `production` environment) | `site/`, `crates/` or `examples/` change |
| `conformance.yml` (**Conformance**) | Type check, Compare | `crates/` or `conformance/` change;<br/>weekly with the latest shadcn/ui |
| `lint.yml` (**Lint**) | Workflows (actionlint, zizmor, pinact),<br/>Python (`uv.lock`, compile) | `.github/` or `scripts/` change |
| `release.yml` (**Release**) | Release PR, Check, Publish, Binaries | Every push to `main` (see [Releasing](#releasing)) |

### Supply-chain policies

- **Actions** are pinned to the commit of their release (`pinact run -u -m 7`, checked by `pinact run --check --verify`), and every job declares the permissions it needs (`permissions: {}` at the top of each workflow).
- **Rust** is installed with the rustup of the runner (`.github/actions/setup-rust`), not a third-party action.
- **pnpm** (`site/` and `conformance/`, `pnpm-workspace.yaml`): `minimumReleaseAge` of 7 days, `trustPolicy: no-downgrade`, `blockExoticSubdeps`, and `strictDepBuilds` with an explicit `allowBuilds`. pnpm 12 also checks the lockfile against these policies on every install.
- **uv** (`scripts/`): `exclude-newer = "7 days"`.
- **Dependabot** waits 7 days (`cooldown`) before proposing an update; the tools of `mise.toml` are updated by hand (`mise lock`).

---

## Releasing

The crates `shadcn-iced` (imported as `shadcn_iced`), `shadcn_iced_core` and `cargo-shadcn-iced` share the workspace version and are released together by [release-plz](https://release-plz.dev) (`release-plz.toml`, `.github/workflows/release.yml`):

1. Every push to `main` opens or updates the **release pull request**: it bumps the version (following the conventional commits and `cargo-semver-checks`) and updates `CHANGELOG.md`.
2. Merging it **publishes** the crates to crates.io with [trusted publishing](https://crates.io/docs/trusted-publishing) (no token), tags `v<version>` and creates the GitHub release.
3. The **binaries** of `cargo shadcn-iced` (Linux, macOS and Windows, x86-64 and ARM64) are attached to the release with their build provenance, for `cargo binstall cargo-shadcn-iced`.

One-time setup:

- **crates.io**: trusted publishing cannot create a crate, so publish `shadcn_iced_core` and `cargo-shadcn-iced` once by hand (`shadcn-iced` already exists). Then add a trusted publisher to each of the three crates: repository `r4ai/shadcn-iced`, workflow `release.yml`, environment `crates-io`.
- **GitHub**: create the `crates-io` environment (restricted to `main`). For the release pull request to trigger CI, create a GitHub App with the *Contents* and *Pull requests* write permissions, and add its ID as the `RELEASE_APP_ID` variable and its key as the `RELEASE_APP_PRIVATE_KEY` secret; without it, release-plz opens the pull request with `GITHUB_TOKEN` (enable "Allow GitHub Actions to create and approve pull requests").
