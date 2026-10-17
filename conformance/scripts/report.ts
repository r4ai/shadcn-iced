/**
 * Writes `out/report/index.html`: every snapshot of the last Playwright run
 * next to its iced counterpart and their diff, worst first.
 *
 * ```sh
 * pnpm report
 * ```
 */
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs"
import { join } from "node:path"

import { allowance, out, type RunRecord } from "../tests/spec.ts"

const runsDirectory = join(out, "runs")

if (!existsSync(runsDirectory)) {
  console.error("No runs found: run `pnpm test` first.")
  process.exit(1)
}

const runs: RunRecord[] = readdirSync(runsDirectory)
  .filter((name) => name.endsWith(".json"))
  .map((name) => JSON.parse(readFileSync(join(runsDirectory, name), "utf8")))

type Status = "fail" | "regressed" | "improved" | "new" | "ok"

interface Row {
  run: RunRecord
  snapshot: RunRecord["snapshots"][number] | null
  status: Status
}

function status(run: RunRecord, snapshot: Row["snapshot"]): Status {
  if (
    run.reference.failures.length ||
    run.iced.failures.length ||
    run.iced.status !== "ok"
  ) {
    return "fail"
  }
  if (!snapshot || snapshot.score === null) return "fail"
  if (snapshot.baseline === null) return "new"
  if (snapshot.score > snapshot.baseline + allowance(snapshot.baseline)) {
    return "regressed"
  }
  if (snapshot.score < snapshot.baseline - allowance(snapshot.baseline)) {
    return "improved"
  }
  return "ok"
}

const rows: Row[] = runs.flatMap<Row>((run) =>
  run.snapshots.length
    ? run.snapshots.map((snapshot) => ({
        run,
        snapshot,
        status: status(run, snapshot),
      }))
    : [{ run, snapshot: null, status: "fail" }]
)

rows.sort(
  (a, b) =>
    (b.snapshot?.score ?? 2) - (a.snapshot?.score ?? 2) ||
    (a.snapshot?.key ?? "").localeCompare(b.snapshot?.key ?? "")
)

const escape = (value: string) =>
  value.replace(
    /[&<>"]/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!
  )

const size = (box: { width: number; height: number } | null) =>
  box ? `${box.width / 2}×${box.height / 2}` : "–"

function card({ run, snapshot, status }: Row) {
  const title = snapshot?.key ?? `${run.component}/${run.scenario}/${run.theme}`
  const failures = [
    ...run.reference.failures.map((failure) => `shadcn/ui: ${failure}`),
    ...(run.iced.status === "ok" ? [] : [`shadcn_iced: ${run.iced.status}`]),
    ...run.iced.failures.map((failure) => `shadcn_iced: ${failure}`),
  ]

  const score =
    snapshot?.score == null
      ? "–"
      : `${snapshot.score.toFixed(4)}${
          snapshot.baseline == null
            ? ""
            : ` <small>(baseline ${snapshot.baseline.toFixed(4)})</small>`
        }`

  const images = snapshot
    ? ["reference", "iced", "diff"]
        .map(
          (kind) =>
            `<figure><figcaption>${kind === "reference" ? "shadcn/ui" : kind === "iced" ? "shadcn_iced" : "diff"}</figcaption><img loading="lazy" src="../${kind}/${snapshot.path}" alt="${kind}"></figure>`
        )
        .join("")
    : ""

  return `<article data-status="${status}" data-key="${escape(title)}">
  <header>
    <h2>${escape(title)}</h2>
    <span class="status ${status}">${status}</span>
    <span class="score">score ${score}</span>
    <span class="size">size ${size(snapshot?.reference ?? null)} vs ${size(snapshot?.iced ?? null)}</span>
  </header>
  ${failures.length ? `<ul class="failures">${failures.map((f) => `<li>${escape(f)}</li>`).join("")}</ul>` : ""}
  <div class="images">${images}</div>
</article>`
}

const counts = rows.reduce<Record<string, number>>((counts, row) => {
  counts[row.status] = (counts[row.status] ?? 0) + 1
  return counts
}, {})

const scored = rows.filter((row) => row.snapshot?.score != null)
const mean =
  scored.reduce((sum, row) => sum + row.snapshot!.score!, 0) /
  Math.max(1, scored.length)

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Conformance report</title>
<style>
  :root { color-scheme: light dark; --bg: #fff; --fg: #111; --muted: #666; --border: #ddd; --card: #fafafa; }
  @media (prefers-color-scheme: dark) { :root { --bg: #111; --fg: #eee; --muted: #999; --border: #333; --card: #181818; } }
  body { margin: 0; padding: 16px; font: 14px/1.5 system-ui, sans-serif; background: var(--bg); color: var(--fg); }
  h1 { font-size: 20px; margin: 0 0 8px; }
  .summary, .controls { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; margin-bottom: 12px; color: var(--muted); }
  input[type=search] { padding: 4px 8px; min-width: 240px; }
  article { border: 1px solid var(--border); background: var(--card); border-radius: 8px; padding: 12px; margin-bottom: 12px; }
  article header { display: flex; flex-wrap: wrap; gap: 12px; align-items: baseline; }
  h2 { font-size: 15px; margin: 0; font-family: ui-monospace, monospace; }
  .score, .size { color: var(--muted); }
  .status { font-weight: 600; text-transform: uppercase; font-size: 12px; }
  .fail, .regressed { color: #d33; } .improved { color: #2a2; } .new { color: #c80; } .ok { color: var(--muted); }
  .failures { color: #d33; margin: 8px 0; }
  .images { display: flex; flex-wrap: wrap; gap: 12px; margin-top: 8px; }
  figure { margin: 0; } figcaption { color: var(--muted); font-size: 12px; }
  img { max-width: 100%; border: 1px solid var(--border); image-rendering: pixelated; background: repeating-conic-gradient(#8882 0 25%, transparent 0 50%) 0 0 / 16px 16px; }
  .zoom img { width: auto; zoom: 2; }
</style>
</head>
<body>
<h1>shadcn_iced vs shadcn/ui</h1>
<div class="summary">
  <span>${rows.length} snapshots</span>
  ${Object.entries(counts)
    .map(
      ([status, count]) =>
        `<span class="status ${status}">${count} ${status}</span>`
    )
    .join("")}
  <span>mean score ${mean.toFixed(4)}</span>
</div>
<div class="controls">
  <input type="search" id="filter" placeholder="Filter by key, e.g. button/ or /dark">
  <label><input type="checkbox" id="problems"> Problems only</label>
  <label><input type="checkbox" id="zoom"> Zoom</label>
</div>
<main>
${rows.map(card).join("\n")}
</main>
<script>
  const filter = document.getElementById("filter")
  const problems = document.getElementById("problems")
  const zoom = document.getElementById("zoom")
  function apply() {
    const query = filter.value.trim()
    for (const article of document.querySelectorAll("article")) {
      const matches = article.dataset.key.includes(query)
      const problem = ["fail", "regressed", "new"].includes(article.dataset.status)
      article.hidden = !matches || (problems.checked && !problem)
    }
    document.body.classList.toggle("zoom", zoom.checked)
  }
  for (const input of [filter, problems, zoom]) input.addEventListener("input", apply)
</script>
</body>
</html>
`

mkdirSync(join(out, "report"), { recursive: true })
writeFileSync(join(out, "report/index.html"), html)
console.log(
  `${join(out, "report/index.html")}: ${rows.length} snapshots, ${JSON.stringify(counts)}, mean score ${mean.toFixed(4)}`
)
