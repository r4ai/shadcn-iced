// The select of the theme customizer (`components/ui/select.tsx`), which
// behaves like the Base UI select of shadcn/ui.
import { expect, reload, test, visit } from "./fixtures";

test.beforeEach(async ({ page }) => {
  await visit(page, "/docs/theming/");
});

const center = (box: { y: number; height: number }) => box.y + box.height / 2;

const themeSelect = (page: import("@playwright/test").Page) =>
  page.getByRole("main").getByRole("button", { name: /^Theme/ });

test("opens over the trigger with the selected item in place of the value", async ({ page }) => {
  const trigger = themeSelect(page);
  await expect(trigger).toHaveText("Same as base");
  await trigger.click();

  const listbox = page.getByRole("listbox");
  await expect(listbox).toBeVisible();

  const selected = listbox.getByRole("option", { name: "Same as base" });
  await expect(selected).toHaveAttribute("aria-selected", "true");

  const triggerBox = (await trigger.boundingBox())!;
  const itemBox = (await selected.boundingBox())!;
  expect(Math.abs(center(itemBox) - center(triggerBox))).toBeLessThanOrEqual(1);
  // The text of the item lines up with the value of the trigger.
  expect(Math.abs(itemBox.x + 2 - (triggerBox.x + 4))).toBeLessThanOrEqual(4);

  // The list is shortened to stay in the viewport.
  const listBox = (await listbox.boundingBox())!;
  expect(listBox.y + listBox.height).toBeLessThanOrEqual(page.viewportSize()!.height);
  // Without the open/close animation of an item-aligned list.
  await expect(page.locator("[data-slot=select-content]")).toHaveCSS("animation-name", "none");
});

test("an item clicked right after opening is picked", async ({ page }) => {
  const trigger = themeSelect(page);
  await trigger.click();
  await page.getByRole("option", { name: "Violet" }).click();
  await expect(trigger).toHaveText("Violet");
});

test("the click that opens it does not pick an item", async ({ page }) => {
  const trigger = themeSelect(page);
  await trigger.click();
  await page.waitForTimeout(500);

  await expect(page.getByRole("listbox")).toBeVisible();
  await expect(trigger).toHaveText("Same as base");
});

test("a press released on another item right away does not pick it", async ({ page }) => {
  const trigger = themeSelect(page);
  const box = (await trigger.boundingBox())!;
  // Press, move onto the next item and release at once, as a quick click
  // that slips: the release comes before the delay of Base UI (200ms).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 28);
  await page.mouse.up();

  await expect(page.getByRole("listbox")).toBeVisible();
  await expect(trigger).toHaveText("Same as base");
});

test("a press dragged onto an item and released later picks it", async ({ page }) => {
  const trigger = themeSelect(page);
  const box = (await trigger.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 28, { steps: 4 });
  await page.waitForTimeout(450);
  await page.mouse.up();

  await expect(page.getByRole("listbox")).toBeHidden();
  await expect(trigger).toHaveText("Amber");
});

test("picking an item updates the previews' theme, its code and the stored theme", async ({ page }) => {
  const trigger = themeSelect(page);
  await trigger.click();
  await page.getByRole("option", { name: "Blue" }).click();

  await expect(page.getByRole("listbox")).toBeHidden();
  await expect(trigger).toHaveText("Blue");
  await expect(trigger).toBeFocused();
  await expect(page.locator("[data-rehype-pretty-code-figure] pre").first()).toContainText("Accent::Blue");

  const stored = await page.evaluate(() => localStorage.getItem("shadcn-iced:theme"));
  expect(JSON.parse(stored!)).toMatchObject({ accent: "blue" });

  await reload(page);
  await expect(themeSelect(page)).toHaveText("Blue");
});

test("works with the keyboard", async ({ page }) => {
  const trigger = themeSelect(page);
  await trigger.focus();
  await page.keyboard.press("Enter");

  const listbox = page.getByRole("listbox");
  await expect(listbox).toBeVisible();
  await expect(listbox.getByRole("option", { name: "Same as base" })).toHaveAttribute("data-highlighted");

  await page.keyboard.press("ArrowDown");
  await expect(listbox.getByRole("option", { name: "Amber" })).toHaveAttribute("data-highlighted");
  await page.keyboard.press("Enter");

  await expect(listbox).toBeHidden();
  await expect(trigger).toHaveText("Amber");

  await page.keyboard.press("Enter");
  await expect(listbox).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(listbox).toBeHidden();
  await expect(trigger).toBeFocused();
  await expect(trigger).toHaveText("Amber");
});

test("closes when clicking outside", async ({ page }) => {
  await themeSelect(page).click();
  await expect(page.getByRole("listbox")).toBeVisible();
  await page.mouse.click(5, 300);
  await expect(page.getByRole("listbox")).toBeHidden();
});
