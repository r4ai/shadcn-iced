/**
 * Runs every scenario against shadcn/ui and compares it with the snapshots
 * and results of the iced runner (`cargo run -p conformance`).
 *
 * A run (scenario × theme) fails when:
 * - an expectation does not hold for shadcn/ui (the scenario is wrong, or
 *   shadcn/ui changed),
 * - an expectation does not hold for shadcn_iced, or it could not run,
 * - a snapshot differs more from shadcn/ui than recorded in `baseline.json`.
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"

import { expect, test, type CDPSession, type Page } from "@playwright/test"

import { compare } from "./compare.ts"
import {
  DEFAULT_VIEWPORT,
  NOW,
  THEMES,
  allowance,
  out,
  readBaseline,
  runKey,
  readScenarios,
  snapshotKey,
  snapshotPath,
  steps,
  type Locator,
  type RunRecord,
} from "./spec.ts"

const baseline = readBaseline()

/** The outcome of the iced runner for a run, if any. */
function icedOutcome(
  component: string,
  scenario: string,
  theme: string
): IcedOutcome | undefined {
  const path = join(
    out,
    "iced/runs",
    `${component}--${scenario}--${theme}.json`
  )
  return existsSync(path) ? JSON.parse(readFileSync(path, "utf8")) : undefined
}

interface IcedOutcome {
  status: "ok" | "missing" | "error"
  errors: string[]
  checks: { step: number; text: string; expected: string; ok: boolean }[]
  snapshots: string[]
}

/**
 * Takes screenshots until two consecutive ones are identical: some
 * animations are driven by JavaScript (e.g. Embla carousels), which CSS
 * cannot disable.
 */
async function stableScreenshot(page: Page) {
  const take = () => page.screenshot({ animations: "disabled", caret: "hide" })
  let previous = await take()

  for (let attempt = 0; attempt < 40; attempt++) {
    await page.waitForTimeout(50)
    const next = await take()
    if (next.equals(previous)) return next
    previous = next
  }

  throw new Error("the page never stops changing")
}

/** The part of a CDP `DOM.Node` that {@link systemFonts} reads. */
interface DomNode {
  nodeId: number
  nodeType: number
  nodeValue: string
  children?: DomNode[]
  shadowRoots?: DomNode[]
  contentDocument?: DomNode
}

/**
 * The fonts of the system that render some text of the page: none should,
 * since the iced runner has only the bundled fonts.
 */
async function systemFonts(cdp: CDPSession) {
  const { root } = await cdp.send("DOM.getDocument", {
    depth: -1,
    pierce: true,
  })
  const elements: number[] = []
  const visit = (node: DomNode) => {
    if (
      node.children?.some(
        (child) => child.nodeType === 3 && child.nodeValue.trim() !== ""
      )
    ) {
      elements.push(node.nodeId)
    }
    for (const child of [
      ...(node.children ?? []),
      ...(node.shadowRoots ?? []),
      ...(node.contentDocument ? [node.contentDocument] : []),
    ]) {
      visit(child)
    }
  }
  visit(root)

  const fonts = new Set<string>()
  for (const nodeId of elements) {
    const result = await cdp
      .send("CSS.getPlatformFontsForNode", { nodeId })
      .catch(() => ({ fonts: [] }))
    for (const font of result.fonts) {
      if (!font.isCustomFont) fonts.add(font.familyName)
    }
  }
  return [...fonts]
}

async function locate(page: Page, locator: Locator) {
  if (typeof locator === "object" && "at" in locator) {
    return { x: locator.at[0], y: locator.at[1] }
  }

  const {
    text,
    nth = 0,
    exact = true,
  } = typeof locator === "string" ? { text: locator } : locator

  const element = page
    .getByText(text, { exact })
    .filter({ visible: true })
    .nth(nth)

  const box = await element.boundingBox({ timeout: 2000 }).catch(() => null)

  if (!box) {
    throw new Error(`text ${JSON.stringify(text)} (#${nth}) not found`)
  }

  return { x: box.x + box.width / 2, y: box.y + box.height / 2 }
}

