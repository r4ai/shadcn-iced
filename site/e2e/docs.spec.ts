// The pages of the docs: the sidebar, the table of contents, the pager, the
// code blocks and the installation tabs.
import { expect, reload, test, visit } from "./fixtures";

test.use({ permissions: ["clipboard-read", "clipboard-write"] });

// The clipboard of the system is shared by the tests that run at the same
// time: the page records what it copies instead.
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const copied: string[] = [];
    Object.assign(window, { copied });
    navigator.clipboard.writeText = async (text: string) => {
      copied.push(text);
    };
  });
});

const lastCopied = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as unknown as { copied: string[] }).copied.at(-1));

test("the sidebar marks the current page and its section", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  const sidebar = page.locator("[data-slot=sidebar]");
  await expect(sidebar.getByRole("link", { name: "Button", exact: true })).toHaveAttribute("data-active", "true");
  await expect(sidebar.getByRole("link", { name: "Components", exact: true })).toHaveAttribute("data-active", "true");
  await expect(sidebar.getByRole("link", { name: "Introduction" })).toHaveAttribute("data-active", "false");

  await sidebar.getByRole("link", { name: "Badge", exact: true }).click();
  await expect(page).toHaveTitle("Badge - shadcn-iced");
  await expect(sidebar.getByRole("link", { name: "Badge", exact: true })).toHaveAttribute("data-active", "true");
  await expect(sidebar.getByRole("link", { name: "Button", exact: true })).toHaveAttribute("data-active", "false");
});

test("the pager goes to the previous and next pages", async ({ page }) => {
  await visit(page, "/docs/installation/");
  await page.getByRole("link", { name: "Next" }).click();
  await expect(page).toHaveTitle("Theming - shadcn-iced");
  await page.getByRole("link", { name: "Previous" }).click();
  await expect(page).toHaveTitle("Installation - shadcn-iced");
  await page.getByRole("link", { name: "Introduction" }).last().click();
  await expect(page).toHaveTitle("Introduction - shadcn-iced");
});

test("the table of contents follows the heading in view", async ({ page }) => {
  await visit(page, "/docs/installation/");
  const toc = page.getByText("On This Page").locator("..");
  const link = toc.getByRole("link", { name: "Copy and paste" });
  await expect(link).toHaveAttribute("data-active", "false");

  await page.evaluate(() => document.getElementById("copy-and-paste")!.scrollIntoView());
  await expect(link).toHaveAttribute("data-active", "true");
  await expect(toc.getByRole("link", { name: "As a library" })).toHaveAttribute("data-active", "false");

  await toc.getByRole("link", { name: "As a library" }).click();
  await expect(page).toHaveURL(/#as-a-library$/);
  await expect(toc.getByRole("link", { name: "As a library" })).toHaveAttribute("data-active", "true");
});

test("headings link to themselves", async ({ page }) => {
  await visit(page, "/docs/installation/");
  const anchor = page.getByRole("heading", { name: "Copy and paste" }).getByRole("link");
  await expect(anchor).toHaveAttribute("href", "#copy-and-paste");
});

test("the copy button copies the code and shows a check for a moment", async ({ page }) => {
  await visit(page, "/docs/installation/");
  const figure = page.locator("[data-rehype-pretty-code-figure]").filter({ hasText: "Cargo.toml" });
  const button = figure.getByRole("button", { name: "Copy" });
  await button.click();

  await expect(button).toHaveAttribute("data-copied", "true");
  expect(await lastCopied(page)).toContain('shadcn_iced = { git = "');
  await expect(button).toHaveAttribute("data-copied", "false", { timeout: 3000 });
});

test("a shell command shows in a terminal", async ({ page }) => {
  await visit(page, "/docs/installation/");
  const figure = page.locator("[data-rehype-pretty-code-figure]").filter({ hasText: "cargo install" });
  await expect(figure.getByText("cargo", { exact: true }).first()).toBeVisible();
  await figure.getByRole("button", { name: "Copy" }).click();
  expect(await lastCopied(page)).toBe("cargo install --git https://github.com/r4ai/shadcn-iced cargo-shadcn-iced");
});

test("the installation tab is remembered", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  await page.getByRole("tab", { name: "Library" }).click();
  await page.locator("[data-slot=sidebar]").getByRole("link", { name: "Badge", exact: true }).click();
  await expect(page).toHaveTitle("Badge - shadcn-iced");
  await expect(page.getByRole("tab", { name: "Library" })).toHaveAttribute("aria-selected", "true");

  await reload(page);
  await expect(page.getByRole("tab", { name: "Library" })).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel")).toContainText("use shadcn_iced::badge::*;");
});

