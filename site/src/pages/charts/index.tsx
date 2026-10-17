import { createFileRoute } from "@tanstack/solid-router";

import { seo } from "~/lib/seo";

import { ChartsPage, DESCRIPTION } from "./-components/charts-page";

// The first kind of chart, like ui.shadcn.com/charts.
export const Route = createFileRoute("/charts/")({
  head: () => seo("Charts - shadcn-iced", DESCRIPTION),
  component: () => <ChartsPage kind="area" />,
});
