import { t } from '../i18n'
import type { TimelineProps } from './types'
import type { TimelineViewport } from './useTimelineViewport'
import { formatPosition, rulerLabel } from './timeDisplay'
type Props = Pick<
  TimelineProps,
  'recording' | 'position' | 'duration' | 'onSeek' | 'timeMode' | 'frameRate'
> & {
  viewport: TimelineViewport
}
export default function TimelineRuler({
  recording,
  position,
  duration,
  onSeek,
  timeMode,
  frameRate,
  viewport,
}: Props) {
  const { rulerRef, extent, seekAt, ticks, view, pixelsPerSecond, tickStep } = viewport
  return (
    <div className="tl-ruler-row">
      <div className="tl-ruler-heading">
        <span>{t('timeline.controls')}</span>
        <span>{t('timeline.level')}</span>
      </div>
      <div
        ref={rulerRef}
        className="tl-ruler"
        role="slider"
        tabIndex={0}
        aria-label={t('timeline.playhead')}
        aria-valuemin={0}
        aria-valuemax={Math.max(extent, position)}
        aria-valuenow={position}
        aria-valuetext={formatPosition(position, timeMode, frameRate)}
        aria-disabled={recording}
        onPointerDown={(event) => {
          if (event.button === 0 && !recording) {
            event.currentTarget.setPointerCapture(event.pointerId)
            seekAt(event.clientX)
          }
        }}
        onPointerMove={(event) => {
          if (event.currentTarget.hasPointerCapture(event.pointerId)) seekAt(event.clientX)
        }}
        onPointerUp={(event) => {
          if (event.currentTarget.hasPointerCapture(event.pointerId))
            event.currentTarget.releasePointerCapture(event.pointerId)
        }}
        onKeyDown={(event) => {
          if (recording) return
          const next = event.key === 'Home' ? 0 : event.key === 'End' ? duration : null
          if (next !== null) {
            event.preventDefault()
            event.stopPropagation()
            onSeek(Math.max(0, next))
          }
        }}
      >
        {ticks.map((tick) => (
          <span
            className="tl-ruler-tick"
            key={tick}
            style={{ left: (tick - view.start) * pixelsPerSecond }}
          >
            {rulerLabel(tick, tickStep, timeMode, frameRate)}
          </span>
        ))}
      </div>
    </div>
  )
}
