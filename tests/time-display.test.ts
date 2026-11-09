import { describe, expect, it } from 'vitest'
import {
  defaultTimeDisplay,
  effectiveMode,
  formatPosition,
  formatTimecode,
  frameAt,
  frameStart,
  nextMode,
  rulerLabel,
  rulerTicks,
  steppedPosition,
  withStep,
  type FrameRate,
} from '../src/timeline/timeDisplay'
import { canSplitAt, splitPosition } from '../src/timeline/splitting'
import { formatTime, neutralEffects, type Clip, type Track } from '../src/types'

const fps30: FrameRate = { numerator: 30, denominator: 1 }
const fps25: FrameRate = { numerator: 25, denominator: 1 }
const ntsc: FrameRate = { numerator: 30000, denominator: 1001 }

describe('time display formatting', () => {
  it('keeps the existing clock format in time mode and without a frame rate', () => {
    expect(formatPosition(83.456, 'time', fps30)).toBe(formatTime(83.456, true))
    expect(formatPosition(83.456, 'frames', null)).toBe(formatTime(83.456, true))
    expect(formatPosition(83.456, 'timecode', null)).toBe(formatTime(83.456, true))
  })
  it('counts frames from zero and shows the frame that contains the position', () => {
    expect(formatPosition(0, 'frames', fps30)).toBe('0')
    expect(formatPosition(1, 'frames', fps30)).toBe('30')
    expect(formatPosition(1.999, 'frames', fps30)).toBe('59')
    expect(formatPosition(frameStart(7, ntsc), 'frames', ntsc)).toBe('7')
    expect(formatPosition(frameStart(7, ntsc) - 0.0001, 'frames', ntsc)).toBe('6')
  })
  it('formats timecode at 30 and 25 fps without rounding up into the next frame', () => {
    expect(formatTimecode(0, fps30)).toBe('00:00:00:00')
    expect(formatTimecode(1.999, fps30)).toBe('00:00:01:29')
    expect(formatTimecode(2, fps30)).toBe('00:00:02:00')
    expect(formatTimecode(61.5, fps30)).toBe('00:01:01:15')
    expect(formatTimecode(1.999, fps25)).toBe('00:00:01:24')
    expect(formatTimecode(3600 + 0.04, fps25)).toBe('01:00:00:01')
    expect(formatTimecode(0.1, fps25)).toBe('00:00:00:02')
  })
  it('counts 29.97 fps timecode in whole frames at a nominal 30 fps without drop frames', () => {
    expect(formatTimecode(frameStart(29, ntsc), ntsc)).toBe('00:00:00:29')
    expect(formatTimecode(frameStart(30, ntsc), ntsc)).toBe('00:00:01:00')
    expect(formatTimecode(1, ntsc)).toBe('00:00:00:29')
    expect(formatTimecode(3600, ntsc)).toBe('00:59:56:12')
    expect(formatTimecode(frameStart(108000, ntsc), ntsc)).toBe('01:00:00:00')
  })
  it('handles very long positions', () => {
    expect(formatTimecode(2_000_000, fps30)).toBe('555:33:20:00')
    expect(formatPosition(2_000_000, 'frames', fps25)).toBe('50000000')
    expect(formatTimecode(frameStart(59_999_999, fps30), fps30)).toBe('555:33:19:29')
    expect(formatPosition(-3, 'frames', fps30)).toBe('0')
  })
})

describe('time display modes', () => {
  it('cycles through every mode only when the project has a frame rate', () => {
    expect(nextMode('time', fps25)).toBe('frames')
    expect(nextMode('frames', fps25)).toBe('timecode')
    expect(nextMode('timecode', fps25)).toBe('time')
    expect(nextMode('time', null)).toBe('time')
    expect(nextMode('frames', null)).toBe('time')
    expect(effectiveMode('timecode', null)).toBe('time')
    expect(effectiveMode('timecode', ntsc)).toBe('timecode')
  })
  it("remembers each mode's step separately", () => {
    const display = withStep(withStep(defaultTimeDisplay, 'frames', 10), 'time', 0.5)
    expect(display).toEqual({ mode: 'time', timeStep: 0.5, framesStep: 10, timecodeStep: 1 })
  })
})