for (const file of readScenarios()) {
  test.describe(file.component, () => {
    for (const scenario of file.scenarios) {
      for (const theme of scenario.themes ?? THEMES) {
        test(`${scenario.id} [${theme}]`, async ({ page }) => {
          const [width, height] = scenario.viewport ?? DEFAULT_VIEWPORT
          const iced = icedOutcome(file.component, scenario.id, theme)

          const record: RunRecord = {
            component: file.component,
            scenario: scenario.id,
            theme,
            reference: { failures: [] },
            iced: {
              status: iced?.status ?? "not run",
              failures: [
                ...(iced?.status === "ok"
                  ? []
                  : [`status: ${iced?.status ?? "not run"}`]),
                ...(iced?.errors ?? []),
                ...(iced?.checks ?? [])
                  .filter((check) => !check.ok)
                  .map(
                    (check) =>
                      `step ${check.step}: ${JSON.stringify(check.text)} should be ${check.expected}`
                  ),
              ],
              known:
                baseline.knownFailures[
                  runKey(file.component, scenario.id, theme)
                ] ?? [],
            },
            snapshots: [],
          }

          try {
            await page.setViewportSize({ width, height })
            // "Today" is fixed: iced fixtures use the same date.
            await page.clock.setFixedTime(NOW)
            await page.goto(
              `/?component=${file.component}&scenario=${scenario.id}&theme=${theme}&static`
            )
            await expect(page.locator("#missing")).toHaveCount(0)
            await page.evaluate(() => document.fonts.ready)
            // Text must never fall back to a font of the system.
            expect(
              await page.evaluate(() =>
                document.fonts.check(`14px "Geist Variable"`)
              ),
              "Geist is loaded"
            ).toBe(true)
            const cdp = await page.context().newCDPSession(page)
            await cdp.send("DOM.enable")
            await cdp.send("CSS.enable")

            for (const [index, step] of steps(scenario).entries()) {
              if ("snapshot" in step) {
                const path = snapshotPath(
                  file.component,
                  scenario.id,
                  step.snapshot,
                  theme
                )
                const referencePath = join(out, "reference", path)
                mkdirSync(dirname(referencePath), { recursive: true })
                // Fonts of overlays load when they are first shown.
                await page.evaluate(() => document.fonts.ready)
                writeFileSync(referencePath, await stableScreenshot(page))

                const fallbacks = await systemFonts(cdp)
                if (fallbacks.length > 0) {
                  record.reference.failures.push(
                    `snapshot ${step.snapshot}: text falls back to fonts of the system (${fallbacks.join(", ")})`
                  )
                }

                const key = snapshotKey(
                  file.component,
                  scenario.id,
                  step.snapshot,
                  theme
                )
                const icedPath = join(out, "iced", path)
                const comparison = existsSync(icedPath)
                  ? compare(referencePath, icedPath, join(out, "diff", path))
                  : null

                record.snapshots.push({
                  name: step.snapshot,
                  key,
                  path: path.replaceAll("\\", "/"),
                  score: comparison?.score ?? null,
                  baseline: baseline.scores[key] ?? null,
                  reference: comparison?.reference ?? null,
                  iced: comparison?.iced ?? null,
                })
              } else if ("hover" in step) {
                const { x, y } = await locate(page, step.hover)
                await page.mouse.move(x, y, { steps: step.steps ?? 1 })
              } else if ("rightClick" in step) {
                const { x, y } = await locate(page, step.rightClick)
                await page.mouse.move(x, y)
                await page.mouse.down({ button: "right" })
                await page.mouse.up({ button: "right" })
              } else if ("click" in step) {
                const { x, y } = await locate(page, step.click)
                await page.mouse.move(x, y)
                await page.mouse.down()
                await page.mouse.up()
              } else if ("drag" in step) {
                const from = await locate(page, step.drag.from)
                const to = await locate(page, step.drag.to)
                await page.mouse.move(from.x, from.y)
                await page.mouse.down()
                await page.mouse.move(to.x, to.y, { steps: 5 })
                await page.mouse.up()
              } else if ("press" in step) {
                await page.keyboard.press(step.press)
              } else if ("type" in step) {
                await page.keyboard.type(step.type)
              } else if ("wait" in step) {
                await page.waitForTimeout(step.wait)
              } else if ("expect" in step) {
                for (const text of step.expect.visible ?? []) {
                  const visible = page
                    .getByText(text, { exact: true })
                    .filter({ visible: true })
                  await expect(visible.first())
                    .toBeVisible({ timeout: 2000 })
                    .catch(() =>
                      record.reference.failures.push(
                        `step ${index}: ${JSON.stringify(text)} should be visible`
                      )
                    )
                }

                for (const text of step.expect.hidden ?? []) {
                  const visible = page
                    .getByText(text, { exact: true })
                    .filter({ visible: true })
                  await expect(visible)
                    .toHaveCount(0, { timeout: 2000 })
                    .catch(() =>
                      record.reference.failures.push(
                        `step ${index}: ${JSON.stringify(text)} should be hidden`
                      )
                    )
                }
              }
            }
          } catch (error) {
            record.reference.failures.push(String(error))
          } finally {
            const recordPath = join(
              out,
              "runs",
              `${file.component}--${scenario.id}--${theme}.json`
            )
            mkdirSync(dirname(recordPath), { recursive: true })
            writeFileSync(recordPath, JSON.stringify(record, null, 2))
          }

          expect
            .soft(record.reference.failures, "shadcn/ui expectations")
            .toEqual([])
          // Known failures are accepted until they are fixed; then they must
          // leave the baseline, so that they cannot come back unnoticed.
          const { failures, known } = record.iced
          expect
            .soft(
              failures.filter((failure) => !known.includes(failure)),
              "new shadcn_iced failures"
            )
            .toEqual([])
          expect
            .soft(
              known.filter((failure) => !failures.includes(failure)),
              "fixed shadcn_iced failures (run `pnpm baseline`)"
            )
            .toEqual([])

          for (const snapshot of record.snapshots) {
            if (snapshot.score === null) continue

            if (snapshot.baseline === null) {
              test.info().annotations.push({
                type: "no baseline",
                description: `${snapshot.key}: ${snapshot.score}`,
              })
              // New scenarios must come with their baseline on CI.
              if (process.env.CI) {
                expect
                  .soft(snapshot.baseline, `${snapshot.key} has a baseline`)
                  .not.toBeNull()
              }
              continue
            }

            expect
              .soft(
                snapshot.score,
                `${snapshot.key} differs from shadcn/ui at most as much as its baseline`
              )
              .toBeLessThanOrEqual(
                snapshot.baseline + allowance(snapshot.baseline)
              )
          }
        })
      }
    }
  })
}
