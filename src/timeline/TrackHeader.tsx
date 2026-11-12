import { useRef, useState } from 'react'
import { localizedName, t } from '../i18n'
import { AudioLines, LockKeyhole, Trash2, UnlockKeyhole, Video, WandSparkles } from 'lucide-react'
import type { Track } from '../types'
import TrackMeter from './TrackMeter'

const speechSpeeds = [0.8, 0.9, 1, 1.1, 1.2, 1.3, 1.5]
const maxTrackName = 120

const speedOptions = (current: number) =>
  speechSpeeds.includes(current) ? speechSpeeds : [...speechSpeeds, current].sort((a, b) => a - b)

const stopPropagation = (event: { stopPropagation: () => void }) => event.stopPropagation()

function TrackName({
  track,
  soloHint,
  onRename,
}: {
  track: Track
  soloHint?: string
  onRename: (name: string) => void
}) {
  const [editing, setEditing] = useState(false)
  const cancelled = useRef(false)
  const name = localizedName(track.name)
  if (!editing)
    return (
      <button
        className="tl-track-name"
        title={soloHint ?? t('track.renameHint', { name })}
        onDoubleClick={(event) => {
          event.stopPropagation()
          cancelled.current = false
          setEditing(true)
        }}
      >
        {name}
      </button>
    )
  return (
    <input
      className="tl-track-name-input"
      aria-label={t('track.nameLabel')}
      defaultValue={name}
      maxLength={maxTrackName}
      autoFocus
      onFocus={(event) => event.currentTarget.select()}
      onClick={stopPropagation}
      onKeyDown={(event) => {
        if (event.key === 'Enter') event.currentTarget.blur()
        if (event.key === 'Escape') {
          cancelled.current = true
          event.currentTarget.blur()
        }
      }}
      onBlur={(event) => {
        const renamed = event.currentTarget.value.trim()
        setEditing(false)
        if (!cancelled.current && renamed && renamed !== name) onRename(renamed)
      }}
    />
  )
}

interface Props {
  track: Track
  index: number
  recording: boolean
  operationPending: boolean
  onRemoveTrack: (track: Track) => void
  soloHint?: string
  onSelectTrack: (id: string) => void
  onUpdateTrack: (id: string, patch: Partial<Track>) => void
  getLevel: (id: string) => number
}
export default function TrackHeader({
  track,
  index,
  recording,
  operationPending,
  onRemoveTrack,
  soloHint,
  onSelectTrack,
  onUpdateTrack,
  getLevel,
}: Props) {
  return (
    <div className="tl-track-header" onClick={() => onSelectTrack(track.id)}>
      <div className="tl-track-heading">
        <span className="tl-track-number">{(index + 1).toString().padStart(2, '0')}</span>
        {track.kind === 'video' ? <Video size={13} /> : <AudioLines size={13} />}
        <TrackName
          track={track}
          soloHint={soloHint}
          onRename={(name) => onUpdateTrack(track.id, { name })}
        />
        <span className="tl-track-db">
          {track.volume > 0 ? '+' : ''}
          {track.volume.toFixed(1)}
        </span>
      </div>
      <div className="tl-track-bottom">
        <div className="tl-circles">
          <button
            className={`tl-circle ${track.mute ? 'is-muted' : ''}`}
            title={t('track.mute')}
            aria-label={t('track.muteLabel', { value0: localizedName(track.name) })}
            aria-pressed={track.mute}
            onClick={() => onUpdateTrack(track.id, { mute: !track.mute })}
          >
            M
          </button>
          <button
            className={`tl-circle ${track.solo ? 'is-solo' : ''}`}
            title={t('track.solo')}
            aria-label={t('track.soloLabel', { value0: localizedName(track.name) })}
            aria-pressed={track.solo}
            onClick={() => onUpdateTrack(track.id, { solo: !track.solo })}
          >
            S
          </button>
          <button
            className={`tl-circle ${track.armed ? 'is-armed' : ''}`}
            title={track.kind === 'video' ? t('track.selectAudio') : t('track.recordHere')}
            aria-label={t('track.armLabel', { value0: localizedName(track.name) })}
            aria-pressed={track.armed}
            disabled={track.kind === 'video' || recording}
            onClick={() => onUpdateTrack(track.id, { armed: !track.armed })}
          >
            R
          </button>
          <button
            className={`tl-circle ${track.locked ? 'is-active' : ''}`}
            title={track.locked ? t('track.unlock') : t('track.lock')}
            aria-label={t('track.lockLabel', { value0: localizedName(track.name) })}
            aria-pressed={track.locked}
            onClick={() => onUpdateTrack(track.id, { locked: !track.locked })}
          >
            {track.locked ? <LockKeyhole size={9} /> : <UnlockKeyhole size={9} />}
          </button>
          <button
            className={`tl-circle ${!track.fxBypass ? 'is-fx' : ''}`}
            title={track.fxBypass ? t('track.enableFx') : t('track.bypassFx')}
            aria-label={t('track.fxLabel', { value0: localizedName(track.name) })}
            aria-pressed={!track.fxBypass}
            onClick={() => onUpdateTrack(track.id, { fxBypass: !track.fxBypass })}
          >
            <WandSparkles size={9} />
          </button>
          {Array.from({ length: 7 }, (_, slot) => (
            <button
              key={slot}
              className="tl-circle tl-reserved"
              disabled
              title={t('track.reserved')}
              aria-label={t('track.reserved')}
            />
          ))}
        </div>
        <div className="tl-track-level">
          <TrackMeter id={track.id} getLevel={getLevel} />
          <div className="tl-track-footer">
            <span>
              {track.locked
                ? t('track.locked')
                : track.armed
                  ? t('track.microphone')
                  : track.kind === 'video'
                    ? t('track.videoAudio')
                    : t('track.audioTrack')}
            </span>
            {track.kind === 'audio' && (
              <select
                className="tl-track-speed"
                title={t('track.speechSpeed')}
                aria-label={t('track.speechSpeedLabel', { value0: localizedName(track.name) })}
                value={track.speechSpeed}
                disabled={recording || operationPending}
                onClick={stopPropagation}
                onChange={(event) =>
                  onUpdateTrack(track.id, { speechSpeed: Number(event.target.value) })
                }
              >
                {speedOptions(track.speechSpeed).map((speed) => (
                  <option key={speed} value={speed}>
                    {t('track.speechSpeedValue', { speed })}
                  </option>
                ))}
              </select>
            )}
            <button
              className="tl-track-delete"
              title={t('track.delete', { name: localizedName(track.name) })}
              aria-label={t('track.delete', { name: localizedName(track.name) })}
              disabled={recording || operationPending || track.locked}
              onClick={(event) => {
                event.stopPropagation()
                onRemoveTrack(track)
              }}
            >
              <Trash2 size={11} />
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
