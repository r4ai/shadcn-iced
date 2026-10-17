import { readFileSync, writeFileSync, mkdirSync } from "node:fs"
import { dirname } from "node:path"

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
   * A pixel only counts as different when no pixel of the other image within
   * `TOLERANCE` physical pixels has its color (in both directions), so
   * sub-pixel offsets and anti-aliasing (text rasterizers, hinting, GPU or
   * operating system differences) do not count, while any difference of a
   * logical pixel or more in size, position or color does.
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
  const mismatched = difference(a, b, diff.data, width, height)

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

/** How far (in physical pixels) a matching pixel may be. */
const TOLERANCE = 1

/** How far outside the range of its neighbors an edge pixel may be. */
const EDGE_SLACK = 12

/** The maximum perceptual color delta of matching pixels (YIQ, as pixelmatch with a threshold of 0.1). */
const MAX_DELTA = 35215 * 0.1 * 0.1

/**
 * Counts the pixels that differ, and paints them in red over a faded copy of
 * `a` into `output` (tolerated differences in yellow).
 */
function difference(
  a: Uint8Array,
  b: Uint8Array,
  output: Uint8Array,
  width: number,
  height: number
) {
  let count = 0

  const near = (
    from: Uint8Array,
    i: number,
    to: Uint8Array,
    x: number,
    y: number
  ) => {
    // A pixel matches when its color is close to a neighbor's, or lies
    // between the colors of its neighbors (a partially covered edge pixel).
    const min = [255, 255, 255]
    const max = [0, 0, 0]

    for (let dy = -TOLERANCE; dy <= TOLERANCE; dy++) {
      for (let dx = -TOLERANCE; dx <= TOLERANCE; dx++) {
        const nx = x + dx
        const ny = y + dy
        if (nx < 0 || ny < 0 || nx >= width || ny >= height) continue
        const j = (ny * width + nx) * 4
        if (delta(from, i, to, j) <= MAX_DELTA) return true
        for (let c = 0; c < 3; c++) {
          min[c] = Math.min(min[c], to[j + c])
          max[c] = Math.max(max[c], to[j + c])
        }
      }
    }

    return [0, 1, 2].every(
      (c) =>
        from[i + c] >= min[c] - EDGE_SLACK && from[i + c] <= max[c] + EDGE_SLACK
    )
  }

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4
      let color: [number, number, number]

      if (delta(a, i, b, i) <= MAX_DELTA) {
        // Faded grayscale of the reference.
        const gray =
          255 -
          0.1 * (255 - (a[i] * 0.299 + a[i + 1] * 0.587 + a[i + 2] * 0.114))
        color = [gray, gray, gray]
      } else if (near(a, i, b, x, y) && near(b, i, a, x, y)) {
        color = [255, 200, 0]
      } else {
        count++
        color = [255, 0, 0]
      }

      output[i] = color[0]
      output[i + 1] = color[1]
      output[i + 2] = color[2]
      output[i + 3] = 255
    }
  }

  return count
}

/** The perceptual delta between two opaque pixels (YIQ). */
function delta(a: Uint8Array, i: number, b: Uint8Array, j: number) {
  const dr = a[i] - b[j]
  const dg = a[i + 1] - b[j + 1]
  const db = a[i + 2] - b[j + 2]

  const y = dr * 0.29889531 + dg * 0.58662247 + db * 0.11448223
  const iq = dr * 0.59597799 - dg * 0.2741761 - db * 0.32180189
  const q = dr * 0.21147017 - dg * 0.52261711 + db * 0.31114694

  return 0.5053 * y * y + 0.299 * iq * iq + 0.1957 * q * q
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
