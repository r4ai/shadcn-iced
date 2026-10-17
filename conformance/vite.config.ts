import { resolve } from "node:path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig, searchForWorkspaceRoot } from "vite"

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": resolve(import.meta.dirname, "./src"),
    },
  },
  server: {
    fs: {
      // The symbol fonts of `index.css` are those of shadcn_iced.
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        resolve(import.meta.dirname, "../crates/shadcn_iced_core/assets/fonts"),
      ],
    },
  },
})
