import * as React from "react"
import { CircleAlert, CircleCheck, CircleDashed } from "lucide-react"

import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu"
import type { Fixtures } from "@/fixtures"

const components = [
  {
    title: "Alert Dialog",
    description:
      "A modal dialog that interrupts the user with important content and expects a response.",
  },
  {
    title: "Hover Card",
    description:
      "For sighted users to preview content available behind a link.",
  },
  {
    title: "Progress",
    description:
      "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
  },
  {
    title: "Scroll-area",
    description: "Visually or semantically separates content.",
  },
  {
    title: "Tabs",
    description:
      "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
  },
  {
    title: "Tooltip",
    description:
      "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
  },
]

function ListItem({
  title,
  children,
  href,
  ...props
}: React.ComponentPropsWithoutRef<"li"> & { href: string }) {
  return (
    <li {...props}>
      <NavigationMenuLink render={<a href={href} />}>
        <div className="flex flex-col gap-1 text-sm">
          <div className="leading-none font-medium">{title}</div>
          <div className="line-clamp-2 text-muted-foreground">{children}</div>
        </div>
      </NavigationMenuLink>
    </li>
  )
}

const example = () => (
  <NavigationMenu>
    <NavigationMenuList>
      <NavigationMenuItem>
        <NavigationMenuTrigger>Getting started</NavigationMenuTrigger>
        <NavigationMenuContent>
          <ul className="w-96">
            <ListItem href="#" title="Introduction">
              Re-usable components built with Tailwind CSS.
            </ListItem>
            <ListItem href="#" title="Installation">
              How to install dependencies and structure your app.
            </ListItem>
            <ListItem href="#" title="Typography">
              Styles for headings, paragraphs, lists...etc
            </ListItem>
          </ul>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem>
        <NavigationMenuTrigger>Components</NavigationMenuTrigger>
        <NavigationMenuContent>
          <ul className="grid w-[500px] gap-2 md:grid-cols-2">
            {components.map((component) => (
              <ListItem key={component.title} title={component.title} href="#">
                {component.description}
              </ListItem>
            ))}
          </ul>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem>
        <NavigationMenuTrigger>With Icon</NavigationMenuTrigger>
        <NavigationMenuContent>
          <ul className="grid w-[200px]">
            <li>
              <NavigationMenuLink render={<a href="#" />}>
                <CircleAlert />
                Backlog
              </NavigationMenuLink>
              <NavigationMenuLink render={<a href="#" />}>
                <CircleDashed />
                To Do
              </NavigationMenuLink>
              <NavigationMenuLink render={<a href="#" />}>
                <CircleCheck />
                Done
              </NavigationMenuLink>
            </li>
          </ul>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem>
        <NavigationMenuLink
          render={<a href="#" />}
          className={navigationMenuTriggerStyle()}
        >
          Docs
        </NavigationMenuLink>
      </NavigationMenuItem>
    </NavigationMenuList>
  </NavigationMenu>
)

export default {
  closed: example,
  "getting-started": example,
  components: example,
  switch: example,
} satisfies Fixtures
