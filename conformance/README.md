# Conformance

Checks that `shadcn_iced` looks and behaves like [shadcn/ui](https://ui.shadcn.com), by running the same scenarios against both and comparing the results.

- **shadcn/ui side**: this directory is a Vite + React app initialized with the shadcn CLI (`base-nova` style, Base UI), so `src/components/ui` holds the unmodified, latest registry components. Playwright drives it.
- **iced side**: [`iced/`](iced) is a headless runner (`cargo run -p conformance`) that renders the same scenarios with `shadcn_iced` and the `tiny-skia` renderer, keeping the widget tree across interactions like the real iced runtime.

Both sides read the scenarios from [`scenarios/`](scenarios), perform their steps (hover, click, key presses, typing, ...), check their expectations, and take snapshots at a scale factor of 2. Playwright then compares every snapshot pair and fails when:

1. an expectation does not hold for shadcn/ui (the scenario is wrong, or shadcn/ui changed its behavior),
2. an expectation does not hold for `shadcn_iced`, or the iced runner could not perform a step,
3. a snapshot differs from shadcn/ui more than its accepted score in [`baseline.json`](baseline.json).

## Usage

Requires Rust, Node.js 24+ and pnpm.

```sh
cd conformance
pnpm install
pnpm exec playwright install chromium

pnpm qa          # = pnpm iced && pnpm test; pnpm report
```

| Command | What it does |
| :--- | :--- |
| `pnpm iced [FILTER...]` | Renders the scenarios with `shadcn_iced` into `out/iced` (and their outcomes into `out/iced/runs`). Filters match `component/scenario` keys. |
| `pnpm test [-g FILTER]` | Runs the scenarios against shadcn/ui into `out/reference`, compares them with `out/iced`, and writes `out/diff` and `out/runs`. |
| `pnpm report` | Writes `out/report/index.html`, every snapshot pair with its diff, worst first. Serve `out/` (e.g. `pnpm exec vite out`) to browse it. |
| `pnpm baseline [FILTER...]` | Records the scores of the last run in `baseline.json`. |
| `pnpm shadcn:update` | Updates the shadcn CLI and reinstalls every component from the registry. |
| `pnpm dev` | Serves the fixtures; `/` lists them, `/?component=button&scenario=default&theme=dark` shows one. |

## Scores and the baseline

Text is rasterized differently by browsers and iced, so snapshots are never pixel-identical. The **score** of a snapshot pair is the share of its content (pixels that are not background in either image) that differs: `0` is identical and `1` is entirely different. A pixel only counts when no pixel of the other image within one physical pixel (half a logical pixel) has its color or lies between its neighbors' colors, so anti-aliasing and sub-pixel offsets are ignored while any difference of a logical pixel or more in size, position or color counts. The diff images show counted pixels in red and tolerated ones in yellow. The report also shows the size of the content of both images, which makes layout differences (paddings, heights, gaps) easy to spot.

`baseline.json` records the accepted score of every snapshot. A run fails when a score gets worse than its baseline by more than `max(0.01, 10%)`, so any regression is caught while the known differences are fixed one by one. After improving a component:

```sh
pnpm iced button && pnpm test -g button && pnpm baseline button/
```

and commit `baseline.json` with the fix. On CI, a snapshot without a baseline fails.

## Determinism

Snapshots must not depend on the machine running the suite:

- **Fonts**: both sides only use the bundled Geist and Geist Mono (`@fontsource-variable/*` on the web, `shadcn_iced::font` in iced); the page loads them before rendering and the runner checks that Geist is loaded, so no text falls back to a system font.
- **Rasterization**: Chromium runs with `--font-render-hinting=none --disable-lcd-text --disable-gpu --force-color-profile=srgb --hide-scrollbars` (no hinting, grayscale anti-aliasing, software rendering, sRGB, no native scrollbars); iced always renders with `tiny-skia` at a scale factor of 2 with no system dependency. Playwright, hence Chromium, is pinned to an exact version.
- **Time and locale**: the browser clock is fixed at `2025-06-15T12:00:00Z` (`NOW` in `tests/spec.ts`; iced fixtures must use this date as "today"), with the `en-US` locale, the `UTC` timezone and reduced motion. Animations are disabled on both sides.
- **Server**: the suite runs against a production build served by `vite preview` on `CONFORMANCE_PORT` (4517 by default), never reusing a running server, which could serve another checkout.
- **Scoring**: the comparison tolerates anti-aliasing and sub-pixel offsets (see above), and scores may get worse by `max(0.01, 10%)` before failing.

The remaining differences between operating systems are small, but the baseline is produced on Linux: the `Conformance` workflow uploads the `baseline.json` of its run with its report, to be committed when the scores of another platform are off.

## Writing scenarios

A scenario file covers a component:

```json
{
  "$schema": "./schema.json",
  "component": "dropdown-menu",
  "scenarios": [
    {
      "id": "basic",
      "viewport": [320, 280],
      "steps": [
        { "snapshot": "closed" },
        { "click": "Open" },
        { "expect": { "visible": ["Profile", "Settings"] } },
        { "snapshot": "open" },
        { "press": "ArrowDown" },
        { "snapshot": "highlighted" },
        { "press": "Escape" },
        { "expect": { "hidden": ["Profile"] } }
      ]
    }
  ]
}
```

- `viewport` is the logical size of the page (default `[320, 120]`); fixtures are placed at the top-left corner with a padding of 16.
- `themes` defaults to `["light", "dark"]`.
- Steps: `snapshot`, `hover` (with `"steps": n` to move the pointer in n moves instead of jumping), `click`, `rightClick`, `drag` (`{ "from": ..., "to": ... }`), `press` (Playwright key names: `Enter`, `Escape`, `Tab`, `Shift+Tab`, `ArrowDown`, `Space`, `a`, ...), `type`, `wait` (milliseconds) and `expect` (`visible` and `hidden` texts, matched exactly). Without steps, a scenario takes a single snapshot named `default`.
- Pointer locators are the content of a visible text (`"Open"`, `{ "text": "Open", "nth": 1 }` for the second one, or `{ "text": "Profile", "exact": false }` for a text containing it, e.g. a menu item with a shortcut) or a position (`{ "at": [100, 40] }`). Prefer texts: they do not depend on the layout. On the web, texts are matched against the whole text of an element: CSS transforms (`capitalize`) are not applied, and input values or placeholders are not texts (show a readout like "You typed: ..." instead).
- iced has no built-in focus traversal: like iced applications do, the runner moves the focus with `focus_next`/`focus_previous` when no widget captures Tab.

[`scenarios/schema.json`](scenarios/schema.json) describes the format; [`tests/spec.ts`](tests/spec.ts) and [`iced/src/spec.rs`](iced/src/spec.rs) implement it.

Every scenario needs a fixture on both sides, keyed by its id:

- `src/fixtures/<component>.tsx` exports the React fixtures, written with the components of `src/components/ui` the way the shadcn/ui documentation uses them;
- `iced/src/fixtures/<component>.rs` has the iced counterparts, registered in `iced/src/fixtures/mod.rs`. State lives in a shared key-value `Store` (`store::set("checked")` makes a callback that stores its argument) so fixtures stay one-liners; `store.set_editor("text", "...")` in `init`, `store.editor("text")` and `store::edit("text")` hold the content of text editors.

Fixtures must be equivalent: same texts, icons, sizes and layout (`flex gap-2` is `row![..].spacing(8)`), and the same initial state.

## Pitfalls

- Animations are disabled on both sides (`?static` disables CSS transitions and animations; the iced runner calls `shadcn_iced::animation::set_enabled(false)`). Delays (tooltips, hover cards) still apply: use `wait`.
- The pointer stays where the last `hover` or `click` left it, which keeps hover styles in later snapshots on both sides.
- `expect` waits up to 2 seconds on the shadcn/ui side and checks immediately on the iced side.
