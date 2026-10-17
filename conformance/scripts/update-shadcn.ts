/**
 * Updates the reference to the latest shadcn/ui: bumps the `shadcn` CLI, then
 * reinstalls every component of `src/components/ui` (and their dependencies)
 * from the registry, with the style of `components.json` (`base-nova`).
 *
 * ```sh
 * pnpm shadcn:update
 * ```
 *
 * Then run `pnpm qa` and look at what changed: failing expectations and
 * regressed scores point at the components that shadcn_iced must catch up with.
 */
import { execFileSync } from "node:child_process"
import { readdirSync } from "node:fs"
import { join } from "node:path"

import { root } from "../tests/spec.ts"

const run = (command: string, args: string[]) =>
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
  })

const components = readdirSync(join(root, "src/components/ui"))
  .filter((name) => name.endsWith(".tsx"))
  .map((name) => name.replace(/\.tsx$/, ""))

run("pnpm", ["update", "--latest", "shadcn"])
run("pnpm", ["exec", "shadcn", "add", "--yes", "--overwrite", ...components])
run("git", ["--no-pager", "diff", "--stat", "--", "."])
