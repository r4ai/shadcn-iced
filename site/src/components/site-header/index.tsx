// Port of the header of ui.shadcn.com (components/site-header.tsx).
import { Separator } from "~/components/ui/separator";

import { GitHubLink } from "./github-link";
import { MainNav } from "./main-nav";
import { MobileNav } from "./mobile-nav";
import { ModeSwitcher } from "./mode-switcher";
import { ThemeMenu } from "./theme-menu";

export function SiteHeader() {
  return (
    <header class="sticky top-0 z-50 w-full bg-background">
      <div class="container-wrapper px-6">
        <div class="flex h-(--header-height) items-center **:data-[slot=separator]:h-4!">
          <MobileNav class="flex lg:hidden" />
          <MainNav class="hidden lg:flex" />
          <div class="ml-auto flex items-center gap-2 md:flex-1 md:justify-end">
            <GitHubLink />
            <Separator orientation="vertical" />
            <ModeSwitcher />
            <Separator orientation="vertical" />
            <ThemeMenu />
          </div>
        </div>
      </div>
    </header>
  );
}
