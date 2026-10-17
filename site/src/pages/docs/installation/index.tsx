import { createFileRoute } from "@tanstack/solid-router";

import { CodeBlock } from "~/components/code-block";
import { seo } from "~/lib/seo";

import { DocsPage } from "../-components/docs-page";
import { Heading } from "../-components/heading";
import add from "./-snippets/add.sh?highlight";
import cargoToml from "./-snippets/Cargo.toml?highlight";
import copied from "./-snippets/copied.rs?highlight";
import installCli from "./-snippets/install-cli.sh?highlight";
import main from "./-snippets/main.rs?highlight";

const TITLE = "Installation";
const DESCRIPTION = "Add shadcn-iced to an iced 0.14 application, as a library or as source you own.";

export const Route = createFileRoute("/docs/installation/")({
  head: () => seo(`${TITLE} - shadcn-iced`, DESCRIPTION),
  component: Installation,
});

function Installation() {
  return (
    <DocsPage
      title={TITLE}
      description={DESCRIPTION}
      toc={[
        { id: "as-a-library", label: "As a library" },
        { id: "copy-and-paste", label: "Copy and paste" },
        { id: "fonts-and-icons", label: "Fonts and icons" },
      ]}
      prev={{ to: "/docs", label: "Introduction" }}
      next={{ to: "/docs/theming", label: "Theming" }}
    >
      <Heading level={2} id="as-a-library">
        As a library
      </Heading>
      <p>
        Add <code>iced</code> and <code>shadcn_iced</code> to your <code>Cargo.toml</code>:
      </p>
      <CodeBlock snippet={cargoToml} title="Cargo.toml" />
      <p>Then import the prelude, pick a theme and load the bundled fonts:</p>
      <CodeBlock snippet={main} title="src/main.rs" />

      <Heading level={2} id="copy-and-paste">
        Copy and paste
      </Heading>
      <p>Install the CLI:</p>
      <CodeBlock snippet={installCli} />
      <p>Then, in your project, add the dependencies and the components you want:</p>
      <CodeBlock snippet={add} />
      <p>
        The components, and the components they use, are written to <code>src/ui/</code>, and <code>src/ui/mod.rs</code>{" "}
        is kept up to date. Declare the module and use them like any of your own code:
      </p>
      <CodeBlock snippet={copied} title="src/main.rs" />

      <Heading level={2} id="fonts-and-icons">
        Fonts and icons
      </Heading>
      <p>
        <code>.shadcn_iced()</code> loads Geist, Geist Mono, Inter and the Lucide icon font, and makes Geist the default
        font. Every Lucide icon is a constant of <code>shadcn_iced::lucide</code>, e.g. <code>lucide::ARROW_UP</code>.
      </p>
    </DocsPage>
  );
}
