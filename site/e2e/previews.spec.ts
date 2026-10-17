// The live previews: iced, compiled to WebAssembly, mounts each example in a
// canvas, and closes it when the page goes away.
import type { Page } from "@playwright/test";

import { expect, test, visit } from "./fixtures";

const frames = (page: Page) => page.locator("[data-preview]");
const failed = (page: Page) => page.getByText("The live preview could not start in this browser.");

/**
 * Waits until the first preview runs. Headless Chromium has no WebGPU
 * adapter: the previews fall back to the software renderer of iced.
 */
async function waitForFirstPreview(page: Page) {
  const canvas = frames(page).first().locator("canvas");
  await expect(canvas).toBeVisible({ timeout: 30_000 });
  await expect(failed(page)).toHaveCount(0);
  return canvas;
}

test("a preview mounts its example", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  const canvas = await waitForFirstPreview(page);
  const box = (await canvas.boundingBox())!;
  expect(box.height).toBeGreaterThanOrEqual(320 - 2);
});

test("navigating away closes the previews of the page", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  await waitForFirstPreview(page);

  await page.locator("[data-slot=sidebar]").getByRole("link", { name: "Badge", exact: true }).click();
  await expect(page).toHaveTitle("Badge - shadcn-iced");
  await expect(frames(page).first().locator("canvas")).toBeVisible({ timeout: 30_000 });

  // Every canvas belongs to a preview of the current page.
  const canvases = await page.locator("canvas").count();
  const mounted = await frames(page)
    .filter({ has: page.locator("canvas") })
    .count();
  expect(canvases).toBe(mounted);
});

test("the previews follow the theme of the customizer", async ({ page }) => {
  await visit(page, "/docs/theming/");
  await waitForFirstPreview(page);
  const code = page.locator("[data-rehype-pretty-code-figure] pre").first();
  await expect(code).toContainText("BaseColor::Neutral");

  await page
    .getByRole("main")
    .getByRole("button", { name: /^Base color/ })
    .click();
  await page.getByRole("option", { name: "Zinc" }).click();
  await expect(code).toContainText("BaseColor::Zinc");
});

test("a preview grows to fit an example taller than it starts", async ({ page }) => {
  await visit(page, "/docs/components/table/");
  await waitForFirstPreview(page);
  await expect.poll(async () => (await frames(page).first().boundingBox())!.height).toBeGreaterThan(400);
});

test("the wheel scrolls the page over a preview", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  const canvas = await waitForFirstPreview(page);
  const box = (await canvas.boundingBox())!;

  // Clear of the button in the middle of the example.
  await page.mouse.move(box.x + 20, box.y + 20);
  await page.mouse.wheel(0, 200);
  await expect.poll(() => page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
});

test("the wheel scrolls a scroll area of a preview, not the page", async ({ page }) => {
  await visit(page, "/docs/components/scroll-area/");
  const canvas = await waitForFirstPreview(page);
  const box = (await canvas.boundingBox())!;

  // The scroll area is in the middle of the example.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, 200);
  await page.waitForTimeout(500);
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});
