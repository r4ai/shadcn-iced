// Prerendered to `404.html`, the page Cloudflare serves for unknown paths.
import { createFileRoute } from "@tanstack/solid-router";

import { NotFound } from "~/components/not-found";
import { seo } from "~/lib/seo";

export const Route = createFileRoute("/404")({
  head: () => seo("Page not found - shadcn-iced", "This page does not exist."),
  component: NotFound,
});
