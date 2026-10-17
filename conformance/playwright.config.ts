import { defineConfig, devices } from "@playwright/test"

// `CONFORMANCE_PORT` lets several checkouts run the suite at the same time.
const port = Number(process.env.CONFORMANCE_PORT ?? 4517)

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
    // The same environment everywhere: the page must not depend on the
    // machine running it (see "Determinism" in README.md).
    locale: "en-US",
    timezoneId: "UTC",
    colorScheme: "light",
    reducedMotion: "reduce",
    launchOptions: {
      args: [
        // Rasterize text the same way on every operating system: no
        // hinting, no subpixel (LCD) anti-aliasing, like tiny-skia.
        "--font-render-hinting=none",
        "--disable-lcd-text",
        // Software rendering and sRGB output, whatever the GPU and display.
        "--disable-gpu",
        "--force-color-profile=srgb",
        // Native scrollbars differ between operating systems.
        "--hide-scrollbars",
      ],
    },
  },
  webServer: {
    command: `pnpm exec vite --port ${port} --strictPort`,
    url: `http://localhost:${port}`,
    reuseExistingServer: !process.env.CI,
  },
})
