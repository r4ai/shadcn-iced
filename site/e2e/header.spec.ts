// The header: navigation, the mode switcher and the theme menu.
import { expect, reload, test, visit } from "./fixtures";

test("the mode switcher toggles dark mode and remembers it", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await visit(page, "/");
  const html = page.locator("html");
  await expect(html).not.toHaveClass(/\bdark\b/);

  await page.getByRole("button", { name: "Toggle theme" }).click();
  await expect(html).toHaveClass(/\bdark\b/);
  expect(await page.evaluate(() => localStorage.getItem("shadcn-iced:mode"))).toBe("dark");

  // Set before the first paint on the next load.
  await reload(page);
  await expect(html).toHaveClass(/\bdark\b/);

  await page.getByRole("button", { name: "Toggle theme" }).click();
  await expect(html).not.toHaveClass(/\bdark\b/);
});

test("the D key toggles dark mode, and the theme color follows", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await visit(page, "/");
  const html = page.locator("html");
  const meta = page.locator("meta[name=theme-color]");
  await expect(meta).toHaveAttribute("content", "#ffffff");

  await page.keyboard.press("d");
  await expect(html).toHaveClass(/\bdark\b/);
  await expect(meta).toHaveAttribute("content", "#0a0a0a");

  // Not with a modifier.
  await page.keyboard.press("Alt+d");
  await expect(html).toHaveClass(/\bdark\b/);

  await page.keyboard.press("D");
  await expect(html).not.toHaveClass(/\bdark\b/);

  await reload(page);
  await expect(meta).toHaveAttribute("content", "#ffffff");
});

test("follows the system until a mode is picked", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await visit(page, "/");
  await expect(page.locator("html")).toHaveClass(/\bdark\b/);

  await page.emulateMedia({ colorScheme: "light" });
  await expect(page.locator("html")).not.toHaveClass(/\bdark\b/);
});

test("the main navigation goes to its pages without reloading", async ({ page }) => {
  await visit(page, "/");
  await page.evaluate(() => ((window as unknown as { marker: boolean }).marker = true));

  const nav = page.getByRole("navigation").first();
  await nav.getByRole("link", { name: "Components" }).click();
  await expect(page).toHaveURL(/\/docs\/components\/?$/);
  await expect(page).toHaveTitle("Components - shadcn-iced");
  await expect(nav.getByRole("link", { name: "Components" })).toHaveAttribute("data-active", "true");
  await expect(nav.getByRole("link", { name: "Home" })).toHaveAttribute("data-active", "false");

  await nav.getByRole("link", { name: "Theming" }).click();
  await expect(page).toHaveTitle("Theming - shadcn-iced");
  expect(await page.evaluate(() => (window as unknown as { marker?: boolean }).marker)).toBe(true);
});

test("the theme menu opens, resets the theme and closes", async ({ page }) => {
  await visit(page, "/docs/theming/");
  const main = page.getByRole("main");
  await main.getByRole("button", { name: /^Radius/ }).click();
  await page.getByRole("option", { name: "Large" }).click();
  await expect(main.getByRole("button", { name: /^Radius/ })).toHaveText("Large");

  const menuButton = page.getByRole("banner").getByRole("button", { name: "Theme", exact: true });
  await menuButton.click();
  const dialog = page.getByRole("dialog", { name: "Preview theme" });
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute("data-side", "bottom");
  await expect(menuButton).toHaveAttribute("aria-expanded", "true");
  // The menu and the customizer show the same theme.
  await expect(dialog.getByRole("button", { name: /^Radius/ })).toHaveText("Large");

  await dialog.getByRole("button", { name: "Reset" }).click();
  await expect(dialog.getByRole("button", { name: /^Radius/ })).toHaveText("Default");
  await expect(main.getByRole("button", { name: /^Radius/ })).toHaveText("Default");

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(menuButton).toBeFocused();
});

test("the GitHub link opens the repository in a new tab", async ({ page }) => {
  await visit(page, "/");
  const link = page.getByRole("link", { name: "GitHub" }).first();
  await expect(link).toHaveAttribute("href", "https://github.com/r4ai/shadcn-iced");
  await expect(link).toHaveAttribute("target", "_blank");
});
