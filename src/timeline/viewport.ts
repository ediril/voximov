export interface ViewWindow {
  start: number
  span: number
}
export type ScrollMode = 'pan' | 'left' | 'right'
export const minimumSpan = 0.25
export const fittedView = (duration: number) => ({
  start: 0,
  span: duration > 0 ? Math.max(minimumSpan, duration * 1.05) : 60,
})
export const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value))

export const zoomLevels = [
  0.25, 0.5, 0.75, 1, 1.5, 2, 3, 5, 7.5, 10, 15, 20, 30, 45, 60, 90, 120, 180, 300, 600, 900, 1200,
  1800, 2700, 3600, 5400, 7200,
]
const levelTolerance = 1e-6

export function steppedSpan(span: number, direction: 'in' | 'out') {
  if (direction === 'in')
    return [...zoomLevels].reverse().find((level) => level < span - levelTolerance) ?? minimumSpan
  return zoomLevels.find((level) => level > span + levelTolerance) ?? span * 1.5
}

export function zoomViewport(
  view: ViewWindow,
  targetSpan: number,
  maximumSpan: number,
  position: number,
  pointerFraction?: number,
): ViewWindow {
  const span = clamp(targetSpan, minimumSpan, maximumSpan)
  const fraction =
    pointerFraction !== undefined
      ? clamp(pointerFraction, 0, 1)
      : position >= view.start && position <= view.start + view.span
        ? (position - view.start) / view.span
        : 0.5
  const anchor = view.start + view.span * fraction
  return { start: Math.max(0, anchor - fraction * span), span }
}

export function thumbGeometry(view: ViewWindow, extent: number, width: number) {
  const thumbWidth = Math.min(width, Math.max(60, (width * view.span) / extent))
  const left = extent > view.span ? (view.start / (extent - view.span)) * (width - thumbWidth) : 0
  return { width: thumbWidth, left }
}

export function dragViewport(
  view: ViewWindow,
  extent: number,
  width: number,
  mode: ScrollMode,
  pixels: number,
): ViewWindow {
  if (mode === 'pan') {
    const thumbWidth = thumbGeometry(view, extent, width).width
    const maximum = extent - view.span
    const delta = width > thumbWidth ? (pixels / (width - thumbWidth)) * maximum : 0
    return { ...view, start: clamp(view.start + delta, 0, maximum) }
  }
  const delta = (pixels / width) * extent
  const end = view.start + view.span
  if (mode === 'left') {
    const start = clamp(view.start + delta, 0, end - minimumSpan)
    return { start, span: end - start }
  }
  return { start: view.start, span: clamp(view.span + delta, minimumSpan, extent - view.start) }
}
