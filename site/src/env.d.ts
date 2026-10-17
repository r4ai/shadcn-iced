/// <reference types="vite/client" />

/** A highlighted snippet: the code, and its HTML highlighted by Shiki. */
interface Highlighted {
  code: string;
  html: string;
  /** The language of Shiki, e.g. `rust` or `shellscript`. */
  lang: string;
}

declare module "*?highlight" {
  const snippet: Highlighted;
  export default snippet;
}

declare module "virtual:site/charts" {
  import type { ChartPage } from "~/lib/components";

  export const charts: ChartPage[];
}

declare module "virtual:site/components" {
  import type { ComponentDetail, ComponentSummary } from "~/lib/components";

  export const components: ComponentSummary[];
  export const loaders: Record<string, () => Promise<{ default: ComponentDetail }>>;
}
