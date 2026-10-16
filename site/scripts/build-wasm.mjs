// Builds the live previews (`examples/gallery-web`) to `public/wasm/`:
// cargo → wasm-bindgen → wasm-opt.
//
// Needs the `wasm32-unknown-unknown` target and the wasm-bindgen CLI of the
// version in Cargo.lock:
//
//   rustup target add wasm32-unknown-unknown
//   cargo install wasm-bindgen-cli --version <version> --locked
import { execFileSync } from "node:child_process";
import { readFileSync, rmSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(site, "..");
const out = join(site, "public/wasm");
const run = (command, args) => execFileSync(command, args, { cwd: root, stdio: "inherit" });

const lock = readFileSync(join(root, "Cargo.lock"), "utf8");
const bindgen = lock.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/)?.[1];
const installed = execFileSync("wasm-bindgen", ["--version"], { encoding: "utf8" }).trim();
if (bindgen && !installed.endsWith(bindgen)) {
  throw new Error(
    `${installed} does not match wasm-bindgen ${bindgen} of Cargo.lock: ` +
      `cargo install wasm-bindgen-cli --version ${bindgen} --locked`,
  );
}

run("cargo", ["build", "-p", "gallery-web", "--target", "wasm32-unknown-unknown", "--profile", "wasm"]);

rmSync(out, { recursive: true, force: true });
run("wasm-bindgen", [
  "--target",
  "web",
  "--no-typescript",
  "--out-dir",
  out,
  join(root, "target/wasm32-unknown-unknown/wasm/gallery_web.wasm"),
]);

const wasm = join(out, "gallery_web_bg.wasm");
const before = statSync(wasm).size;
const require = createRequire(import.meta.url);
const wasmOpt = join(dirname(require.resolve("binaryen/package.json")), "bin/wasm-opt");
execFileSync(
  process.execPath,
  [
    wasmOpt,
    "-O2",
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
    "--enable-sign-ext",
    "--enable-reference-types",
    "--enable-multivalue",
    "--enable-mutable-globals",
    wasm,
    "-o",
    wasm,
  ],
  { stdio: "inherit" },
);

const mb = (bytes) => `${(bytes / 1024 / 1024).toFixed(1)} MB`;
console.log(`gallery_web_bg.wasm: ${mb(before)} → ${mb(statSync(wasm).size)}`);
