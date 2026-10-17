// The navigation of the site: the links of the header, and the sections of
// the docs (the sidebar and the mobile menu).

export const NAV_ITEMS = [
  { to: "/", label: "Home" },
  { to: "/docs", label: "Docs" },
  { to: "/docs/components", label: "Components" },
  { to: "/charts", label: "Charts" },
  { to: "/examples", label: "Examples" },
  { to: "/docs/theming", label: "Theming" },
] as const;

export const DOCS_SECTIONS = [
  { to: "/docs", label: "Introduction" },
  { to: "/docs/components", label: "Components" },
  { to: "/docs/installation", label: "Installation" },
  { to: "/docs/theming", label: "Theming" },
] as const;

/** Whether the section `to` is the page at `pathname`, or contains it. */
export function isSectionActive(to: string, pathname: string) {
  const path = pathname.replace(/\/$/, "") || "/";
  return to === "/docs" || to === "/" ? path === to : path === to || path.startsWith(`${to}/`);
}
