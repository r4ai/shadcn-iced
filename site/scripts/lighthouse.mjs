// Measures the built site with Lighthouse, and compares it with another build
// of it: the site of a pull request against the one of `main`, for the comment
// of `.github/workflows/lighthouse.yml`.
//
//   node scripts/lighthouse.mjs --head dist [--base ../base/dist] [--runs 10]
//     [--page /] [--head-label <sha>] [--base-label <sha>] [--out lighthouse]
//   node scripts/lighthouse.mjs --merge <dir> [--head-label <sha>] [--base-label <sha>] [--out lighthouse]
//
// `--page` measures one of the pages of `lighthouse-pages.json` (repeatable),
// for a job of the matrix of the workflow each; `--merge` reports the runs of
// every `results.json` under `dir` instead of measuring.
//
// Both builds are served like Cloudflare serves them (`scripts/serve.mjs`) and
// measured on the same machine, so that its speed and its load affect both
// alike: after a warm-up run that is left out, a page of each build in turn,
// in an order that alternates from one run to the next. The report shows the
// medians of the runs, and marks a change only when the runs of the two builds
// differ significantly (a Mann–Whitney U test), by more than a minimum: on a shared
// runner, the timings that depend on the CPU (the Total Blocking Time, the
// Speed Index, and the Performance score with them) vary by tens of percent
// from one run to the next.
//
// Writes `<out>/results.json`, with every run, and the Markdown report
// `<out>/report.md`, whose first line is `MARKER`.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import * as chromeLauncher from "chrome-launcher";
import lighthouse, { desktopConfig } from "lighthouse";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");

/** The pages measured, also read by the workflow for its matrix. */
const PAGES = JSON.parse(readFileSync(join(site, "scripts/lighthouse-pages.json"), "utf8"));

/**
 * The scores and the audits compared, with the smallest change that counts:
 * below its absolute or its relative part, a change is too small to report,
 * however significant. Scores are higher is better, audits lower is better.
 */
const SCORES = [
  ["performance", "Performance", [2, 0]],
  ["accessibility", "Accessibility", [1, 0]],
  ["best-practices", "Best Practices", [1, 0]],
  ["seo", "SEO", [1, 0]],
];
const AUDITS = [
  ["first-contentful-paint", "First Contentful Paint", "ms", [50, 0.05]],
  ["largest-contentful-paint", "Largest Contentful Paint", "ms", [50, 0.05]],
  ["total-blocking-time", "Total Blocking Time", "ms", [50, 0.2]],
  ["cumulative-layout-shift", "Cumulative Layout Shift", "unitless", [0.01, 0]],
  ["speed-index", "Speed Index", "ms", [300, 0.1]],
  ["total-byte-weight", "Total Byte Weight", "bytes", [1024, 0.005]],
];

/** The significance level of a change. */
const ALPHA = 0.05;

/** The first line of the report, to find the comment to update. */
const MARKER = "<!-- lighthouse-report -->";

const { values: args } = parseArgs({
  options: {
    head: { type: "string", default: "dist" },
    base: { type: "string" },
    runs: { type: "string", default: "10" },
    page: { type: "string", multiple: true },
    merge: { type: "string" },
    "head-label": { type: "string", default: "HEAD" },
    "base-label": { type: "string", default: "main" },
    out: { type: "string", default: "lighthouse" },
  },
});

/** Serves a build (`<dist>/client`) on `port`. */
async function serve(dist, port) {
  const server = spawn(process.execPath, [join(site, "scripts/serve.mjs"), String(port), dist], {
    stdio: ["ignore", "pipe", "inherit"],
  });
  await new Promise((ready, fail) => {
    server.stdout.once("data", ready);
    server.once("exit", (code) => fail(new Error(`scripts/serve.mjs exited with ${code}`)));
  });

  return { url: `http://localhost:${port}`, close: () => server.kill() };
}

/** One Lighthouse run of `url`, in a fresh browser (with an empty cache). */
async function measure(url) {
  const chrome = await chromeLauncher.launch({
    chromeFlags: [
      "--headless=new",
      "--no-sandbox",
      // WebGL without a GPU, for the live previews.
      "--enable-unsafe-swiftshader",
    ],
  });

  try {
    const result = await lighthouse(url, { port: chrome.port, output: "json", logLevel: "error" }, desktopConfig);
    const { categories, audits, environment, runtimeError } = result.lhr;
    if (runtimeError) throw new Error(`${url}: ${runtimeError.message}`);

    return {
      scores: Object.fromEntries(SCORES.map(([id]) => [id, (categories[id]?.score ?? NaN) * 100])),
      audits: Object.fromEntries(AUDITS.map(([id]) => [id, audits[id]?.numericValue ?? NaN])),
      // The speed of the CPU during the run, to tell a slow runner apart.
      benchmarkIndex: environment.benchmarkIndex,
    };
  } finally {
    chrome.kill();
  }
}

