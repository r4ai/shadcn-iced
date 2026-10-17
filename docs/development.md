# Development

This guide covers building, testing, running the component gallery, generating visual snapshots, and working on the documentation website.

## Prerequisites

- **Rust**: 1.88 or newer (Edition 2024)
- **Node.js & pnpm** (only required when working on `site/`): Node.js 22+ and `pnpm`

## Building and Testing

Run the standard Cargo checks across the workspace:

```sh
# Check formatting
cargo fmt --all --check

# Run Clippy across all workspace members and targets
cargo clippy --workspace --all-targets -- -D warnings

# Run unit and integration tests
cargo test --workspace
```

### Verifying Copy-and-Paste Isolation

Every component in `crates/shadcn_iced/src/components/` must compile when copied into a standalone project that only depends on `iced` and `shadcn_iced_core`.

To verify this locally (also run in CI):

```sh
./scripts/check-copy.sh
```

This script builds `cargo-shadcn-iced`, initializes a temporary Cargo project, runs `cargo shadcn-iced add --all`, and checks the resulting crate with `RUSTFLAGS="-D warnings"`.

## Running the Examples

Run the desktop component gallery (which includes all 66 components and an interactive theme customizer) or the minimal `hello` example:

```sh
cargo run -p gallery
cargo run -p hello
```

To open the gallery directly on a specific component page, set the `GALLERY_PAGE` environment variable to the component ID:

```sh
GALLERY_PAGE=sonner cargo run -p gallery
```

The example apps (`examples/dashboard`, `examples/settings`) are crates of their own: a library with the state, `update` and `view` of the app, and a `main.rs` that runs it. The gallery maps their views (`pages/apps.rs`) for the website's `/examples` pages, and the `snapshot` binary renders them by id (`dashboard`, `settings`):

```sh
cargo run -p dashboard
cargo run -p settings
cargo run -p gallery --bin snapshot -- settings:dark --click Notifications --size 1280x1000
```

## Headless Visual QA (`snapshot`)

The `gallery` package includes a `snapshot` binary that renders component pages headlessly using `iced_test::Simulator` and saves PNG images to `target/snapshots/`.

### Rendering All Pages

Use the `tiny-skia` software backend for deterministic rendering without a display server:

```sh
ICED_TEST_BACKEND=tiny-skia cargo run -p gallery --bin snapshot
```

### Rendering Specific Pages and Interactions

Pass one or more `page[:mode]` targets along with interaction and theme flags:

```sh
# Click a trigger and hover a submenu item
cargo run -p gallery --bin snapshot -- dropdown-menu:dark --live --click "Open" --hover 700,538

# Open a dialog in light mode
cargo run -p gallery --bin snapshot -- dialog:light --click "Edit Profile"

# Customize theme and chart colors
cargo run -p gallery --bin snapshot -- chart:light --accent blue --chart rose

# Test inverted menu styling
cargo run -p gallery --bin snapshot -- dropdown-menu:light --menu-color inverted --live --click "Open"

# Capture an in-progress animation frame (animations are disabled by default)
cargo run -p gallery --bin snapshot -- dropdown-menu:light --animations --live --click "Open" --wait 70
```

Available flags for `snapshot`:

- **Targets**: `<page-id>` or `<page-id>:light` / `<page-id>:dark`
- **Actions**:
  - `--click "<label>"`: Click the text widget matching `<label>`
  - `--at <x>,<y>`: Click at a logical coordinate
  - `--hover <x>,<y>`: Move the cursor to a logical coordinate
  - `--type "<text>"`: Type text into the focused widget
  - `--key <escape|enter|tab|down|up|space>`: Press a key
  - `--wait <ms>`: Advance time (useful for hover delays and animations)
  - `--live`: Perform subsequent actions on the final view without rebuilding, preserving transient widget state (hover, focus, self-managed overlays)
- **Theme Options**:
  - `--base <name>`, `--accent <name>`, `--chart <name>`
  - `--radius <none|small|medium|large>`
  - `--menu-color <inverted|translucent|inverted-translucent>`
  - `--menu-accent bold`
  - `--heading <geist|inter>`
  - `--customize`: Open the theme customizer sheet
  - `--animations`: Enable animations during the run

## Conformance with `shadcn/ui`

[`conformance/`](../conformance/README.md) runs the same scenarios (pointer and keyboard interactions, expected texts, snapshots) against the latest `shadcn/ui`, in Chromium, and against `shadcn_iced`, headlessly, then compares them. It needs Node.js 24+ and pnpm:

```sh
cd conformance
pnpm install
pnpm exec playwright install chromium
pnpm qa        # render with iced, run shadcn/ui, compare, write out/report/index.html
```

A run fails when a snapshot differs from `shadcn/ui` more than its accepted score in `conformance/baseline.json`, or when an expectation fails that is not listed there as known. After improving a component, record its new scores with `pnpm baseline <component>/` and commit them with the fix. `pnpm shadcn:update` updates the reference to the latest `shadcn/ui`; the `Conformance` workflow does it every week to catch upstream changes.

## Documentation Website (`site/`)

The website in `site/` is built with TanStack Start, Solid, Vite, and Tailwind CSS. It displays each gallery example alongside its extracted Rust source code and a live WebAssembly preview.

### How Site Assets Are Generated

1. **`pnpm data`** (`site/scripts/gen-data.mjs`): Runs `cargo run -p gallery --bin site_data` to locate every `example(...)` call in the gallery, extracts the corresponding Rust source snippets (including referenced imports and helpers) and component source files, and writes `site/src/data/components.json`.
2. **`pnpm wasm`** (`site/scripts/build-wasm.mjs`): Compiles `examples/gallery-web` for `wasm32-unknown-unknown` using the `wasm` Cargo profile, generates bindings with `wasm-bindgen`, and optimizes the binary with `wasm-opt` into `site/public/wasm/`. Build-machine paths are remapped before compilation, and the final binary is checked for unremapped prefixes before it can be deployed.

### Local Development Setup

Install the WebAssembly target and matching `wasm-bindgen-cli` version:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <version-in-Cargo.lock> --locked
```

Then install dependencies, generate the gallery data and WASM bundle, and start the dev server:

```sh
cd site
pnpm install
pnpm data && pnpm wasm   # Re-run after modifying examples or components
pnpm dev
```

### Linting and Building the Site

```sh
cd site
pnpm lint        # Lint with oxlint
pnpm fmt:check   # Check formatting with oxfmt
pnpm check       # Run TypeScript type checking
pnpm build       # Prerender static pages to site/dist/client
```

### Deployment

The `Site` GitHub Actions workflow (`.github/workflows/site.yml`) builds the site and deploys `site/dist/client` to Cloudflare Workers static assets (`site/wrangler.jsonc`) on pushes to `main`. Deployment requires the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets configured in the repository's `production` environment.
