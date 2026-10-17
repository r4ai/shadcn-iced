// The color mode of the site and the theme of the live previews, kept in
// localStorage. The mode is set before the first paint by the script of
// `__root.tsx`; the provider takes it over once the page is hydrated. Like
// the next-themes setup of ui.shadcn.com, a switch does not animate, the D key
// switches, and the `theme-color` of the browser follows.
import { createContext, createEffect, createSignal, onCleanup, onMount, useContext, type ParentProps } from "solid-js";
import { createStore } from "solid-js/store";

import { previews } from "~/lib/previews";
import { read, write } from "~/lib/storage";
import { DEFAULT_THEME, type ThemeOptions } from "~/lib/theme";

export const MODE_KEY = "shadcn-iced:mode";

/** `META_THEME_COLORS` of hooks/use-meta-color.ts of ui.shadcn.com. */
export const META_THEME_COLORS = { light: "#ffffff", dark: "#0a0a0a" };
const THEME_KEY = "shadcn-iced:theme";

interface ThemeContextValue {
  dark: () => boolean;
  setDark: (dark: boolean) => void;
  /** The theme of the previews. */
  theme: ThemeOptions;
  setTheme: (theme: Partial<ThemeOptions>) => void;
}

const ThemeContext = createContext<ThemeContextValue>();

export function ThemeProvider(props: ParentProps) {
  const [dark, setDarkSignal] = createSignal(false);
  const [theme, setThemeStore] = createStore<ThemeOptions>({ ...DEFAULT_THEME });

  const applyDark = (value: boolean) => {
    // `disableTransitionOnChange` of next-themes: no color animates.
    const style = document.createElement("style");
    style.textContent = "*,*::before,*::after{transition:none!important}";
    document.head.append(style);

    document.documentElement.classList.toggle("dark", value);
    document
      .querySelector('meta[name="theme-color"]')
      ?.setAttribute("content", value ? META_THEME_COLORS.dark : META_THEME_COLORS.light);
    setDarkSignal(value);

    // Apply the new colors before the transitions come back.
    void getComputedStyle(document.body).opacity;
    requestAnimationFrame(() => style.remove());
  };

  const setDark = (on: boolean) => {
    applyDark(on);
    write(MODE_KEY, on ? "dark" : "light");
  };

  // `ThemeShortcut`: D switches the mode, outside of the fields.
  const shortcut = (event: KeyboardEvent) => {
    if ((event.key !== "d" && event.key !== "D") || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target;
    if (
      (target instanceof HTMLElement && target.isContentEditable) ||
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement
    ) {
      return;
    }
    event.preventDefault();
    setDark(!dark());
  };

  // Follow the system while no mode was picked.
  const follow = (event: MediaQueryListEvent) => {
    if (!read(MODE_KEY)) applyDark(event.matches);
  };

  onMount(() => {
    setDarkSignal(document.documentElement.classList.contains("dark"));
    try {
      setThemeStore({ ...DEFAULT_THEME, ...JSON.parse(read(THEME_KEY) ?? "{}") });
    } catch {
      // Keep the default theme.
    }

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    media.addEventListener("change", follow);
    document.addEventListener("keydown", shortcut);
    onCleanup(() => {
      media.removeEventListener("change", follow);
      document.removeEventListener("keydown", shortcut);
    });
  });

  // The previews follow the mode and the theme.
  createEffect(() => previews()?.setTheme(dark(), theme.base, theme.accent, theme.radius));

  const value: ThemeContextValue = {
    dark,
    setDark,
    theme,
    setTheme: (patch) => {
      setThemeStore(patch);
      write(THEME_KEY, JSON.stringify(theme));
    },
  };

  return <ThemeContext.Provider value={value}>{props.children}</ThemeContext.Provider>;
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used inside <ThemeProvider>");
  return context;
}
