import { createFileRoute } from "@tanstack/solid-router";
import { For } from "solid-js";

import { CodeBlock } from "~/components/code-block";
import { PreviewFrame } from "~/components/preview-frame";
import { PreviewThemeFields } from "~/components/preview-theme-fields";
import { components } from "~/lib/components";
import { seo } from "~/lib/seo";

import { DocsPage } from "../-components/docs-page";
import { Heading } from "../-components/heading";
import { ThemeCode } from "./-components/theme-code";
import tokens from "./-snippets/tokens.rs?highlight";

const TITLE = "Theming";
const DESCRIPTION = "Every theme of shadcn/ui's create page, generated from its OKLCH tokens, in light and dark mode.";

// A few examples that show the colors, the radius and the charts.
const PICKS = [
  ["card", "Demo"],
  ["chart", "Area Chart - Stacked"],
  ["button", "Variants"],
  ["switch", "Choice Card"],
] as const;

const PREVIEWS = PICKS.flatMap(([id, title]) => {
  const example = components.find((c) => c.id === id)?.examples.find((e) => e.title === title);
  return example ? [{ id, example: example.index, heights: example.heights }] : [];
});

export const Route = createFileRoute("/docs/theming/")({
  head: () => seo(`${TITLE} - shadcn-iced`, DESCRIPTION),
  component: Theming,
});

function Theming() {
  return (
    <DocsPage
      title={TITLE}
      description={DESCRIPTION}
      toc={[
        { id: "customize", label: "Customize" },
        { id: "tokens", label: "Tokens" },
      ]}
      prev={{ to: "/docs/installation", label: "Installation" }}
      next={{ to: "/docs/components", label: "Components" }}
    >
      <Heading level={2} id="customize">
        Customize
      </Heading>
      <p>
        Pick a base color, a theme and a radius: the previews below, and every preview of the site, follow. Then copy
        the code that builds the theme into your application.
      </p>
      <div data-not-typeset class="mt-6 grid gap-6 rounded-2xl border p-4 sm:p-6">
        <PreviewThemeFields class="gap-4 sm:grid-cols-3" />
        <ThemeCode />
      </div>
      <div data-not-typeset class="mt-6 grid gap-4 lg:grid-cols-2">
        <For each={PREVIEWS}>
          {(preview) => (
            <div class="overflow-hidden rounded-2xl border">
              <PreviewFrame page={preview.id} example={preview.example} minHeight={300} heights={preview.heights} />
            </div>
          )}
        </For>
      </div>

      <Heading level={2} id="tokens">
        Tokens
      </Heading>
      <p>
        A theme is an <code>iced::Theme</code> that carries the tokens of shadcn/ui: <code>background</code>,{" "}
        <code>foreground</code>, <code>primary</code>, <code>muted</code>, <code>accent</code>, <code>border</code>,{" "}
        <code>ring</code>, the chart and sidebar colors, and the radius. Components read them with{" "}
        <code>shadcn_iced::tokens(theme)</code>, and so can your own widgets:
      </p>
      <CodeBlock snippet={tokens} />
    </DocsPage>
  );
}
