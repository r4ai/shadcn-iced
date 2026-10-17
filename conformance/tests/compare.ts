import { readFileSync, writeFileSync, mkdirSync } from "node:fs"
import { dirname } from "node:path"

import pixelmatch from "pixelmatch"
import { PNG } from "pngjs"

export interface Box {
  x: number
  y: number
  width: number
  height: number
}

export interface Comparison {
  /**
   * The share of the content (pixels that are not background in either
   * image) that differs perceptibly, from 0 (identical) to 1.
   *
   * Anti-aliased pixels are ignored, so different text rasterizers do not
   * dominate the score.
   */
  score: number
  /** The bounding box of the content of each image, in physical pixels. */
  reference: Box | null
  iced: Box | null
}

/** Compares two PNG files and writes a diff image to `diffPath`. */
export function compare(
  referencePath: string,
  icedPath: string,
  diffPath: string
): Comparison {
  const reference = PNG.sync.read(readFileSync(referencePath))
  const iced = PNG.sync.read(readFileSync(icedPath))

  const width = Math.max(reference.width, iced.width)
  const height = Math.max(reference.height, iced.height)
  const a = pad(reference, width, height)
  const b = pad(iced, width, height)

  const diff = new PNG({ width, height })
  const mismatched = pixelmatch(a, b, diff.data, width, height, {
    threshold: 0.1,
    includeAA: false,
    alpha: 0.15,
  })

  mkdirSync(dirname(diffPath), { recursive: true })
  writeFileSync(diffPath, PNG.sync.write(diff))

  const contentA = content(a, width, height)
  const contentB = content(b, width, height)

  let union = 0
  for (let i = 0; i < contentA.mask.length; i++) {
    if (contentA.mask[i] || contentB.mask[i]) union++
  }

  return {
    score: round(union === 0 ? 0 : Math.min(1, mismatched / union)),
    reference: contentA.box,
    iced: contentB.box,
  }
}

function round(value: number) {
  return Math.round(value * 10000) / 10000
}

/** Extends an image with its background color (its top-left pixel). */
function pad(image: PNG, width: number, height: number): Uint8Array {
  if (image.width === width && image.height === height) return image.data

  const data = new Uint8Array(width * height * 4)
  const background = image.data.subarray(0, 4)

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const target = (y * width + x) * 4
      if (x < image.width && y < image.height) {
        const source = (y * image.width + x) * 4
        data.set(image.data.subarray(source, source + 4), target)
      } else {
        data.set(background, target)
      }
    }
  }

  return data
}

/** The pixels that differ from the background (the top-left pixel). */
function content(data: Uint8Array, width: number, height: number) {
  const mask = new Uint8Array(width * height)
  let minX = width
  let minY = height
  let maxX = -1
  let maxY = -1

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4
      const delta = Math.max(
        Math.abs(data[i] - data[0]),
        Math.abs(data[i + 1] - data[1]),
        Math.abs(data[i + 2] - data[2])
      )

      if (delta > 6) {
        mask[y * width + x] = 1
        minX = Math.min(minX, x)
        minY = Math.min(minY, y)
        maxX = Math.max(maxX, x)
        maxY = Math.max(maxY, y)
      }
    }
  }

  const box =
    maxX < 0
      ? null
      : { x: minX, y: minY, width: maxX - minX + 1, height: maxY - minY + 1 }

  return { mask, box }
}
