import type { Clip, MediaAsset, Track } from '../types'
import type { CommandPayloads, GenerationEvent, ScriptClipRef } from '../protocol'

export const speechWordsPerSecond = 2.5
export const minimumScriptSlot = 0.25
export const defaultScriptSlot = 4
export const overflowTolerance = 0.01
export const speechNoticeDuration = 10000

export type ScriptClip = Clip & { script: NonNullable<Clip['script']> }

export interface ActiveGeneration {
  state: 'queued' | 'downloading' | 'generating'
  progress: number
}

export type ActiveGenerations = Readonly<Record<string, ActiveGeneration>>

export type ScriptClipStatus =
  | { state: 'draft' | 'queued' | 'fits' | 'stale' }
  | { state: 'downloading' | 'generating'; progress: number }
  | { state: 'overflow'; overflowSeconds: number }

export type ScriptClipState = ScriptClipStatus['state']

export type ScriptRowAction = 'generate' | 'regenerate' | 'stop'

export type FillLevel = 'ok' | 'tight' | 'over'

export const isScriptClip = (clip: Clip): clip is ScriptClip => !!clip.script

export function estimatedSpeechSeconds(text: string, speed: number) {
  const words = text.split(/\s+/).filter(Boolean).length
  return words / speechWordsPerSecond / speed
}

export function fillRatio(
  text: string,
  boxSeconds: number,
  speed: number,
): { ratio: number; level: FillLevel } {
  const ratio = boxSeconds > 0 ? estimatedSpeechSeconds(text, speed) / boxSeconds : Infinity
  return { ratio, level: ratio < 0.85 ? 'ok' : ratio <= 1 ? 'tight' : 'over' }
}

export function scriptClipStatus(
  clip: ScriptClip,
  asset: Pick<MediaAsset, 'duration'> | undefined,
  effectiveVoice: string,
  speed: number,
  generation: ActiveGeneration | undefined,
): ScriptClipStatus {
  if (generation?.state === 'downloading' || generation?.state === 'generating')
    return { state: generation.state, progress: generation.progress }
  if (generation?.state === 'queued') return { state: 'queued' }
  const take = clip.script.take
  if (!take || !asset) return { state: 'draft' }
  if (take.text !== clip.script.text || take.voice !== effectiveVoice || take.speed !== speed)
    return { state: 'stale' }
  const overflowSeconds = asset.duration - clip.duration
  if (overflowSeconds > overflowTolerance) return { state: 'overflow', overflowSeconds }
  return { state: 'fits' }
}

export function scriptClipsInOrder(tracks: Track[]) {
  return tracks
    .flatMap((track) =>
      track.clips
        .filter(isScriptClip)
        .map((clip) => ({ trackId: track.id, locked: track.locked, clip })),
    )
    .sort((a, b) => a.clip.start - b.clip.start)
}

export function pendingScriptClips(
  tracks: Track[],
  statusOf: (clip: ScriptClip) => ScriptClipStatus,
): ScriptClipRef[] {
  return scriptClipsInOrder(tracks)
    .filter(({ locked, clip }) => {
      const { state } = statusOf(clip)
      return !locked && (state === 'draft' || state === 'stale') && clip.script.text.trim() !== ''
    })
    .map(({ trackId, clip }) => ({ trackId, clipId: clip.id }))
}

export function scriptSaveRequests(
  trackId: string,
  clip: ScriptClip,
  text: string,
  generate: boolean,
): {
  patch: CommandPayloads['script_patch'] | null
  generate: CommandPayloads['script_generate'] | null
} {
  const trimmed = singleLineText(text).trim()
  return {
    patch: trimmed !== clip.script.text ? { trackId, clipId: clip.id, text: trimmed } : null,
    generate: generate && trimmed ? { clips: [{ trackId, clipId: clip.id }] } : null,
  }
}

export function scriptDrawTarget(tracks: Track[], trackId: string) {
  const track = tracks.find((track) => track.id === trackId)
  return track && track.kind === 'audio' && !track.locked ? track : undefined
}

export function scriptRowAction(
  state: ScriptClipState,
  text: string,
  locked: boolean,
): ScriptRowAction | null {
  if (state === 'queued' || state === 'downloading' || state === 'generating') return 'stop'
  if (locked || !text.trim()) return null
  if (state === 'draft') return 'generate'
  if (state === 'stale') return 'regenerate'
  return null
}

export function findScriptClip(tracks: readonly Track[], clipId: string) {
  for (const track of tracks)
    for (const clip of track.clips) if (clip.id === clipId && isScriptClip(clip)) return clip
  return undefined
}

export const projectVoicePayload = (voice: string): CommandPayloads['project_voice'] =>
  voice ? { voice } : {}

export function withGenerationEvent(
  active: ActiveGenerations,
  event: GenerationEvent,
): ActiveGenerations {
  if (event.state === 'queued' || event.state === 'downloading' || event.state === 'generating')
    return { ...active, [event.clipId]: { state: event.state, progress: event.progress } }
  if (!active[event.clipId]) return active
  const remaining = { ...active }
  delete remaining[event.clipId]
  return remaining
}

export const isGenerating = (active: ActiveGenerations) => Object.keys(active).length > 0

export const singleLineText = (text: string) => text.replace(/[\r\n]+/g, ' ')

export function scriptDraftRange(from: number, to: number, moved: boolean) {
  if (!moved) return { position: Math.max(0, from), duration: defaultScriptSlot }
  const position = Math.max(0, Math.min(from, to))
  return { position, duration: Math.max(minimumScriptSlot, Math.max(from, to) - position) }
}
