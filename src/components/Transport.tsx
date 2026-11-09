import { localizedName, t } from '../i18n'
import { Headphones, Mic, Pause, Play, SkipBack, Square } from 'lucide-react'
import type { StudioController } from '../useStudio'
import { formatPosition } from '../timeline/timeDisplay'
import ExportButton from './ExportButton'

type Props = Pick<
  StudioController,
  | 'position'
  | 'playing'
  | 'recording'
  | 'busy'
  | 'duration'
  | 'armedTrack'
  | 'pause'
  | 'seek'
  | 'play'
  | 'record'
  | 'inputLevel'
  | 'generating'
  | 'exportAudio'
  | 'exportVideo'
  | 'canExportVideo'
  | 'timeMode'
  | 'frameRate'
>

export default function Transport({
  position,
  playing,
  recording,
  busy,
  duration,
  armedTrack,
  pause,
  seek,
  play,
  record,
  inputLevel,
  generating,
  exportAudio,
  exportVideo,
  canExportVideo,
  timeMode,
  frameRate,
}: Props) {
  const waiting = generating && !playing && !recording
  return (
    <div className="transport" role="group" aria-label={t('transport.controls')}>
      <div className="tl-toolbar-group">
        <button
          className="icon-button"
          title={t('transport.homeTitle')}
          aria-label={t('transport.home')}
          disabled={recording || !!busy}
          onClick={() => {
            void pause()
            seek(0)
          }}
        >
          <SkipBack size={14} />
        </button>
        <button
          className="play-button"
          title={waiting ? t('transport.waitForGeneration') : t('transport.playTitle')}
          aria-label={playing ? t('transport.pause') : t('transport.play')}
          disabled={recording || !!busy || waiting}
          onClick={() => void play()}
        >
          {playing ? (
            <Pause size={15} fill="currentColor" />
          ) : (
            <Play size={15} fill="currentColor" />
          )}
        </button>
        <button
          className="icon-button"
          title={t('transport.stop')}
          aria-label={t('transport.stop')}
          disabled={!!busy}
          onClick={() => (recording ? void record() : void pause())}
        >
          <Square size={12} fill="currentColor" />
        </button>
        <span className="time-display">
          {formatPosition(position, timeMode, frameRate)}
          <span>/ {formatPosition(duration, timeMode, frameRate)}</span>
        </span>
      </div>
      <div className="tl-toolbar-group">
        <button
          className={`record-button ${recording ? 'recording' : ''}`}
          aria-label={recording ? t('transport.stopRecording') : t('transport.record')}
          title={
            waiting
              ? t('transport.waitForGeneration')
              : `${recording ? t('transport.stopRecording') : t('transport.record')} · R`
          }
          disabled={!!busy || waiting}
          onClick={() => void record()}
        >
          <span className="record-circle" />
        </button>
        <span
          className="record-target"
          title={armedTrack ? localizedName(armedTrack.name) : t('transport.armHint')}
        >
          <Mic size={13} />
          <span className="mini-meter" aria-label={t('transport.inputLevel')}>
            <i style={{ width: `${Math.min(100, inputLevel * 180)}%` }} />
          </span>
        </span>
      </div>
      <div className="tl-toolbar-group">
        <ExportButton
          exportAudio={exportAudio}
          exportVideo={exportVideo}
          canExportVideo={canExportVideo}
          busy={busy}
          recording={recording}
          duration={duration}
        />
      </div>
    </div>
  )
}

export function MonitorButton({
  monitor,
  setMonitor,
}: Pick<StudioController, 'monitor' | 'setMonitor'>) {
  return (
    <button
      className={`tl-tool ${monitor ? 'is-active' : ''}`}
      aria-label={t('transport.monitor')}
      aria-pressed={monitor}
      title={t('transport.monitorTitle')}
      onClick={() => setMonitor(!monitor)}
    >
      <Headphones size={15} />
    </button>
  )
}