describe('arrow-key stepping', () => {
  it('moves by seconds in time mode and stops at zero', () => {
    const display = { ...defaultTimeDisplay, timeStep: 0.1 }
    expect(steppedPosition(0.7, -1, 1, display, fps30)).toBeCloseTo(0.6, 9)
    expect(formatTime(steppedPosition(0.7, -1, 1, display, fps30), true)).toBe('00:00.600')
    expect(steppedPosition(2, 1, 10, display, null)).toBe(3)
    expect(steppedPosition(0.05, -1, 1, display, fps30)).toBe(0)
  })
  it('lands exactly on frame boundaries in frame modes', () => {
    const frames = { ...defaultTimeDisplay, mode: 'frames' as const, framesStep: 5 }
    expect(steppedPosition(0.51, 1, 1, frames, fps30)).toBe(frameStart(20, fps30))
    expect(steppedPosition(0.51, -1, 1, frames, fps30)).toBe(frameStart(10, fps30))
    expect(steppedPosition(1, 1, 10, frames, ntsc)).toBe(frameStart(29 + 50, ntsc))
    expect(steppedPosition(0.1, -1, 10, frames, ntsc)).toBe(0)
    const timecode = { ...defaultTimeDisplay, mode: 'timecode' as const }
    let position = 0.37
    for (let press = 0; press < 100; press++) {
      position = steppedPosition(position, 1, 1, timecode, ntsc)
      expect(position).toBe(frameStart(frameAt(position, ntsc), ntsc))
    }
    expect(frameAt(position, ntsc)).toBe(111)
  })
  it('falls back to time steps when frame modes are unavailable', () => {
    const frames = { ...defaultTimeDisplay, mode: 'frames' as const, timeStep: 5 }
    expect(steppedPosition(1, 1, 1, frames, null)).toBe(6)
  })
})

describe('ruler ticks', () => {
  it('keeps the existing second steps in time mode', () => {
    const { step, ticks } = rulerTicks({ start: 0, span: 60 }, 820, 'time', fps30)
    expect(step).toBe(10)
    expect(ticks).toEqual([0, 10, 20, 30, 40, 50])
    expect(rulerLabel(1.5, 0.5, 'time', fps30)).toBe('00:01.5')
  })
  it('places frame-mode ticks on frame boundaries and labels them in the chosen unit', () => {
    const { step, ticks } = rulerTicks({ start: 1.01, span: 2 }, 820, 'timecode', ntsc)
    expect(step).toBeCloseTo(frameStart(10, ntsc), 12)
    expect(ticks[0]).toBe(frameStart(40, ntsc))
    expect(ticks.every((tick) => Number.isInteger(Math.round(tick * 30000) / 1001))).toBe(true)
    expect(rulerLabel(ticks[0], step, 'timecode', ntsc)).toBe('00:00:01:10')
    expect(rulerLabel(ticks[1], step, 'frames', ntsc)).toBe('50')
    const wide = rulerTicks({ start: 0, span: 600 }, 820, 'frames', fps25)
    expect(wide.step).toBe(60)
    expect(wide.ticks.slice(0, 3)).toEqual([0, 60, 120])
  })
})

const clip = (id: string, start: number, duration: number, script = false): Clip => ({
  id,
  assetId: script ? null : 'source',
  name: id,
  start,
  offset: 0,
  duration,
  ...(script ? { script: { text: 'Hello' } } : {}),
})

const track = (clips: Clip[], locked = false): Track => ({
  id: `track-${clips.map((entry) => entry.id).join('-')}`,
  name: 'track.audio',
  kind: 'audio',
  color: '#fff',
  mute: false,
  solo: false,
  locked,
  armed: false,
  fxBypass: true,
  volume: 0,
  pan: 0,
  effects: neutralEffects,
  clips,
})

describe('split at the playhead', () => {
  it('snaps to the nearest frame and then to the sample grid', () => {
    expect(splitPosition(4.01, null)).toBe(Math.round(4.01 * 48000) / 48000)
    expect(splitPosition(4.01, ntsc)).toBe(Math.round((120 * 1001 * 48000) / 30000) / 48000)
    expect(splitPosition(-1, fps30)).toBe(0)
  })
  it('is available only when an unlocked, non-script clip crosses the playhead', () => {
    const tracks = [track([clip('locked', 0, 10)], true), track([clip('script', 0, 10, true)])]
    expect(canSplitAt(tracks, 5, null)).toBe(false)
    const audio = track([clip('audio', 2, 4)])
    expect(canSplitAt([...tracks, audio], 5, null)).toBe(true)
    expect(canSplitAt([audio], 2, null)).toBe(false)
    expect(canSplitAt([audio], 6, null)).toBe(false)
    expect(canSplitAt([audio], 2 + 1 / 48000, null)).toBe(true)
    expect(canSplitAt([audio], 2.01, fps30)).toBe(false)
    expect(canSplitAt([audio], 2.02, fps30)).toBe(true)
  })
})
