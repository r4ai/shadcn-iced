import type { ReactNode } from "react"

/** The fixtures of a component, keyed by the ids of its scenarios. */
export type Fixtures = Record<string, () => ReactNode>

const modules = import.meta.glob<{ default: Fixtures }>("./fixtures/*.tsx", {
  eager: true,
})

/** Every fixture, keyed by component (the file name). */
export const fixtures: Record<string, Fixtures> = Object.fromEntries(
  Object.entries(modules).map(([path, module]) => [
    path.replace(/^.*\/(.*)\.tsx$/, "$1"),
    module.default,
  ])
)
