import { defineConfig, devices } from "@playwright/test"

const port = 4517

export default defineConfig({
  testDir: "tests",
  outputDir: "out/playwright",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? [["github"], ["list"]] : "list",
  use: {
    ...devices["Desktop Chrome"],
    baseURL: `http://localhost:${port}`,
    // iced snapshots are rendered at a scale factor of 2.
    deviceScaleFactor: 2,
  },
  webServer: {
    command: `pnpm exec vite --port ${port} --strictPort`,
    url: `http://localhost:${port}`,
    reuseExistingServer: !process.env.CI,
  },
})