test("the sidebar shows the current page and keeps its scroll", async ({ page }) => {
  await visit(page, "/docs/components/toggle-group/");
  const content = page.locator("[data-docs-sidebar-content]");
  await expect(content.getByRole("link", { name: "Toggle Group", exact: true })).toBeInViewport();

  await content.evaluate((element) => element.scrollTo({ top: 120 }));
  await page.waitForTimeout(100);
  await reload(page);
  await expect.poll(() => content.evaluate((element) => element.scrollTop)).toBe(120);
});

test("code blocks in prose keep the size of code blocks", async ({ page }) => {
  await visit(page, "/docs/installation/");
  const pre = page.locator("[data-rehype-pretty-code-figure] pre").first();
  await expect(pre).toHaveCSS("font-size", "14px");
  await expect(pre).toHaveCSS("margin-top", "0px");
});

test("the installation tabs switch between the ways to install", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  const tabs = page.getByRole("tablist");
  const command = tabs.getByRole("tab", { name: "Command" });
  const library = tabs.getByRole("tab", { name: "Library" });
  await expect(command).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel")).toContainText("cargo shadcn-iced add button");

  await library.click();
  await expect(library).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel")).toContainText("use shadcn_iced::button::*;");

  // Arrow keys move between the tabs.
  await page.keyboard.press("ArrowRight");
  const manual = tabs.getByRole("tab", { name: "Manual" });
  await expect(manual).toBeFocused();
  await expect(manual).toHaveAttribute("aria-selected", "true");
});

test("the source of a component expands and collapses", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  await page.getByRole("tab", { name: "Manual" }).click();
  const panel = page.getByRole("tabpanel");
  const collapsible = panel.locator("[data-state]").first();
  await expect(collapsible).toHaveAttribute("data-state", "closed");

  const code = collapsible.locator("[data-slot=code-collapsible-content]");
  const collapsedHeight = (await code.boundingBox())!.height;
  expect(collapsedHeight).toBeLessThanOrEqual(256 + 1);

  await collapsible.getByRole("button", { name: "Expand" }).first().click();
  await expect(collapsible).toHaveAttribute("data-state", "open");
  expect((await code.boundingBox())!.height).toBeGreaterThan(collapsedHeight);

  await collapsible.getByRole("button", { name: "Collapse" }).click();
  await expect(collapsible).toHaveAttribute("data-state", "closed");
});

test("the code of an example shows its first lines until it is shown", async ({ page }) => {
  await visit(page, "/docs/components/button/");
  const preview = page.locator("[data-slot=component-preview]").first();
  const code = preview.locator("[data-slot=code]");
  await expect(code.locator("[data-line]")).toHaveCount(3);

  await code.getByRole("button", { name: "View Code" }).click();
  await expect(code.getByRole("button", { name: "View Code" })).toBeHidden();
  expect(await code.locator("[data-line]").count()).toBeGreaterThan(3);
  await expect(code.locator("[data-rehype-pretty-code-title]")).toHaveText(/button\.rs:\d+/);
  await expect(code.getByRole("link", { name: "View on GitHub" })).toHaveAttribute(
    "href",
    /github\.com\/r4ai\/shadcn-iced\/blob\/main\/examples\/gallery\/.+#L\d+$/,
  );
});

test("the components list links to every component", async ({ page }) => {
  await visit(page, "/docs/components/");
  const links = page.getByRole("main").locator('a[href^="/docs/components/"]');
  expect(await links.count()).toBeGreaterThan(60);
  await page
    .locator("[data-slot=docs] [data-not-typeset]")
    .getByRole("link", { name: "Accordion", exact: true })
    .click();
  await expect(page).toHaveTitle("Accordion - shadcn-iced");
});

test("an unknown page is a 404 page", async ({ page }) => {
  const response = await visit(page, "/docs/nope");
  expect(response?.status()).toBe(404);
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
  await page.getByRole("link", { name: "Browse Components" }).click();
  await expect(page).toHaveTitle("Components - shadcn-iced");
});
