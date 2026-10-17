/**
 * The scenario files shared with the iced runner (`conformance/scenarios`).
 *
 * Mirrors `conformance/iced/src/spec.rs`; the format is documented in
 * `conformance/README.md` and `scenarios/schema.json`.
 */
import { existsSync, readFileSync, readdirSync } from "node:fs"
import { join } from "node:path"

import type { Box } from "./compare.ts"

export type Theme = "light" | "dark"

/** Where a pointer action happens. */
export type Locator =
  /** The center of the first visible text with this exact content. */
  | string
  /**
   * The center of the `nth` (0-based) visible text with this content, or
   * containing it when `exact` is false (the smallest element on the web).
   */
  | { text: string; nth?: number; exact?: boolean }
  /** A logical position in the page. */
  | { at: [number, number] }

export type Step =
  | { snapshot: string }
  /** `steps` intermediate pointer moves (1 by default: a jump). */
  | { hover: Locator; steps?: number }
  | { click: Locator }
  | { rightClick: Locator }
  | { drag: { from: Locator; to: Locator } }
  /** A key combination with Playwright's names, e.g. `ArrowDown` or `Shift+Tab`. */
  | { press: string }
  | { type: string }
  | { wait: number }
  | { expect: { visible?: string[]; hidden?: string[] } }

export interface Scenario {
  id: string
  description?: string
  /** The logical size of the page. */
  viewport?: [number, number]
  themes?: Theme[]
  steps?: Step[]
}

export interface ScenarioFile {
  component: string
  scenarios: Scenario[]
}

export const root = join(import.meta.dirname, "..")
export const out = join(root, "out")

export const DEFAULT_VIEWPORT: [number, number] = [320, 120]
export const THEMES: Theme[] = ["light", "dark"]

/** A scenario without steps takes a single snapshot. */
export function steps(scenario: Scenario): Step[] {
  return scenario.steps?.length ? scenario.steps : [{ snapshot: "default" }]
}

export function readScenarios(): ScenarioFile[] {
  const directory = join(root, "scenarios")

  return readdirSync(directory)
    .filter((name) => name.endsWith(".json") && name !== "schema.json")
    .sort()
    .map(
      (name) =>
        JSON.parse(readFileSync(join(directory, name), "utf8")) as ScenarioFile
    )
}

/** The key of a snapshot in `baseline.json` and the reports. */
export function snapshotKey(
  component: string,
  scenario: string,
  snapshot: string,
  theme: Theme
) {
  return `${component}/${scenario}/${snapshot}/${theme}`
}

/** Where both runners write a snapshot, relative to `out/<runner>`. */
export function snapshotPath(
  component: string,
  scenario: string,
  snapshot: string,
  theme: Theme
) {
  return join(component, scenario, `${snapshot}-${theme}.png`)
}

/** The accepted score of every snapshot (see `baseline.json`). */
export function readBaseline(): Record<string, number> {
  const path = join(root, "baseline.json")
  return existsSync(path) ? JSON.parse(readFileSync(path, "utf8")).scores : {}
}

/** How much worse than its baseline the score of a snapshot may get. */
export function allowance(baseline: number) {
  return Math.max(0.01, baseline * 0.1)
}

/** What the Playwright runner records for each run (scenario × theme). */
export interface RunRecord {
  component: string
  scenario: string
  theme: Theme
  reference: { failures: string[] }
  iced: { status: string; failures: string[] }
  snapshots: {
    name: string
    key: string
    /** Relative to `out/{reference,iced,diff}`. */
    path: string
    score: number | null
    baseline: number | null
    reference: Box | null
    iced: Box | null
  }[]
}

/**
 * The current time of every page (`Date.now()`), so that "today" does not
 * depend on when the suite runs. iced fixtures must use the same date.
 */
export const NOW = new Date("2025-06-15T12:00:00Z")
