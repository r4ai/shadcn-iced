// Small screens: the menu of the header replaces the navigation and the
// sidebar.
import { expect, test, visit } from "./fixtures";

test("the menu lists the pages and closes once one is picked", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  await expect(page.locator("[data-slot=sidebar]")).toBeHidden();

  const toggle = page.getByRole("button", { name: /Toggle Menu/ });
  await toggle.click();
  const menu = page.getByRole("dialog");
  await expect(menu).toBeVisible();
  await expect(menu.getByText("Sections")).toBeVisible();
  await expect(menu.getByRole("link", { name: "Accordion" })).toBeVisible();

  // It covers the page under the header.
  const box = (await menu.boundingBox())!;
  const viewport = page.viewportSize()!;
  expect(Math.round(box.width)).toBe(viewport.width);
  expect(Math.round(box.y + box.height)).toBe(viewport.height);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);

  await menu.getByRole("link", { name: "Installation" }).click();
  await expect(page).toHaveTitle("Installation - shadcn-iced");
  await expect(menu).toBeHidden();
});

test("the menu closes with Escape", async ({ page }) => {
  await visit(page, "/");
  const toggle = page.getByRole("button", { name: /Toggle Menu/ });
  await toggle.click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(toggle).toBeFocused();
});

test("no page scrolls sideways", async ({ page }) => {
  // One page after the other, in the same tab.
  // oxlint-disable no-await-in-loop
  for (const path of [
    "/",
    "/docs/",
    "/docs/installation/",
    "/docs/theming/",
    "/docs/components/",
    "/docs/components/chart/",
  ]) {
    await visit(page, path);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow, path).toBeLessThanOrEqual(0);
  }
});
