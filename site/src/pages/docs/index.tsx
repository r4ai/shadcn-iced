import { createFileRoute, Link } from "@tanstack/solid-router";

import { CodeBlock } from "~/components/code-block";
import { components, REPOSITORY } from "~/lib/components";
import { seo } from "~/lib/seo";

import { DocsPage } from "./-components/docs-page";
import { Heading } from "./-components/heading";
import card from "./-snippets/card.rs?highlight";

const TITLE = "Introduction";
const DESCRIPTION =
  "Beautifully designed components for iced, ported from shadcn/ui. Use them as a library, or copy them into your project and make them your own.";

export const Route = createFileRoute("/docs/")({
  head: () => seo(`${TITLE} - shadcn-iced`, DESCRIPTION),
  component: Introduction,
});

function Introduction() {
  return (
    <DocsPage
      title={TITLE}
      description={DESCRIPTION}
      toc={[
        { id: "how-it-works", label: "How it works" },
        { id: "what-is-included", label: "What is included" },
        { id: "live-previews", label: "Live previews" },
      ]}
      next={{ to: "/docs/installation", label: "Installation" }}
    >
      <p>
        shadcn-iced brings the components of{" "}
        <a href="https://ui.shadcn.com" target="_blank" rel="noreferrer">
          shadcn/ui
        </a>{" "}
        to{" "}
        <a href="https://iced.rs" target="_blank" rel="noreferrer">
          iced
        </a>
        , the cross-platform GUI library for Rust. It follows the current default style of shadcn/ui (Base UI with the
        Nova style), down to its spacing, colors, radii and animations.
      </p>
      <CodeBlock snippet={card} title="src/main.rs" />

      <Heading level={2} id="how-it-works">
        How it works
      </Heading>
      <p>Like shadcn/ui, shadcn-iced is the code of its components first. You choose how to use it:</p>
      <ul>
        <li>
          <strong>As a library</strong>: depend on the <code>shadcn_iced</code> crate and update it like any dependency.
        </li>
        <li>
          <strong>Copy and paste</strong>: <code>cargo shadcn-iced add button dialog</code> writes the source of the
          components to <code>src/ui/</code>, where you can change anything.
        </li>
      </ul>
      <p>
        Both share <code>shadcn_iced_core</code>: the theme, the fonts and icons, and the headless primitives (anchored
        overlays, modal layers, listboxes, disclosures) that Base UI provides to shadcn/ui.
      </p>

      <Heading level={2} id="what-is-included">
        What is included
      </Heading>
      <ul>
        <li>
          <Link to="/docs/components">{components.length} components</Link>, the whole registry of shadcn/ui.
        </li>
        <li>
          Every theme of the <code>/create</code> page of shadcn/ui: 7 base colors, 17 themes, chart colors, radii and
          menu styles, in light and dark mode. See <Link to="/docs/theming">Theming</Link>.
        </li>
        <li>Geist, Geist Mono and Inter, and all 2,179 Lucide icons, bundled.</li>
        <li>The animations of tw-animate-css for overlays, sheets, accordions and toasts.</li>
      </ul>

      <Heading level={2} id="live-previews">
        Live previews
      </Heading>
      <p>
        The previews of this site are not screenshots. Each one runs the example from the{" "}
        <a href={`${REPOSITORY}/tree/main/examples/gallery`} target="_blank" rel="noreferrer">
          gallery
        </a>{" "}
        with iced compiled to WebAssembly, drawn with WebGPU (or a software renderer when your browser lacks it). The
        code under it is the source that builds it. Use the Theme menu in the header to see them in any theme.
      </p>
    </DocsPage>
  );
}
