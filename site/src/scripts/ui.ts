// The behavior of the static parts of the site: tabs, copy buttons, the
// mode toggle and the theme menu.
import { currentTheme, isDark, onThemeChange, setDark, setTheme } from "./theme";

// Tabs: `[data-tabs]` with `[data-tab=name]` triggers and
// `[data-panel=name]` panels.
document.addEventListener("click", (event) => {
  const trigger = (event.target as Element).closest<HTMLElement>("[data-tab]");
  const tabs = trigger?.closest<HTMLElement>("[data-tabs]");
  if (!trigger || !tabs) return;

  const name = trigger.dataset.tab;
  for (const tab of tabs.querySelectorAll<HTMLElement>(":scope [data-tab]")) {
    if (tab.closest("[data-tabs]") !== tabs) continue;
    tab.setAttribute("aria-selected", String(tab.dataset.tab === name));
  }
  for (const panel of tabs.querySelectorAll<HTMLElement>(":scope [data-panel]")) {
    if (panel.closest("[data-tabs]") !== tabs) continue;
    panel.dataset.active = String(panel.dataset.panel === name);
  }
});

// Copy buttons: copy the text of `[data-copy-source]` in the same
// `[data-copy-scope]`.
document.addEventListener("click", async (event) => {
  const button = (event.target as Element).closest<HTMLElement>("[data-copy]");
  if (!button) return;

  const scope = button.closest("[data-copy-scope]");
  const source = scope?.querySelector<HTMLElement>("[data-copy-source]");
  const text = button.dataset.copy || source?.innerText || "";

  try {
    await navigator.clipboard.writeText(text);
    button.dataset.copied = "true";
    setTimeout(() => delete button.dataset.copied, 1600);
  } catch {
    if (source) window.getSelection()?.selectAllChildren(source);
  }
});

// Expandable code blocks.
document.addEventListener("click", (event) => {
  const button = (event.target as Element).closest<HTMLElement>("[data-expand]");
  const block = button?.closest<HTMLElement>("[data-expandable]");
  if (!button || !block) return;

  block.dataset.expanded = String(block.dataset.expanded !== "true");
});

// Mode toggle.
for (const toggle of document.querySelectorAll<HTMLElement>("[data-mode-toggle]")) {
  toggle.addEventListener("click", () => setDark(!isDark()));
}

// Theme menu: `select[data-theme-option=base|accent|radius]`.
function syncThemeControls() {
  const theme = currentTheme();
  for (const select of document.querySelectorAll<HTMLSelectElement>("[data-theme-option]")) {
    select.value = theme[select.dataset.themeOption as keyof typeof theme];
  }
}

document.addEventListener("change", (event) => {
  const select = (event.target as Element).closest<HTMLSelectElement>("[data-theme-option]");
  if (!select) return;

  setTheme({ ...currentTheme(), [select.dataset.themeOption!]: select.value });
});

document.addEventListener("click", (event) => {
  if ((event.target as Element).closest("[data-theme-reset]")) {
    setTheme({ base: "neutral", accent: "", radius: "default" });
  }
});

syncThemeControls();
onThemeChange(syncThemeControls);

// Popovers: `[data-popover]` with a `[data-popover-trigger]` button.
document.addEventListener("click", (event) => {
  const target = event.target as Element;
  const trigger = target.closest<HTMLElement>("[data-popover-trigger]");

  for (const popover of document.querySelectorAll<HTMLElement>("[data-popover]")) {
    const inside = popover.contains(target);
    if (trigger && popover.contains(trigger)) {
      popover.dataset.open = String(popover.dataset.open !== "true");
    } else if (!inside) {
      popover.dataset.open = "false";
    }
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return;
  for (const popover of document.querySelectorAll<HTMLElement>("[data-popover]")) {
    popover.dataset.open = "false";
  }
});
