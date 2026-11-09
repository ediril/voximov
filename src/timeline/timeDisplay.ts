import { formatTime } from '../types'

export type TimeMode = 'time' | 'frames' | 'timecode'

export interface FrameRate {
  numerator: number
  denominator: number
}

export interface TimeDisplay {
  mode: TimeMode
  timeStep: number
  framesStep: number
  timecodeStep: number
}

export const defaultTimeDisplay: TimeDisplay = {
  mode: 'time',
  timeStep: 1,
  framesStep: 1,
  timecodeStep: 1,
}

export const timeSteps = [0.1, 0.5, 1, 5, 10]
export const frameSteps = [1, 5, 10, 30]
export const stepMultiplier = 10
export const sampleRate = 48000

const timeModes: TimeMode[] = ['time', 'frames', 'timecode']
const frameTolerance = 1e-6
const timeTickSteps = [
  0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 1800, 3600,
]
const secondTickMultiples = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 1800, 3600]
const minimumTickPixels = 82

export const framesPerSecond = (rate: FrameRate) => rate.numerator / rate.denominator

export const frameStart = (frame: number, rate: FrameRate) =>
  (frame * rate.denominator) / rate.numerator

export const frameAt = (seconds: number, rate: FrameRate) =>
  Math.max(0, Math.floor(seconds * framesPerSecond(rate) + frameTolerance))

export const nearestFrame = (seconds: number, rate: FrameRate) =>
  Math.round((Math.round(Math.max(0, seconds) * sampleRate) / sampleRate) * framesPerSecond(rate))

export const onSampleGrid = (seconds: number) => Math.round(seconds * sampleRate) / sampleRate

export const effectiveMode = (mode: TimeMode, rate: FrameRate | null): TimeMode =>
  rate ? mode : 'time'

export function nextMode(mode: TimeMode, rate: FrameRate | null): TimeMode {
  if (!rate) return 'time'
  return timeModes[(timeModes.indexOf(mode) + 1) % timeModes.length]
}

const pad = (value: number) => value.toString().padStart(2, '0')

export function formatTimecode(seconds: number, rate: FrameRate) {
  const nominal = Math.max(1, Math.round(framesPerSecond(rate)))
  const frame = frameAt(seconds, rate)
  const wholeSeconds = Math.floor(frame / nominal)
  const hours = Math.floor(wholeSeconds / 3600)
  const minutes = Math.floor(wholeSeconds / 60) % 60
  return `${pad(hours)}:${pad(minutes)}:${pad(wholeSeconds % 60)}:${pad(frame % nominal)}`
}

export function formatPosition(seconds: number, mode: TimeMode, rate: FrameRate | null) {
  const shown = effectiveMode(mode, rate)
  if (shown === 'frames' && rate) return frameAt(seconds, rate).toString()
  if (shown === 'timecode' && rate) return formatTimecode(seconds, rate)
  return formatTime(seconds, true)
}

export const stepOptions = (mode: TimeMode) => (mode === 'time' ? timeSteps : frameSteps)

export function stepFor(display: TimeDisplay, mode: TimeMode) {
  if (mode === 'frames') return display.framesStep
  if (mode === 'timecode') return display.timecodeStep
  return display.timeStep
}

export function withStep(display: TimeDisplay, mode: TimeMode, step: number): TimeDisplay {
  if (mode === 'frames') return { ...display, framesStep: step }
  if (mode === 'timecode') return { ...display, timecodeStep: step }
  return { ...display, timeStep: step }
}

export function steppedPosition(
  position: number,
  direction: 1 | -1,
  multiplier: number,
  display: TimeDisplay,
  rate: FrameRate | null,
) {
  const mode = effectiveMode(display.mode, rate)
  const step = stepFor(display, mode) * multiplier * direction
  if (mode === 'time' || !rate) return Math.max(0, onSampleGrid(position + step))
  return frameStart(Math.max(0, frameAt(position, rate) + step), rate)
}

export interface RulerTicks {
  step: number
  ticks: number[]
}

function frameTickStep(desiredSeconds: number, rate: FrameRate) {
  const nominal = Math.max(1, Math.round(framesPerSecond(rate)))
  const candidates = [
    ...[1, 2, 5, 10, 15].filter((frames) => frames < nominal),
    ...secondTickMultiples.map((multiple) => multiple * nominal),
  ]
  const desired = desiredSeconds * framesPerSecond(rate)
  return candidates.find((frames) => frames >= desired) ?? candidates[candidates.length - 1]
}

export function rulerTicks(
  view: { start: number; span: number },
  laneWidth: number,
  mode: TimeMode,
  rate: FrameRate | null,
): RulerTicks {
  const desired = view.span / Math.max(2, laneWidth / minimumTickPixels)
  const end = view.start + view.span
  if (effectiveMode(mode, rate) === 'time' || !rate) {
    const step = timeTickSteps.find((candidate) => candidate >= desired) ?? 3600
    const ticks: number[] = []
    for (let index = Math.ceil(view.start / step); index * step < end; index++)
      ticks.push(Math.round(index * step * 1000000) / 1000000)
    return { step, ticks }
  }
  const frames = frameTickStep(desired, rate)
  const ticks: number[] = []
  for (
    let index = Math.max(
      0,
      Math.ceil((view.start * framesPerSecond(rate)) / frames - frameTolerance),
    );
    frameStart(index * frames, rate) < end;
    index++
  )
    ticks.push(frameStart(index * frames, rate))
  return { step: frameStart(frames, rate), ticks }
}

export function rulerLabel(tick: number, step: number, mode: TimeMode, rate: FrameRate | null) {
  if (effectiveMode(mode, rate) !== 'time' && rate) return formatPosition(tick, mode, rate)
  if (step >= 1) return formatTime(tick)
  const digits = step < 0.1 ? 2 : 1
  const fraction = Math.round((tick % 1) * 10 ** digits) % 10 ** digits
  return `${formatTime(tick)}.${fraction.toString().padStart(digits, '0')}`
}
