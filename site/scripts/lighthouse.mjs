// Measures the built site with Lighthouse, and compares it with another build
// of it: the site of a pull request against the one of `main`, for the comment
// of `.github/workflows/lighthouse.yml`.
//
//   node scripts/lighthouse.mjs --head dist [--base ../base/dist] [--runs 3]
//     [--head-label <sha>] [--base-label <sha>] [--out lighthouse]
//
// Both builds are served like Cloudflare serves them (`scripts/serve.mjs`) and
// measured on the same machine, in turns, so that its load affects both alike.
// Each value is the median of the runs. Writes `<out>/results.json` and the
// Markdown report `<out>/report.md`, whose first line is `MARKER`.
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import * as chromeLauncher from "chrome-launcher";
import lighthouse, { desktopConfig } from "lighthouse";

/** The pages measured: the home page and one page of each kind. */
const PAGES = ["/", "/docs/components/button/", "/charts/area/", "/examples/dashboard/", "/examples/settings/"];

const CATEGORIES = [
  ["performance", "Performance"],
  ["accessibility", "Accessibility"],
  ["best-practices", "Best Practices"],
  ["seo", "SEO"],
];

/**
 * The audits compared, all of them lower is better, with the smallest change
 * that counts: below its absolute or its relative part, a change is noise.
 * The thresholds come from runs of the same build against itself, where the
 * Total Blocking Time and the Speed Index of the live previews vary most.
 */
const AUDITS = [
  ["first-contentful-paint", "First Contentful Paint", "ms", [100, 0.1]],
  ["largest-contentful-paint", "Largest Contentful Paint", "ms", [100, 0.1]],
  ["total-blocking-time", "Total Blocking Time", "ms", [100, 0.5]],
  ["cumulative-layout-shift", "Cumulative Layout Shift", "unitless", [0.01, 0]],
  ["speed-index", "Speed Index", "ms", [1000, 0.25]],
  ["total-byte-weight", "Total Byte Weight", "bytes", [1024, 0.01]],
];

/** The smallest change of a score that counts. */
const SCORE_THRESHOLD = [3, 0];

/** The first line of the report, to find the comment to update. */
const MARKER = "<!-- lighthouse-report -->";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");

