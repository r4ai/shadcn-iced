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
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(site, "..");
const out = join(site, "public/wasm");
const target = resolve(root, process.env.CARGO_TARGET_DIR ?? "target");
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

// Panic locations survive wasm-opt: remap build paths before compiling.
// Cargo splits RUSTFLAGS on whitespace; encoded flags also preserve paths with spaces.
const rustflags =
  process.env.CARGO_ENCODED_RUSTFLAGS !== undefined
    ? process.env.CARGO_ENCODED_RUSTFLAGS.split("\x1f").filter(Boolean)
    : (process.env.RUSTFLAGS ?? "").split(/\s+/).filter(Boolean);
const prefixes = [
  [homedir(), "/home/build"],
  [resolve(root, process.env.CARGO_HOME ?? join(homedir(), ".cargo")), "/cargo"],
  [resolve(root, process.env.RUSTUP_HOME ?? join(homedir(), ".rustup")), "/rustup"],
  [root, "/workspace"],
];
execFileSync("cargo", ["build", "-p", "gallery-web", "--target", "wasm32-unknown-unknown", "--profile", "wasm"], {
  cwd: root,
  stdio: "inherit",
  env: {
    ...process.env,
    CARGO_ENCODED_RUSTFLAGS: [...rustflags, ...prefixes.map(([from, to]) => `--remap-path-prefix=${from}=${to}`)].join(
      "\x1f",
    ),
  },
});

rmSync(out, { recursive: true, force: true });
run("wasm-bindgen", [
  "--target",
  "web",
  "--no-typescript",
  "--out-dir",
  out,
  join(target, "wasm32-unknown-unknown/wasm/gallery_web.wasm"),
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

// Reject any build-machine prefixes that escaped remapping.
const binary = readFileSync(wasm);
for (const [prefix] of prefixes) {
  if ([prefix, prefix.replaceAll("\\", "/")].some((path) => binary.includes(Buffer.from(path)))) {
    throw new Error("The preview WASM still contains a build-machine path.");
  }
}

const mb = (bytes) => `${(bytes / 1024 / 1024).toFixed(1)} MB`;
console.log(`gallery_web_bg.wasm: ${mb(before)} → ${mb(statSync(wasm).size)}`);
