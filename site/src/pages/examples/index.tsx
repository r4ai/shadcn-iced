import { createFileRoute } from "@tanstack/solid-router";

import { seo } from "~/lib/seo";

import { DESCRIPTION, ExamplesPage } from "./-components/examples-page";

// The first app, like ui.shadcn.com/examples.
export const Route = createFileRoute("/examples/")({
  head: () => seo("Examples - shadcn-iced", DESCRIPTION),
  component: () => <ExamplesPage name="dashboard" />,
});
