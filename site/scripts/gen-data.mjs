// Generates `src/data/components.json`: every page of the gallery with the
// source of its examples (for the Code tabs) and of its component (for the
// manual installation), from `cargo run -p gallery --bin site_data`.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { exampleSource } from "./rust-source.mjs";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(site, "..");

const json = execFileSync("cargo", ["run", "--quiet", "-p", "gallery", "--bin", "site_data"], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
  stdio: ["ignore", "pipe", "inherit"],
});

const { pages } = JSON.parse(json);
const sources = new Map();
const read = (file) => {
  if (!sources.has(file)) {
    sources.set(file, readFileSync(join(root, file), "utf8").replace(/\r\n/g, "\n"));
  }
  return sources.get(file);
};

const components = pages.map((page) => {
  const component = `crates/iced_ui/src/components/${page.id.replaceAll("-", "_")}.rs`;

  return {
    id: page.id,
    title: page.title,
    description: page.description,
    source: existsSync(join(root, component))
      ? { file: component, code: read(component).trimEnd() }
      : null,
    examples: page.examples.map((example, index) => {
      const { code } = exampleSource(read(example.file), example.line, example.column);
      return {
        index,
        title: example.title,
        file: example.file,
        line: example.line,
        code,
      };
    }),
  };
});

const out = join(site, "src/data/components.json");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, JSON.stringify({ components }, null, 2) + "\n");

const count = components.reduce((n, c) => n + c.examples.length, 0);
console.log(`${components.length} components, ${count} examples → ${out}`);
