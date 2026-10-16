// @ts-check
import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

// Deployed to Cloudflare Workers (static assets), at the root of its domain.
export default defineConfig({
  base: process.env.BASE ?? "/",
  trailingSlash: "ignore",
  vite: {
    plugins: [tailwindcss()],
  },
});