function median(values) {
  const sorted = values.filter(Number.isFinite).toSorted((a, b) => a - b);
  if (sorted.length === 0) return NaN;

  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

/**
 * The two-sided p-value of the Mann–Whitney U test of two samples: how likely
 * runs of the same build would differ as much. Exact, from the distribution of
 * U without ties (counted by `ways`), which suits a few runs.
 */
function mannWhitney(a, b) {
  a = a.filter(Number.isFinite);
  b = b.filter(Number.isFinite);
  const [n, m] = [a.length, b.length];
  if (n === 0 || m === 0) return 1;

  let u = 0;
  for (const x of a) for (const y of b) u += x > y ? 1 : x === y ? 0.5 : 0;

  // `ways[i][j][k]`: the orderings of i and j values where U is k.
  const ways = Array.from({ length: n + 1 }, () => Array.from({ length: m + 1 }));
  for (let i = 0; i <= n; i++) {
    for (let j = 0; j <= m; j++) {
      if (i === 0 || j === 0) {
        ways[i][j] = [1];
        continue;
      }
      ways[i][j] = Array.from({ length: i * j + 1 }, (_, k) => {
        // The largest value of the two samples is the i-th of `a` (above the
        // j values of `b`), or the j-th of `b`.
        const fromA = k >= j ? (ways[i - 1][j][k - j] ?? 0) : 0;
        return fromA + (ways[i][j - 1][k] ?? 0);
      });
    }
  }

  const counts = ways[n][m];
  const total = counts.reduce((sum, count) => sum + count, 0);
  const tail = Math.floor(Math.min(u, n * m - u));
  const below = counts.slice(0, tail + 1).reduce((sum, count) => sum + count, 0);
  return Math.min(1, (2 * below) / total);
}

/** Measures `pages` of the builds, and returns their runs by build and page. */
async function measureAll(pages, runs) {
  const builds = [{ name: "head", dist: resolve(args.head) }];
  if (args.base) builds.push({ name: "base", dist: resolve(args.base) });

  const servers = await Promise.all(builds.map((build, index) => serve(build.dist, 4400 + index)));
  const measured = Object.fromEntries(
    builds.map((build) => [build.name, Object.fromEntries(pages.map((page) => [page, []]))]),
  );

  try {
    // oxlint-disable no-await-in-loop
    for (let run = -1; run < runs; run++) {
      for (const page of pages) {
        // Alternated, so that neither build is always measured first.
        const order = run % 2 === 0 ? builds.toReversed() : builds;
        for (const build of order) {
          const server = servers[builds.indexOf(build)];
          console.log(`${run < 0 ? "Warm-up" : `Run ${run + 1}/${runs}`}: ${build.name} ${page}`);
          const sample = await measure(server.url + page);
          if (run >= 0) measured[build.name][page].push(sample);
        }
      }
    }
    // oxlint-enable no-await-in-loop
  } finally {
    for (const server of servers) server.close();
  }

  return measured;
}

/** The runs of every `results.json` under `dir`, by build and page. */
function mergeAll(dir) {
  const merged = {};
  const files = readdirSync(dir, { recursive: true }).filter((file) => file.endsWith("results.json"));
  for (const file of files) {
    const { samples } = JSON.parse(readFileSync(join(dir, file), "utf8"));
    for (const [name, pages] of Object.entries(samples)) {
      for (const [page, runs] of Object.entries(pages)) {
        ((merged[name] ??= {})[page] ??= []).push(...runs);
      }
    }
  }

  return merged;
}

const samples = args.merge ? mergeAll(resolve(args.merge)) : await measureAll(args.page ?? PAGES, Number(args.runs));
if (!samples.head) throw new Error("No runs to report.");

const pages = PAGES.filter((page) => samples.head[page]);
const missing = PAGES.filter((page) => !samples.head[page]);
const runs = Math.min(...pages.map((page) => samples.head[page].length));

/** The values of a score or an audit in the runs of a page of a build. */
const values = (name, page, kind, id) => samples[name][page].map((sample) => sample[kind][id]);

/** A value in its unit. */
function format(value, unit) {
  if (!Number.isFinite(value)) return "–";

  switch (unit) {
    case "ms":
      return value >= 1000 ? `${(value / 1000).toFixed(2)} s` : `${Math.round(value)} ms`;
    case "bytes":
      return `${(value / 1024).toFixed(1)} KiB`;
    case "unitless":
      return value.toFixed(3);
    default:
      return String(Math.round(value));
  }
}

/**
 * The change of a score or an audit of a page, from `main` to the pull
 * request: its medians, its text, its p-value, and whether it is better (1),
 * worse (-1) or not a change (0).
 */
function compare(page, kind, id, unit, [absolute, relative]) {
  const head = median(values("head", page, kind, id));
  if (!samples.base) return { head, text: "", change: 0 };

  const base = median(values("base", page, kind, id));
  if (!Number.isFinite(base) || !Number.isFinite(head)) return { base, head, text: "", change: 0 };

  const difference = head - base;
  const p = mannWhitney(values("head", page, kind, id), values("base", page, kind, id));
  const large = Math.abs(difference) >= absolute && Math.abs(difference) >= relative * Math.abs(base);
  const better = kind === "scores" ? difference > 0 : difference < 0;
  const change = p < ALPHA && large ? (better ? 1 : -1) : 0;

  const sign = difference > 0 ? "+" : difference < 0 ? "−" : "±";
  const ratio = base === 0 ? 0 : Math.round(Math.abs(difference / base) * 100);
  const percent = kind === "scores" ? "" : ` (${ratio === 0 ? "±" : sign}${ratio}%)`;
  const mark = change > 0 ? " 🟢" : change < 0 ? " 🔴" : "";

  return { base, head, p, change, text: `${sign}${format(Math.abs(difference), unit)}${percent}${mark}` };
}

/** A cell of a table: the median, or the change from `main` to it. */
function cell(page, kind, id, unit, threshold) {
  const { base, head, text } = compare(page, kind, id, unit, threshold);
  if (!samples.base) return format(head, unit);

  return `${format(base, unit)} → **${format(head, unit)}** ${text}`;
}

const metrics = [
  ...SCORES.map(([id, label, threshold]) => ["scores", id, label, "score", threshold]),
  ...AUDITS.map(([id, label, unit, threshold]) => ["audits", id, label, unit, threshold]),
];

const lines = [MARKER, "## Lighthouse", ""];

if (samples.base) {
  const changes = { regressions: [], improvements: [] };
  for (const page of pages) {
    for (const [kind, id, label, unit, threshold] of metrics) {
      const { change, text, p } = compare(page, kind, id, unit, threshold);
      const line = `- \`${page}\`: ${label} ${text} (p = ${p.toFixed(3)})`;
      if (change < 0) changes.regressions.push(line);
      if (change > 0) changes.improvements.push(line);
    }
  }

  lines.push(
    `Desktop, the medians of ${runs} runs of each page: \`${args["base-label"]}\` → **\`${args["head-label"]}\`**, each page measured in turns with both builds on a runner of its own. 🔴 and 🟢 mark the changes where the runs of the two builds differ (Mann–Whitney U test, p < ${ALPHA}) by more than a minimum.`,
    "",
  );

  if (changes.regressions.length === 0 && changes.improvements.length === 0) {
    lines.push("No significant change.", "");
  }
  for (const [mark, name, list] of [
    ["🔴", "regression", changes.regressions],
    ["🟢", "improvement", changes.improvements],
  ]) {
    if (list.length === 0) continue;
    lines.push(`### ${mark} ${list.length} ${name}${list.length > 1 ? "s" : ""}`, "", ...list, "");
  }
} else {
  lines.push(
    `Desktop, the medians of ${runs} runs of each page of \`${args["head-label"]}\`. No build of \`${args["base-label"]}\` to compare with.`,
    "",
  );
}

if (missing.length > 0) {
  lines.push(`⚠️ Not measured: ${missing.map((page) => `\`${page}\``).join(", ")}.`, "");
}

lines.push("### Scores", "");
lines.push(`| Page | ${SCORES.map(([, label]) => label).join(" | ")} |`);
lines.push(`| --- |${" --- |".repeat(SCORES.length)}`);
for (const page of pages) {
  const cells = SCORES.map(([id, , threshold]) => cell(page, "scores", id, "score", threshold));
  lines.push(`| \`${page}\` | ${cells.join(" | ")} |`);
}
lines.push("");

lines.push("<details>", "<summary>Metrics</summary>", "");
for (const page of pages) {
  lines.push(`#### \`${page}\``, "", "| Metric | Value |", "| --- | --- |");
  for (const [id, label, unit, threshold] of AUDITS) {
    lines.push(`| ${label} | ${cell(page, "audits", id, unit, threshold)} |`);
  }
  lines.push("");
}
const benchmarks = Object.keys(samples).map((name) => {
  const indices = pages.flatMap((page) => samples[name][page].map((sample) => sample.benchmarkIndex));
  return `${name === "head" ? args["head-label"] : args["base-label"]} ${Math.round(median(indices))}`;
});
lines.push(`The median CPU benchmark index of the runner: ${benchmarks.join(", ")}.`, "", "</details>", "");

const out = resolve(args.out);
mkdirSync(out, { recursive: true });
writeFileSync(join(out, "results.json"), `${JSON.stringify({ runs, samples }, null, 2)}\n`);
writeFileSync(join(out, "report.md"), lines.join("\n"));
console.log(lines.join("\n"));
