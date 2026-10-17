import { createFileRoute, notFound } from "@tanstack/solid-router";
import { apps } from "virtual:site/apps";

import { seo } from "~/lib/seo";

import { DESCRIPTION, ExamplesPage } from "./-components/examples-page";

export const Route = createFileRoute("/examples/$name")({
  loader: ({ params }) => {
    const app = apps.find((candidate) => candidate.id === params.name);
    if (!app) throw notFound();
    return { title: app.title };
  },
  head: ({ loaderData }) => seo(`${loaderData?.title ?? "Examples"} - shadcn-iced`, DESCRIPTION),
  component: NamePage,
});

function NamePage() {
  const params = Route.useParams();

  return <ExamplesPage name={params().name} />;
}
