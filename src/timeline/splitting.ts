import type { Track } from '../types'
import { frameStart, nearestFrame, onSampleGrid, sampleRate, type FrameRate } from './timeDisplay'

const sample = 1 / sampleRate
const edgeTolerance = 1e-9

export function splitPosition(position: number, rate: FrameRate | null) {
  const snapped = rate ? frameStart(nearestFrame(position, rate), rate) : position
  return onSampleGrid(Math.max(0, snapped))
}

export function canSplitAt(tracks: Track[], position: number, rate: FrameRate | null) {
  const cut = splitPosition(position, rate)
  return tracks.some(
    (track) =>
      !track.locked &&
      track.clips.some(
        (clip) =>
          !clip.script &&
          cut - clip.start >= sample - edgeTolerance &&
          clip.duration - (cut - clip.start) >= sample - edgeTolerance,
      ),
  )
}
