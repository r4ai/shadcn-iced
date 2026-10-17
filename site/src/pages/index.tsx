import { createFileRoute } from "@tanstack/solid-router";

import { seo } from "~/lib/seo";

import { Hero } from "./-components/hero";
import { Showcase } from "./-components/showcase";

export const Route = createFileRoute("/")({
  head: () =>
    seo(
      "shadcn-iced",
      "Beautifully designed components for iced, ported from shadcn/ui. Live previews compiled to WebAssembly.",
    ),
  component: Home,
});

function Home() {
  return (
    <div class="flex flex-1 flex-col">
      <Hero />
      <Showcase />
    </div>
  );
}
