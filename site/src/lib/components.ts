// The components of the gallery (see `plugins/site-data.ts`).
import { components, loaders, widths } from "virtual:site/components";

export interface ComponentSummary {
  id: string;
  title: string;
  description: string;
  examples: { index: number; title: string; heights: PreviewHeights }[];
}

/**
 * The heights of the preview of an example (its frame) at each width of
 * `previewWidths`, measured by `site_data` as the preview lays it out:
 * `null` for an example that fills its height.
 */
export type PreviewHeights = (number | null)[];

export interface Example {
  index: number;
  title: string;
  /** The gallery file that builds the example, relative to the repository. */
  file: string;
  line: number;
  code: string;
  html: string;
  lang: string;
  /** The highlighted first lines of the code. */
  preview: string;
  heights: PreviewHeights;
}

export interface ComponentDetail {
  id: string;
  title: string;
  description: string;
  install: { cli: Highlighted; library: Highlighted };
  /** The source of the component, for the manual installation. */
  source: ({ file: string } & Highlighted) | null;
  examples: Example[];
}

export { components };

/** The widths of a preview that the heights of the examples are measured at. */
export const previewWidths: number[] = widths;

/** A page of the chart showcase (`/charts/<kind>`): examples of a kind of chart. */
export interface ChartPage {
  /** The id of the page in the gallery (`charts-area`). */
  id: string;
  title: string;
  description: string;
  examples: Example[];
}

/** An example app (`/examples/<id>`): a crate of its own, previewed whole. */
export interface AppPage {
  /** The id of the page in the gallery, and the name of the crate. */
  id: string;
  title: string;
  description: string;
  /** The command that runs the app (`cargo run -p dashboard`). */
  command: Highlighted;
  /** A single example: the app, with the code of its `src/lib.rs`. */
  examples: Example[];
}

/** Loads a component with its highlighted code, or `undefined`. */
export async function loadComponent(id: string): Promise<ComponentDetail | undefined> {
  return (await loaders[id]?.())?.default;
}

export const REPOSITORY = "https://github.com/r4ai/shadcn-iced";

export function githubUrl(file: string, line?: number) {
  return `${REPOSITORY}/blob/main/${file}${line ? `#L${line}` : ""}`;
}

/** The module of a component (`dropdown-menu` → `dropdown_menu`). */
export const moduleName = (id: string) => id.replaceAll("-", "_");

/**
 * The smallest height of a preview (it grows to fit its example): overlays
 * need room to open inside their canvas.
 */
export function minHeight(id: string) {
  const tall: Record<string, number> = {
    sheet: 560,
    drawer: 520,
    dialog: 480,
    "alert-dialog": 400,
    command: 480,
    "dropdown-menu": 480,
    "context-menu": 420,
    menubar: 440,
    "navigation-menu": 440,
    combobox: 440,
    select: 420,
    "date-picker": 460,
    popover: 400,
    sonner: 400,
    toast: 400,
    // The example apps fill their frame and scroll inside it, like their
    // window: the settings would change the height of the page with each
    // section otherwise.
    dashboard: 860,
    settings: 860,
  };

  return tall[id] ?? 320;
}

/** The anchor of a heading (`Area Chart - Stacked` → `area-chart-stacked`). */
export const slug = (title: string) =>
  title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
