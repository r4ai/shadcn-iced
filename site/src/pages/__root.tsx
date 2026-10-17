import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/solid-router";
import { onMount, type JSX } from "solid-js";
import { HydrationScript } from "solid-js/web";

import { SiteFooter } from "~/components/site-footer";
import { SiteHeader } from "~/components/site-header";
import { META_THEME_COLORS, MODE_KEY, ThemeProvider } from "~/components/theme-provider";
import { DOCS_SIDEBAR_SCROLL_RESTORE_SCRIPT } from "~/lib/docs-sidebar-scroll";

import css from "~/styles/global.css?url";
// After the styles of the site, as on ui.shadcn.com.
import typesetCss from "~/styles/shadcn/typeset.css?url";

const base = import.meta.env.BASE_URL.replace(/\/$/, "");

// Sets the mode before the first paint.
const MODE_SCRIPT = `(() => {
  let mode = null;
  try { mode = localStorage.getItem(${JSON.stringify(MODE_KEY)}); } catch {}
  const dark = mode ? mode === "dark" : matchMedia("(prefers-color-scheme: dark)").matches;
  document.documentElement.classList.toggle("dark", dark);
  document.querySelector('meta[name="theme-color"]')
    ?.setAttribute("content", dark ? ${JSON.stringify(META_THEME_COLORS.dark)} : ${JSON.stringify(META_THEME_COLORS.light)});
})();`;

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charset: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { name: "theme-color", content: META_THEME_COLORS.light },
    ],
    links: [
      { rel: "stylesheet", href: css },
      { rel: "stylesheet", href: typesetCss },
      { rel: "icon", href: `${base}/favicon.svg`, type: "image/svg+xml" },
      { rel: "preconnect", href: "https://fonts.googleapis.com" },
      { rel: "preconnect", href: "https://fonts.gstatic.com", crossOrigin: "anonymous" },
      {
        rel: "stylesheet",
        href: "https://fonts.googleapis.com/css2?family=Geist+Mono:wght@400&family=Geist:wght@100..900&display=swap",
      },
    ],
    scripts: [{ children: MODE_SCRIPT }, { children: DOCS_SIDEBAR_SCROLL_RESTORE_SCRIPT }],
  }),
  shellComponent: RootDocument,
  component: RootLayout,
});

function RootDocument(props: { children: JSX.Element }) {
  return (
    <html lang="en" class="[--header-height:calc(var(--spacing)*14)] lg:[--header-height:calc(var(--spacing)*16)]">
      <head>
        <HydrationScript />
      </head>
      <body class="group/body overscroll-none antialiased [--footer-height:calc(var(--spacing)*14)] xl:[--footer-height:calc(var(--spacing)*24)]">
        <HeadContent />
        {props.children}
        <Scripts />
      </body>
    </html>
  );
}

function RootLayout() {
  // Marks the page as interactive (the end-to-end tests wait for it).
  onMount(() => (document.documentElement.dataset.hydrated = "true"));

  return (
    <ThemeProvider>
      <div data-slot="layout" class="group/layout relative z-10 flex min-h-svh flex-col bg-background">
        <SiteHeader />
        <main class="flex min-h-0 flex-1 flex-col">
          <Outlet />
        </main>
        <SiteFooter />
      </div>
    </ThemeProvider>
  );
}
