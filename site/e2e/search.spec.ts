// The search of the header (the command menu of ui.shadcn.com).
import { expect, test, visit } from "./fixtures";

test("the button opens the search, which goes to a component", async ({ page }) => {
  await visit(page, "/");
  await page.getByRole("button", { name: "Search documentation..." }).click();

  const dialog = page.getByRole("dialog");
  const input = dialog.getByRole("combobox");
  await expect(input).toBeFocused();
  await expect(dialog.getByRole("option", { name: "Docs", exact: true })).toBeVisible();

  await input.fill("dropdown");
  const option = dialog.getByRole("option", { name: "Dropdown Menu", exact: true });
  await expect(option).toHaveAttribute("aria-selected", "true");
  await expect(dialog).toContainText("cargo shadcn-iced add dropdown-menu");

  await input.press("Enter");
  await expect(page).toHaveURL(/\/docs\/components\/dropdown-menu\/?$/);
  await expect(dialog).toBeHidden();
});

test("⌘K and / toggle the search, and Escape closes it", async ({ page }) => {
  await visit(page, "/docs");
  const dialog = page.getByRole("dialog");

  await page.keyboard.press("ControlOrMeta+k");
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();

  await page.keyboard.press("/");
  await expect(dialog).toBeVisible();
  // Typed in the input, / does not close it.
  await page.keyboard.type("/");
  await expect(dialog.getByRole("combobox")).toHaveValue("/");
});

test("the arrows move the selection, and a result goes to its heading", async ({ page }) => {
  await visit(page, "/");
  await page.keyboard.press("ControlOrMeta+k");
  const dialog = page.getByRole("dialog");
  const input = dialog.getByRole("combobox");

  await input.fill("copy and paste");
  await expect(dialog.getByText("Search Results")).toBeVisible();
  const result = dialog.getByRole("option", { name: /Copy and paste/ });
  await expect(result).toHaveAttribute("aria-selected", "true");

  await input.fill("zzzz");
  await expect(dialog.getByText("No results found.")).toBeVisible();

  await input.fill("chart");
  const options = dialog.getByRole("option");
  await expect(options.first()).toHaveAttribute("aria-selected", "true");
  await input.press("ArrowDown");
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true");
  await input.press("ArrowUp");
  await input.press("ArrowUp");
  await expect(options.last()).toHaveAttribute("aria-selected", "true");

  await input.fill("copy and paste");
  await input.press("Enter");
  await expect(page).toHaveURL(/\/docs\/installation\/?#copy-and-paste$/);
});

test("typing in the search does not toggle dark mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await visit(page, "/");
  await page.keyboard.press("ControlOrMeta+k");
  await page.keyboard.type("dialog");
  await expect(page.locator("html")).not.toHaveClass(/\bdark\b/);
});
