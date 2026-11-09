import type { MediaAsset, Track } from '../types'

export function hasExportableVideo(tracks: Track[], assets: Pick<MediaAsset, 'id' | 'kind'>[]) {
  const videoAssets = new Set(
    assets.filter((asset) => asset.kind === 'video').map((asset) => asset.id),
  )
  const videoTrack = tracks.find((track) => track.kind === 'video')
  return !!videoTrack?.clips.some(
    (clip) => clip.duration > 0 && !!clip.assetId && videoAssets.has(clip.assetId),
  )
}
