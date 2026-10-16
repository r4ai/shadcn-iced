// The color mode and the theme options of the site, kept in localStorage.
import { DEFAULT_THEME, type ThemeOptions } from "../lib/theme";

const MODE_KEY = "iced-ui:mode";
const THEME_KEY = "iced-ui:theme";
const EVENT = "iced-ui:theme";

function read(key: string) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // The choice then lasts until the page is left.
  }
}

export const isDark = () => document.documentElement.classList.contains("dark");

export function setDark(dark: boolean) {
  document.documentElement.classList.toggle("dark", dark);
  write(MODE_KEY, dark ? "dark" : "light");
  window.dispatchEvent(new Event(EVENT));
}

export function currentTheme(): ThemeOptions {
  try {
    return { ...DEFAULT_THEME, ...JSON.parse(read(THEME_KEY) ?? "{}") };
  } catch {
    return DEFAULT_THEME;
  }
}

export function setTheme(theme: ThemeOptions) {
  write(THEME_KEY, JSON.stringify(theme));
  window.dispatchEvent(new Event(EVENT));
}

export function onThemeChange(listener: () => void) {
  window.addEventListener(EVENT, listener);
}

// Follow the system when no mode was picked.
window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (event) => {
  if (!read(MODE_KEY)) {
    document.documentElement.classList.toggle("dark", event.matches);
    window.dispatchEvent(new Event(EVENT));
  }
});
