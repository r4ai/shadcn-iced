import { useState } from "react"
import {
  BookOpenIcon,
  BotIcon,
  ChevronsUpDownIcon,
  FrameIcon,
  GalleryVerticalEndIcon,
  LifeBuoyIcon,
  MapIcon,
  MoreHorizontalIcon,
  PieChartIcon,
  Settings2Icon,
  SquareTerminalIcon,
} from "lucide-react"

import type { Fixtures } from "@/fixtures"
import { Separator } from "@/components/ui/separator"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"

const platform = [
  { title: "Playground", icon: SquareTerminalIcon },
  { title: "Models", icon: BotIcon },
  { title: "Documentation", icon: BookOpenIcon },
  { title: "Settings", icon: Settings2Icon },
]

const projects = [
  { title: "Design Engineering", icon: FrameIcon },
  { title: "Sales & Marketing", icon: PieChartIcon },
  { title: "Travel", icon: MapIcon },
]

/**
 * The sidebar fills a `526px` high frame (a transformed box keeps its `fixed`
 * container inside), collapsing to icons.
 */
function Demo() {
  const [active, setActive] = useState("Playground")

  const item = (title: string, Icon: typeof BotIcon) => (
    <SidebarMenuItem key={title}>
      <SidebarMenuButton
        isActive={active === title}
        tooltip={title}
        onClick={() => setActive(title)}
      >
        <Icon />
        <span>{title}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )

  return (
    <div className="relative h-[528px] [transform:translateZ(0)] overflow-hidden rounded-lg border">
      <SidebarProvider style={{ minHeight: 0, height: "100%" }}>
        <Sidebar collapsible="icon" style={{ height: "100%" }}>
          <SidebarHeader>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton size="lg">
                  <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                    <GalleryVerticalEndIcon className="size-4" />
                  </div>
                  <div className="grid flex-1 text-left text-sm leading-tight">
                    <span className="truncate font-medium">Acme Inc</span>
                    <span className="truncate text-xs">Enterprise</span>
                  </div>
                  <ChevronsUpDownIcon className="ml-auto" />
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarHeader>
          <SidebarContent>
            <SidebarGroup>
              <SidebarGroupLabel>Platform</SidebarGroupLabel>
              <SidebarMenu>
                {platform.map(({ title, icon }) => item(title, icon))}
              </SidebarMenu>
            </SidebarGroup>
            <SidebarGroup>
              <SidebarGroupLabel>Projects</SidebarGroupLabel>
              <SidebarMenu>
                {projects.map(({ title, icon }) => item(title, icon))}
                <SidebarMenuItem>
                  <SidebarMenuButton onClick={() => setActive("More")}>
                    <MoreHorizontalIcon />
                    <span>More</span>
                  </SidebarMenuButton>
                  <SidebarMenuBadge>24</SidebarMenuBadge>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          </SidebarContent>
          <SidebarFooter>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton>
                  <LifeBuoyIcon />
                  <span>Support</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarFooter>
        </Sidebar>
        <SidebarInset>
          <header className="flex h-12 shrink-0 items-center gap-2 px-4">
            <SidebarTrigger />
            <Separator orientation="vertical" className="h-4" />
            <span className="text-sm">Data Fetching</span>
          </header>
        </SidebarInset>
      </SidebarProvider>
    </div>
  )
}

export default {
  default: () => <Demo />,
  collapse: () => <Demo />,
} satisfies Fixtures
