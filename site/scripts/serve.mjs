// Serves the built site (`dist/client`) like Cloudflare Workers static
// assets do with `wrangler.jsonc`: `auto-trailing-slash` for the pages, and
// `404.html` for unknown paths. For the end-to-end tests and local checks:
//
//   node scripts/serve.mjs [port] [dist]
//
// `dist` is another build to serve than `dist/` (e.g. the one of `main`, for
// `scripts/lighthouse.mjs`).
import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = process.argv[3]
  ? resolve(process.argv[3], "client")
  : fileURLToPath(new URL("../dist/client", import.meta.url));
const port = Number(process.argv[2] ?? process.env.PORT ?? 4322);

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".txt": "text/plain; charset=utf-8",
};

const isFile = (path) => {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
};

function send(response, status, path) {
  response.writeHead(status, { "content-type": TYPES[extname(path)] ?? "application/octet-stream" });
  createReadStream(path).pipe(response);
}

createServer((request, response) => {
  const { pathname, search } = new URL(request.url ?? "/", "http://localhost");
  const path = normalize(join(root, decodeURIComponent(pathname)));
  if (!path.startsWith(root)) {
    response.writeHead(403).end();
    return;
  }

  if (isFile(path)) {
    send(response, 200, path);
  } else if (isFile(join(path, "index.html"))) {
    if (pathname.endsWith("/")) {
      send(response, 200, join(path, "index.html"));
    } else {
      response.writeHead(307, { location: `${pathname}/${search}` }).end();
    }
  } else if (isFile(`${path}.html`)) {
    send(response, 200, `${path}.html`);
  } else {
    send(response, 404, join(root, "404.html"));
  }
}).listen(port, () => console.log(`http://localhost:${port}`));
