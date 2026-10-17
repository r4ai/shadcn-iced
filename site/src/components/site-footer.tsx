// Port of components/site-footer.tsx of ui.shadcn.com: hidden on the pages
// of the docs.
import { REPOSITORY } from "~/lib/components";

export function SiteFooter() {
  return (
    <footer class="group-has-[[data-slot=docs]]/body:hidden">
      <div class="container-wrapper px-4 xl:px-6">
        <div class="flex h-(--footer-height) items-center justify-between">
          <div class="w-full px-1 text-center text-xs leading-loose text-muted-foreground sm:text-sm">
            Built with{" "}
            <a href="https://iced.rs" target="_blank" rel="noreferrer" class="font-medium underline underline-offset-4">
              iced
            </a>{" "}
            and{" "}
            <a
              href="https://ui.shadcn.com"
              target="_blank"
              rel="noreferrer"
              class="font-medium underline underline-offset-4"
            >
              shadcn/ui
            </a>
            . The source code is available on{" "}
            <a href={REPOSITORY} target="_blank" rel="noreferrer" class="font-medium underline underline-offset-4">
              GitHub
            </a>
            .
          </div>
        </div>
      </div>
    </footer>
  );
}