const { values: args } = parseArgs({
  options: {
    head: { type: "string", default: "dist" },
    base: { type: "string" },
    runs: { type: "string", default: "3" },
    "head-label": { type: "string", default: "HEAD" },
    "base-label": { type: "string", default: "main" },
    out: { type: "string", default: "lighthouse" },
  },
});
const runs = Number(args.runs);

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
    const { categories, audits, runtimeError } = result.lhr;
    if (runtimeError) throw new Error(`${url}: ${runtimeError.message}`);

    return {
      categories: Object.fromEntries(CATEGORIES.map(([id]) => [id, (categories[id]?.score ?? NaN) * 100])),
      audits: Object.fromEntries(AUDITS.map(([id]) => [id, audits[id]?.numericValue ?? NaN])),
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

/** The medians of the runs of a page. */
const summarize = (samples) => ({
  categories: Object.fromEntries(
    CATEGORIES.map(([id]) => [id, median(samples.map((sample) => sample.categories[id]))]),
  ),
  audits: Object.fromEntries(AUDITS.map(([id]) => [id, median(samples.map((sample) => sample.audits[id]))])),
});

const builds = [{ name: "head", dist: resolve(args.head) }];
if (args.base) builds.push({ name: "base", dist: resolve(args.base) });

const servers = await Promise.all(builds.map((build, index) => serve(build.dist, 4400 + index)));
const samples = Object.fromEntries(
  builds.map((build) => [build.name, Object.fromEntries(PAGES.map((page) => [page, []]))]),
);

try {
  // A page of each build in turn, then the next run.
  // oxlint-disable no-await-in-loop
  for (let run = 0; run < runs; run++) {
    for (const page of PAGES) {
      for (const [index, build] of builds.entries()) {
        console.log(`Run ${run + 1}/${runs}: ${build.name} ${page}`);
        samples[build.name][page].push(await measure(servers[index].url + page));
      }
    }
  }
  // oxlint-enable no-await-in-loop
} finally {
  for (const server of servers) server.close();
}

const results = Object.fromEntries(
  Object.entries(samples).map(([name, pages]) => [
    name,
    Object.fromEntries(Object.entries(pages).map(([page, measured]) => [page, summarize(measured)])),
  ]),
);

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
 * The change from `base` to `head`: its text, and whether it is better (1),
 * worse (-1) or within the noise (0).
 */
function delta(base, head, unit, [absolute, relative]) {
  if (!Number.isFinite(base) || !Number.isFinite(head)) return { text: "", change: 0 };

  const difference = head - base;
  const beyond = Math.abs(difference) >= absolute && Math.abs(difference) >= relative * Math.abs(base);
  // Scores: higher is better. Audits: lower is better.
  const change = beyond ? Math.sign(difference) * (unit === "score" ? 1 : -1) : 0;

  const sign = difference > 0 ? "+" : difference < 0 ? "−" : "±";
  const ratio = base === 0 ? 0 : Math.round(Math.abs(difference / base) * 100);
  const percent = unit === "score" ? "" : ` (${ratio === 0 ? "±" : sign}${ratio}%)`;
  const mark = change > 0 ? " 🟢" : change < 0 ? " 🔴" : "";

  return { text: `${sign}${format(Math.abs(difference), unit)}${percent}${mark}`, change };
}

/** A cell of a table: the value, or the change from `main` to it. */
function cell(page, kind, id, unit, threshold) {
  const head = results.head[page][kind][id];
  if (!results.base) return format(head, unit);

  const base = results.base[page][kind][id];
  const { text } = delta(base, head, unit, threshold);
  return `${format(base, unit)} → **${format(head, unit)}** ${text}`;
}

const lines = [MARKER, "## Lighthouse", ""];

if (results.base) {
  const regressions = [];
  const improvements = [];
  const compare = (page, kind, id, label, unit, threshold) => {
    const base = results.base[page][kind][id];
    const { change, text } = delta(base, results.head[page][kind][id], unit, threshold);
    if (change < 0) regressions.push(`\`${page}\`: ${label} ${text}`);
    if (change > 0) improvements.push(`\`${page}\`: ${label} ${text}`);
  };

  for (const page of PAGES) {
    for (const [id, label] of CATEGORIES) {
      compare(page, "categories", id, label, "score", SCORE_THRESHOLD);
    }
    for (const [id, label, unit, threshold] of AUDITS) {
      compare(page, "audits", id, label, unit, threshold);
    }
  }

  lines.push(
    `Desktop, the median of ${runs} runs of each page: \`${args["base-label"]}\` → **\`${args["head-label"]}\`**, measured in turns on the same runner. 🟢 better and 🔴 worse, beyond the noise.`,
    "",
  );

  if (regressions.length === 0 && improvements.length === 0) {
    lines.push("No change beyond the noise.", "");
  }
  for (const [mark, name, list] of [
    ["🔴", "regression", regressions],
    ["🟢", "improvement", improvements],
  ]) {
    if (list.length === 0) continue;
    lines.push(`### ${mark} ${list.length} ${name}${list.length > 1 ? "s" : ""}`, "");
    lines.push(...list.map((line) => `- ${line}`), "");
  }
} else {
  lines.push(
    `Desktop, the median of ${runs} runs of each page of \`${args["head-label"]}\`. No build of \`${args["base-label"]}\` to compare with.`,
    "",
  );
}

lines.push("### Scores", "");
lines.push(`| Page | ${CATEGORIES.map(([, label]) => label).join(" | ")} |`);
lines.push(`| --- |${" --- |".repeat(CATEGORIES.length)}`);
for (const page of PAGES) {
  const cells = CATEGORIES.map(([id]) => cell(page, "categories", id, "score", SCORE_THRESHOLD));
  lines.push(`| \`${page}\` | ${cells.join(" | ")} |`);
}
lines.push("");

lines.push("<details>", "<summary>Metrics</summary>", "");
for (const page of PAGES) {
  lines.push(`#### \`${page}\``, "", "| Metric | Value |", "| --- | --- |");
  for (const [id, label, unit, threshold] of AUDITS) {
    lines.push(`| ${label} | ${cell(page, "audits", id, unit, threshold)} |`);
  }
  lines.push("");
}
lines.push("</details>", "");

const out = resolve(args.out);
mkdirSync(out, { recursive: true });
writeFileSync(join(out, "results.json"), `${JSON.stringify({ runs, results }, null, 2)}\n`);
writeFileSync(join(out, "report.md"), lines.join("\n"));
console.log(lines.join("\n"));
