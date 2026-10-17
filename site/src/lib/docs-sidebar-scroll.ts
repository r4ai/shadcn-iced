// The scroll of the docs sidebar, after lib/docs-sidebar-scroll.ts and
// components/docs-sidebar.tsx of ui.shadcn.com: it comes back on the same
// page, and otherwise the active item is scrolled to the middle when it is
// out of view. A script runs it before the first paint; the sidebar then
// runs it on each page.

export const DOCS_SIDEBAR_SCROLL_STORAGE_KEY = "shadcn-iced:docs-sidebar-scroll";

interface ScrollState {
  pathname: string;
  scrollTop: number;
}

/** The active item: the longest route wins, as section links match by prefix. */
function getActiveItem(container: HTMLElement) {
  const items = container.querySelectorAll<HTMLElement>('[data-active="true"]');
  const center = container.getBoundingClientRect().top + container.clientHeight / 2;
  let active: HTMLElement | undefined;
  let activeLength = -1;
  let activeDistance = Infinity;

  for (const item of items) {
    const length = item.getAttribute("href")?.length ?? 0;
    const rect = item.getBoundingClientRect();
    const distance = Math.abs(rect.top + rect.height / 2 - center);
    if (length > activeLength || (length === activeLength && distance < activeDistance)) {
      active = item;
      activeLength = length;
      activeDistance = distance;
    }
  }

  return active;
}

function readScrollState(): ScrollState | null {
  try {
    return JSON.parse(sessionStorage.getItem(DOCS_SIDEBAR_SCROLL_STORAGE_KEY) ?? "");
  } catch {
    return null;
  }
}

export function saveScrollState(container: HTMLElement) {
  try {
    sessionStorage.setItem(
      DOCS_SIDEBAR_SCROLL_STORAGE_KEY,
      JSON.stringify({ pathname: location.pathname, scrollTop: container.scrollTop } satisfies ScrollState),
    );
  } catch {
    // The scroll then starts over on each page.
  }
}

/** Restores the scroll of the sidebar, or brings its active item into view. */
export function restoreScroll(container: HTMLElement) {
  if (!container.clientHeight) return;

  const stored = readScrollState();
  if (stored?.pathname === location.pathname) {
    container.scrollTop = stored.scrollTop;
    return;
  }

  const active = getActiveItem(container);
  if (!active) return;

  const containerRect = container.getBoundingClientRect();
  const activeRect = active.getBoundingClientRect();
  if (activeRect.top >= containerRect.top && activeRect.bottom <= containerRect.bottom) return;

  container.scrollTop += activeRect.top - containerRect.top - (container.clientHeight - activeRect.height) / 2;
}

/**
 * The script of `<head>` that restores the scroll before the first paint, as
 * the prerendered sidebar is parsed: the functions above, in plain JavaScript.
 */
export const DOCS_SIDEBAR_SCROLL_RESTORE_SCRIPT = `(function () {
  var stored = null;
  try {
    stored = JSON.parse(sessionStorage.getItem(${JSON.stringify(DOCS_SIDEBAR_SCROLL_STORAGE_KEY)}));
  } catch (e) {}

  function getActiveItem(container) {
    var items = container.querySelectorAll('[data-active="true"]');
    var center = container.getBoundingClientRect().top + container.clientHeight / 2;
    var active = null, activeLength = -1, activeDistance = Infinity;
    for (var i = 0; i < items.length; i++) {
      var href = items[i].getAttribute("href");
      var length = href ? href.length : 0;
      var rect = items[i].getBoundingClientRect();
      var distance = Math.abs(rect.top + rect.height / 2 - center);
      if (length > activeLength || (length === activeLength && distance < activeDistance)) {
        active = items[i];
        activeLength = length;
        activeDistance = distance;
      }
    }
    return active;
  }

  function restoreScroll() {
    var container = document.querySelector("[data-docs-sidebar-content]");
    if (!container || !container.clientHeight) return;
    if (stored && stored.pathname === location.pathname) {
      container.scrollTop = stored.scrollTop;
      return;
    }
    var active = getActiveItem(container);
    if (!active) return;
    var containerRect = container.getBoundingClientRect();
    var activeRect = active.getBoundingClientRect();
    if (activeRect.top >= containerRect.top && activeRect.bottom <= containerRect.bottom) return;
    container.scrollTop += activeRect.top - containerRect.top - (container.clientHeight - activeRect.height) / 2;
  }

  var observer = new MutationObserver(restoreScroll);
  observer.observe(document.documentElement, { childList: true, subtree: true });
  restoreScroll();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      restoreScroll();
      observer.disconnect();
    }, { once: true });
  } else {
    observer.disconnect();
  }
})();`;
