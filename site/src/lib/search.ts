// The index of the search of the header (`CommandMenu`): every page of the
// site, in the groups of the command menu of ui.shadcn.com, and the headings
// and examples inside them, its "Search Results" (the full-text search of
// fumadocs there).
import { apps, charts } from "virtual:site/search";

import { components, slug } from "~/lib/components";
import { DOCS_SECTIONS, NAV_ITEMS } from "~/lib/navigation";

export interface SearchItem {
  /** The text the query is matched against, and the key of the item. */
  value: string;
  label: string;
  /** More words that match the item. */
  keywords?: string[];
  /** The page (and anchor) the item goes to, from the base of the site. */
  href: string;
  kind: "page" | "component" | "result";
  /** The page of a result, shown after it. */
  detail?: string;
  /** What ⌘C copies while the item is selected. */
  copy?: string;
}

export interface SearchGroup {
  heading: string;
  items: SearchItem[];
  /** Only shown for a query: too many items to list them all. */
  results?: boolean;
}

/** The kind of a chart page (`charts-area` → `area`). */
const kindOf = (id: string) => id.replace(/^charts-/, "");

/** The headings of the docs pages (their tables of contents). */
const DOCS_HEADINGS: { page: string; href: string; headings: [id: string, label: string][] }[] = [
  {
    page: "Introduction",
    href: "/docs",
    headings: [
      ["how-it-works", "How it works"],
      ["what-is-included", "What is included"],
      ["live-previews", "Live previews"],
    ],
  },
  {
    page: "Installation",
    href: "/docs/installation",
    headings: [
      ["as-a-library", "As a library"],
      ["copy-and-paste", "Copy and paste"],
      ["fonts-and-icons", "Fonts and icons"],
    ],
  },
  {
    page: "Theming",
    href: "/docs/theming",
    headings: [
      ["customize", "Customize"],
      ["tokens", "Tokens"],
    ],
  },
];

export const SEARCH_GROUPS: SearchGroup[] = [
  {
    heading: "Pages",
    items: NAV_ITEMS.map((item) => ({
      value: `Navigation ${item.label}`,
      label: item.label,
      keywords: ["nav", "navigation", item.label.toLowerCase()],
      href: item.to,
      kind: "page",
    })),
  },
  {
    heading: "Sections",
    items: DOCS_SECTIONS.map((section) => ({
      value: `Sections ${section.label}`,
      label: section.label,
      href: section.to,
      kind: "page",
    })),
  },
  {
    heading: "Components",
    items: components.map((component) => ({
      value: `Components ${component.title}`,
      label: component.title,
      keywords: ["component", component.id, component.description],
      href: `/docs/components/${component.id}`,
      kind: "component",
      copy: `cargo shadcn-iced add ${component.id}`,
    })),
  },
  {
    heading: "Charts",
    items: charts.map((chart) => ({
      value: `Charts ${chart.title}`,
      label: chart.title,
      keywords: ["chart", kindOf(chart.id)],
      href: `/charts/${kindOf(chart.id)}`,
      kind: "page",
    })),
  },
  {
    heading: "Examples",
    items: apps.map((app) => ({
      value: `Examples ${app.title}`,
      label: app.title,
      keywords: ["example", "app", app.id, app.description],
      href: `/examples/${app.id}`,
      kind: "page",
    })),
  },
  {
    heading: "Search Results",
    results: true,
    items: [
      ...DOCS_HEADINGS.flatMap((page) =>
        page.headings.map(([id, label]) => ({
          value: `${label} ${page.page} heading`,
          label,
          href: `${page.href}#${id}`,
          kind: "result" as const,
          detail: page.page,
        })),
      ),
      // The first example of a component is its demo, at the top of its page.
      ...components.flatMap((component) =>
        component.examples.slice(1).map((example) => ({
          value: `${example.title} ${component.title} example`,
          label: example.title,
          href: `/docs/components/${component.id}#${slug(example.title)}`,
          kind: "result" as const,
          detail: component.title,
          copy: `cargo shadcn-iced add ${component.id}`,
        })),
      ),
      ...charts.flatMap((chart) =>
        chart.examples.map((title) => ({
          value: `${title} ${chart.title} chart`,
          label: title,
          href: `/charts/${kindOf(chart.id)}`,
          kind: "result" as const,
          detail: chart.title,
        })),
      ),
    ],
  },
];

/**
 * Whether an item matches the query, as the filter of the command menu of
 * ui.shadcn.com: its value or a keyword contains the query, ignoring case.
 */
export function matches(item: SearchItem, query: string) {
  const text = `${item.value} ${item.keywords?.join(" ") ?? ""}`.toLowerCase();
  return text.includes(query.trim().toLowerCase());
}

/** The groups of the items that match `query`, without the empty ones. */
export function search(query: string): SearchGroup[] {
  const blank = !query.trim();

  return SEARCH_GROUPS.flatMap((group) => {
    if (group.results && blank) return [];
    const q = query.trim().toLowerCase();
    // Those whose label has the query first, as cmdk ranks them.
    const items = blank
      ? group.items
      : group.items
          .filter((item) => matches(item, query))
          .toSorted((a, b) => Number(b.label.toLowerCase().includes(q)) - Number(a.label.toLowerCase().includes(q)));
    return items.length ? [{ ...group, items }] : [];
  });
}
