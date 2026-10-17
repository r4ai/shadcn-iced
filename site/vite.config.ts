import { fileURLToPath } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/solid-start/plugin/vite";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

import { siteData } from "./plugins/site-data.ts";

// Deployed to Cloudflare Workers (static assets), at the root of its domain:
// every page is prerendered to `dist/client`.
const base = process.env.BASE ?? "/";

export default defineConfig({
  base,
  resolve: {
    alias: { "~": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  plugins: [
    siteData(),
    tailwindcss(),
    tanstackStart({
      srcDirectory: "src",
      router: {
        basepath: base,
        // Files and folders starting with `-` (e.g. `-components/`) live
        // next to the pages that use them and are not routes.
        routesDirectory: "pages",
      },
      prerender: {
        enabled: true,
        crawlLinks: true,
        autoSubfolderIndex: true,
      },
      pages: [{ path: "/404", prerender: { enabled: true, outputPath: "/404.html" } }],
    }),
    solid({ ssr: true }),
  ],
});
