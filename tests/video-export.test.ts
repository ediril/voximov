import { describe, expect, it } from 'vitest'
import { hasExportableVideo } from '../src/timeline/videoExport'
import { neutralEffects, type Clip, type Track, type TrackKind } from '../src/types'

const clip = (assetId: string | null, duration = 2): Clip => ({
  id: `clip-${assetId}`,
  assetId,
  name: '',
  start: 0,
  offset: 0,
  duration,
})

const track = (kind: TrackKind, clips: Clip[]): Track => ({
  id: `${kind}-${clips.length}`,
  name: `track.${kind}`,
  kind,
  color: '#fff',
  mute: false,
  solo: false,
  locked: false,
  armed: false,
  fxBypass: true,
  volume: 0,
  pan: 0,
  effects: neutralEffects,
  clips,
})

const assets = [
  { id: 'film', kind: 'video' as const },
  { id: 'music', kind: 'audio' as const },
]

describe('video export availability', () => {
  it('needs a video clip on the first video track', () => {
    expect(hasExportableVideo([track('video', [clip('film')])], assets)).toBe(true)
    expect(
      hasExportableVideo([track('voice', [clip('music')]), track('video', [clip('film')])], assets),
    ).toBe(true)
  })
  it('stays unavailable without one', () => {
    expect(hasExportableVideo([], assets)).toBe(false)
    expect(hasExportableVideo([track('video', [])], assets)).toBe(false)
    expect(hasExportableVideo([track('audio', [clip('film')])], assets)).toBe(false)
    expect(hasExportableVideo([track('video', [clip('music')])], assets)).toBe(false)
    expect(hasExportableVideo([track('video', [clip('missing')])], assets)).toBe(false)
    expect(hasExportableVideo([track('video', []), track('video', [clip('film')])], assets)).toBe(
      false,
    )
  })
})
