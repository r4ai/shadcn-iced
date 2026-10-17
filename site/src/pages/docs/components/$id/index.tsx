import { createFileRoute, notFound } from "@tanstack/solid-router";
import { For, Show } from "solid-js";

import { components, loadComponent, slug, type ComponentDetail } from "~/lib/components";
import { seo } from "~/lib/seo";

import { DocsPage, type PagerLink } from "../../-components/docs-page";
import { Heading } from "../../-components/heading";
import { ComponentPreview } from "./-components/component-preview";
import { InstallationTabs } from "./-components/installation-tabs";

export const Route = createFileRoute("/docs/components/$id/")({
  loader: async ({ params }) => {
    const component = await loadComponent(params.id);
    if (!component) throw notFound();
    return component;
  },
  head: ({ loaderData }) => (loaderData ? seo(`${loaderData.title} - shadcn-iced`, loaderData.description) : {}),
  component: ComponentPage,
});

const pagerLink = (index: number): PagerLink | undefined => {
  const component = components[index];
  return component && { to: "/docs/components/$id", params: { id: component.id }, label: component.title };
};

function ComponentPage() {
  const data = Route.useLoaderData();

  // Keyed: the page of another component starts over (tabs, previews).
  return (
    <Show when={data()} keyed>
      {(component) => <ComponentDocs component={component} />}
    </Show>
  );
}

function ComponentDocs(props: { component: ComponentDetail }) {
  const index = components.findIndex((c) => c.id === props.component.id);
  const [demo, ...examples] = props.component.examples;

  return (
    <DocsPage
      title={props.component.title}
      description={props.component.description}
      toc={[
        { id: "installation", label: "Installation" },
        ...(examples.length ? [{ id: "examples", label: "Examples" }] : []),
        ...examples.map((example) => ({ id: slug(example.title), label: example.title, depth: 3 as const })),
      ]}
      prev={pagerLink(index - 1)}
      next={pagerLink(index + 1)}
    >
      <Show when={demo}>{(first) => <ComponentPreview page={props.component.id} example={first()} />}</Show>

      <Heading level={2} id="installation">
        Installation
      </Heading>
      <InstallationTabs component={props.component} />

      <Show when={examples.length > 0}>
        <Heading level={2} id="examples">
          Examples
        </Heading>
        <For each={examples}>
          {(example) => (
            <>
              <Heading level={3} id={slug(example.title)}>
                {example.title}
              </Heading>
              <ComponentPreview page={props.component.id} example={example} />
            </>
          )}
        </For>
      </Show>
    </DocsPage>
  );
}
