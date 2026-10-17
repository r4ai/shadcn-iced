import { createFileRoute, notFound } from "@tanstack/solid-router";
import { charts } from "virtual:site/charts";

import { seo } from "~/lib/seo";

import { ChartsPage, DESCRIPTION, kindOf } from "./-components/charts-page";

export const Route = createFileRoute("/charts/$kind")({
  loader: ({ params }) => {
    const page = charts.find((chart) => kindOf(chart.id) === params.kind);
    if (!page) throw notFound();
    return { title: page.title };
  },
  head: ({ loaderData }) => seo(`${loaderData?.title ?? "Charts"} - shadcn-iced`, DESCRIPTION),
  component: KindPage,
});

function KindPage() {
  const params = Route.useParams();

  return <ChartsPage kind={params().kind} />;
}
