/**
 * Records the scores and the known shadcn_iced failures of the last
 * Playwright run in `baseline.json`.
 *
 * ```sh
 * pnpm baseline            # every run of the last run
 * pnpm baseline button/    # only the keys that contain a filter
 * ```
 *
 * Review the diff of `baseline.json` before committing it: a score that goes
 * up or a new known failure means that shadcn_iced drifted away from
 * shadcn/ui.
 */
import { readFileSync, readdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"

import {
  out,
  readBaseline,
  root,
  runKey,
  type RunRecord,
} from "../tests/spec.ts"

const filters = process.argv.slice(2)
const matches = (key: string) =>
  !filters.length || filters.some((filter) => key.includes(filter))

const { scores, knownFailures } = readBaseline()
let updated = 0

for (const name of readdirSync(join(out, "runs"))) {
  const run: RunRecord = JSON.parse(
    readFileSync(join(out, "runs", name), "utf8")
  )

  for (const snapshot of run.snapshots) {
    if (snapshot.score === null || !matches(snapshot.key)) continue

    if (scores[snapshot.key] !== snapshot.score) {
      scores[snapshot.key] = snapshot.score
      updated++
    }
  }

  const key = runKey(run.component, run.scenario, run.theme)
  if (!matches(key)) continue

  if (run.reference.failures.length) {
    console.warn(`${key}: fix the shadcn/ui expectations first`)
    continue
  }

  if (
    JSON.stringify(knownFailures[key] ?? []) !==
    JSON.stringify(run.iced.failures)
  ) {
    updated++
  }

  if (run.iced.failures.length) {
    knownFailures[key] = run.iced.failures
  } else {
    delete knownFailures[key]
  }
}

const sorted = <T>(record: Record<string, T>) =>
  Object.fromEntries(
    Object.entries(record).sort(([a], [b]) => a.localeCompare(b))
  )

writeFileSync(
  join(root, "baseline.json"),
  JSON.stringify(
    { scores: sorted(scores), knownFailures: sorted(knownFailures) },
    null,
    2
  ) + "\n"
)
console.log(`baseline.json: ${updated} entries updated`)
