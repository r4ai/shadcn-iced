// The test of the site, whose pages are prerendered: `page.goto` resolves
// before they are hydrated, so the tests go through `visit`.
import { test as base, expect, type Page } from "@playwright/test";

/** Opens `path` and waits until the page is hydrated. */
export async function visit(page: Page, path: string) {
  const response = await page.goto(path);
  await expect(page.locator("html")).toHaveAttribute("data-hydrated", "true");
  return response;
}

/** Reloads the page and waits until it is hydrated again. */
export async function reload(page: Page) {
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-hydrated", "true");
}

export const test = base;
export { expect };
