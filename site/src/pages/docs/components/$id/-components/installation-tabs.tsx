// The ways to install a component, in the tabs of `CodeTabs` and the
// `Tabs*` of mdx-components.tsx of ui.shadcn.com. The choice is remembered,
// like the `installationType` of its config.
import { createSignal, onMount, Show } from "solid-js";

import { CodeBlock } from "~/components/code-block";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { githubUrl, moduleName, type ComponentDetail } from "~/lib/components";
import { read, write } from "~/lib/storage";

// `data-[state=active]` of Radix is `data-[selected]` in Kobalte.
const TRIGGER =
  "not-typset rounded-none border-0 border-b-2 border-transparent bg-transparent px-0 pb-3 text-base text-muted-foreground hover:text-primary data-[selected]:border-primary data-[selected]:bg-transparent data-[selected]:text-foreground data-[selected]:shadow-none! dark:data-[selected]:border-primary dark:data-[selected]:bg-transparent";
const CONTENT = "relative *:[figure]:first:mt-0";

const INSTALLATION_KEY = "shadcn-iced:installation";
type Installation = "cli" | "library" | "manual";

export function InstallationTabs(props: { component: ComponentDetail }) {
  const [installation, setInstallation] = createSignal<Installation>("cli");
  // The manual installation needs the source of the component.
  const value = () => (installation() === "manual" && !props.component.source ? "cli" : installation());

  onMount(() => {
    const stored = read(INSTALLATION_KEY);
    if (stored === "cli" || stored === "library" || stored === "manual") setInstallation(stored);
  });

  const choose = (choice: string) => {
    setInstallation(choice as Installation);
    write(INSTALLATION_KEY, choice);
  };

  return (
    <Tabs value={value()} onChange={choose} class="relative mt-6 w-full *:data-[slot=tabs-list]:gap-6">
      <TabsList class="justify-start gap-4 rounded-none bg-transparent px-0">
        <TabsTrigger value="cli" class={TRIGGER}>
          Command
        </TabsTrigger>
        <TabsTrigger value="library" class={TRIGGER}>
          Library
        </TabsTrigger>
        <Show when={props.component.source}>
          <TabsTrigger value="manual" class={TRIGGER}>
            Manual
          </TabsTrigger>
        </Show>
      </TabsList>
      <TabsContent value="cli" class={CONTENT}>
        <CodeBlock snippet={props.component.install.cli} />
        <p>
          It copies the component, and the components it uses, into <code>src/ui/</code>.
        </p>
      </TabsContent>
      <TabsContent value="library" class={CONTENT}>
        <CodeBlock snippet={props.component.install.library} />
        <p>
          It uses the component from the <code>shadcn_iced</code> crate.
        </p>
      </TabsContent>
      <Show when={props.component.source}>
        {(source) => (
          <TabsContent value="manual" class={CONTENT}>
            <p>
              Copy the source of the component into <code>src/ui/{moduleName(props.component.id)}.rs</code> (it uses{" "}
              <code>shadcn_iced_core</code> and, sometimes, other components).{" "}
              <a href={githubUrl(source().file)} target="_blank" rel="noreferrer">
                View it on GitHub
              </a>
              .
            </p>
            <CodeBlock snippet={source()} title={source().file} expandable />
          </TabsContent>
        )}
      </Show>
    </Tabs>
  );
}
