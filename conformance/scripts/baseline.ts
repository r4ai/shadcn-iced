/**
 * Records the scores of the last Playwright run in `baseline.json`.
 *
 * ```sh
 * pnpm baseline            # every snapshot of the last run
 * pnpm baseline button/    # only the snapshots whose key contains a filter
 * ```
 *
 * Review the diff of `baseline.json` before committing it: a score that goes
 * up means shadcn_iced drifted away from shadcn/ui.
 */
import { readFileSync, readdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"

import { out, readBaseline, root, type RunRecord } from "../tests/spec.ts"

const filters = process.argv.slice(2)
const scores = readBaseline()
let updated = 0

for (const name of readdirSync(join(out, "runs"))) {
  const run: RunRecord = JSON.parse(
    readFileSync(join(out, "runs", name), "utf8")
  )

  for (const snapshot of run.snapshots) {
    if (snapshot.score === null) continue
    if (
      filters.length &&
      !filters.some((filter) => snapshot.key.includes(filter))
    ) {
      continue
    }

    if (scores[snapshot.key] !== snapshot.score) {
      scores[snapshot.key] = snapshot.score
      updated++
    }
  }
}

const sorted = Object.fromEntries(
  Object.entries(scores).sort(([a], [b]) => a.localeCompare(b))
)

writeFileSync(
  join(root, "baseline.json"),
  JSON.stringify({ scores: sorted }, null, 2) + "\n"
)
console.log(`baseline.json: ${updated} scores updated`)
