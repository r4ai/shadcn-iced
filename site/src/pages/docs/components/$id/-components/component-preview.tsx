// An example, after components/component-preview-tabs.tsx of ui.shadcn.com:
// the live preview, and its code under it, collapsed to its first lines
// until it is shown.
import { createSignal, Show } from "solid-js";

import { CodeBlock } from "~/components/code-block";
import { PreviewFrame } from "~/components/preview-frame";
import { Button } from "~/components/ui/button";
import { githubUrl, minHeight, type Example } from "~/lib/components";

export function ComponentPreview(props: { page: string; example: Example }) {
  const [codeVisible, setCodeVisible] = createSignal(false);
  const file = () => `${props.example.file.split("/").pop()}:${props.example.line}`;

  return (
    <div
      data-slot="component-preview"
      data-not-typeset
      class="group relative mt-4 mb-12 flex flex-col overflow-hidden rounded-2xl border"
    >
      <div data-slot="preview" class="rounded-t-[calc(var(--radius-2xl)-1px)]">
        <PreviewFrame
          page={props.page}
          example={props.example.index}
          minHeight={minHeight(props.page)}
          heights={props.example.heights}
        />
      </div>
      <div
        data-slot="code"
        class="relative overflow-hidden **:data-[slot=copy-button]:right-4 [&_[data-rehype-pretty-code-figure]]:m-0! [&_[data-rehype-pretty-code-figure]]:rounded-t-none [&_[data-rehype-pretty-code-figure]]:border-t [&_pre]:max-h-72"
      >
        <Show
          when={codeVisible()}
          fallback={
            <div class="relative">
              <figure data-rehype-pretty-code-figure>
                <div class="contents" innerHTML={props.example.preview} />
              </figure>
              <div class="absolute inset-0 flex items-center justify-center pb-4">
                <div
                  class="absolute inset-0"
                  style={{
                    background:
                      "linear-gradient(to top, var(--color-code), color-mix(in oklab, var(--color-code) 60%, transparent), transparent)",
                  }}
                />
                <Button
                  size="sm"
                  variant="outline"
                  class="relative z-10 rounded-lg bg-background text-foreground shadow-none hover:bg-muted dark:bg-background dark:text-foreground dark:hover:bg-muted"
                  onClick={() => setCodeVisible(true)}
                >
                  View Code
                </Button>
              </div>
            </div>
          }
        >
          <CodeBlock snippet={props.example} title={file()} />
          <a
            href={githubUrl(props.example.file, props.example.line)}
            target="_blank"
            rel="noreferrer"
            class="absolute top-2.5 right-13 z-10 text-xs text-muted-foreground transition-colors hover:text-foreground"
          >
            View on GitHub
          </a>
        </Show>
      </div>
    </div>
  );
}
