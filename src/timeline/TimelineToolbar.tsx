import { t, type TranslationKey } from '../i18n'
import {
  Copy,
  ClipboardPaste,
  Maximize2,
  Magnet,
  Minus,
  MousePointer2,
  Plus,
  Scissors,
  TextCursorInput,
  Trash2,
} from 'lucide-react'
import type { TimelineProps } from './types'
import type { TimelineViewport } from './useTimelineViewport'
import { fittedView, minimumSpan, steppedSpan } from './viewport'
import { stepFor, stepOptions, type TimeMode } from './timeDisplay'

type Props = Pick<
  TimelineProps,
  | 'transport'
  | 'monitor'
  | 'tool'
  | 'onToolChange'
  | 'snapping'
  | 'onSnappingChange'
  | 'recording'
  | 'selectedClipId'
  | 'canSplit'
  | 'onSplitAtPlayhead'
  | 'onRemoveClip'
  | 'duration'
  | 'onAddTrack'
  | 'onCopy'
  | 'onPaste'
  | 'canPaste'
  | 'selection'
  | 'operationPending'
  | 'timeDisplay'
  | 'timeMode'
  | 'frameRate'
  | 'onCycleTimeMode'
  | 'onTimeStepChange'
> & { viewport: TimelineViewport; canDelete: boolean }

const modeLabels: Record<TimeMode, TranslationKey> = {
  time: 'timeDisplay.time',
  frames: 'timeDisplay.frames',
  timecode: 'timeDisplay.timecode',
}

const stepLabel = (mode: TimeMode, step: number) =>
  mode === 'time'
    ? t('units.secondsValue', { value0: step })
    : t('units.framesValue', { value0: step })

export default function TimelineToolbar({
  transport,
  monitor,
  tool,
  onToolChange,
  snapping,
  onSnappingChange,
  recording,
  selectedClipId,
  canSplit,
  onSplitAtPlayhead,
  onRemoveClip,
  duration,
  onAddTrack,
  viewport,
  canDelete,
  onCopy,
  onPaste,
  canPaste,
  selection,
  operationPending,
  timeDisplay,
  timeMode,
  frameRate,
  onCycleTimeMode,
  onTimeStepChange,
}: Props) {
  const { view, setView, zoom } = viewport
  const modeName = t(modeLabels[timeMode])
  const step = stepFor(timeDisplay, timeMode)
  return (
    <div className="tl-toolbar">
      <div className="tl-toolbar-group">
        <button
          className={`tl-tool ${tool === 'select' ? 'is-active' : ''}`}
          title={t('tools.selectTitle')}
          aria-label={t('tools.selectMove')}
          aria-pressed={tool === 'select'}
          onClick={() => onToolChange('select')}
        >
          <MousePointer2 size={15} />
        </button>
        <button
          className={`tl-tool ${tool === 'script' ? 'is-active' : ''}`}
          title={t('tools.scriptTitle')}
          aria-label={t('tools.script')}
          aria-pressed={tool === 'script'}
          onClick={() => onToolChange('script')}
          disabled={recording}
        >
          <TextCursorInput size={15} />
        </button>
      </div>
      <div className="tl-toolbar-group">
        <button
          className="tl-tool"
          title={t('tools.splitAtPlayheadTitle')}
          aria-label={t('tools.splitAtPlayhead')}
          disabled={!canSplit}
          onClick={onSplitAtPlayhead}
        >
          <Scissors size={14} />
        </button>
        <button
          className="tl-tool"
          title={t('tools.deleteTitle')}
          aria-label={t('tools.delete')}
          disabled={!canDelete}
          onClick={onRemoveClip}
        >
          <Trash2 size={14} />
        </button>
        <button
          className="tl-tool"
          title={t('selection.copy')}
          aria-label={t('selection.copy')}
          disabled={
            recording || operationPending || (!selectedClipId && !selection?.regions.length)
          }
          onClick={onCopy}
        >
          <Copy size={13} />
        </button>
        <button
          className="tl-tool"
          title={t('selection.paste')}
          aria-label={t('selection.paste')}
          disabled={recording || operationPending || !canPaste}
          onClick={onPaste}
        >
          <ClipboardPaste size={13} />
        </button>
      </div>
      {transport}
      <span className="tl-toolbar-spacer" />
      <div className="tl-toolbar-group">
        <button
          className="tl-time-mode"
          title={
            frameRate
              ? t('timeDisplay.cycleTitle', { mode: modeName })
              : t('timeDisplay.needsVideo')
          }
          aria-label={t('timeDisplay.cycleTitle', { mode: modeName })}
          disabled={!frameRate || operationPending}
          onClick={onCycleTimeMode}
        >
          {modeName}
        </button>
        <select
          className="tl-step"
          title={t('timeDisplay.stepTitle')}
          aria-label={t('timeDisplay.step')}
          value={step}
          disabled={operationPending}
          onChange={(event) => {
            onTimeStepChange(Number(event.target.value))
            event.currentTarget.blur()
          }}
        >
          {stepOptions(timeMode).map((option) => (
            <option key={option} value={option}>
              {stepLabel(timeMode, option)}
            </option>
          ))}
        </select>
      </div>
      <div className="tl-toolbar-group">
        <button
          className={`tl-tool ${snapping ? 'is-active' : ''}`}
          title={`${t('tools.snap')} · ${snapping ? t('common.on') : t('common.off')}`}
          aria-label={t('tools.snap')}
          aria-pressed={snapping}
          onClick={() => onSnappingChange(!snapping)}
        >
          <Magnet size={15} />
        </button>
        {monitor}
      </div>
      <div className="tl-toolbar-group">
        <span className="tl-zoom-label">
          {view.span < 60
            ? t('units.secondsValue', { value0: Number(view.span.toPrecision(2)) })
            : t('units.minutesValue', { value0: Math.round((view.span / 60) * 10) / 10 })}
        </span>
        <button
          className="tl-tool"
          title={t('zoom.outTitle')}
          aria-label={t('zoom.out')}
          onClick={() => zoom((span) => steppedSpan(span, 'out'))}
        >
          <Minus size={15} />
        </button>
        <button
          className="tl-tool"
          title={t('zoom.inTitle')}
          aria-label={t('zoom.in')}
          disabled={view.span <= minimumSpan}
          onClick={() => zoom((span) => steppedSpan(span, 'in'))}
        >
          <Plus size={15} />
        </button>
        <button
          className="tl-tool"
          title={t('zoom.fit')}
          aria-label={t('zoom.fit')}
          onClick={() => setView(fittedView(duration))}
        >
          <Maximize2 size={14} />
        </button>
      </div>
      <div className="tl-toolbar-group">
        <button className="tl-add-track" onClick={onAddTrack} disabled={recording}>
          <Plus size={14} /> {t('timeline.addTrack')}{' '}
        </button>
      </div>
    </div>
  )
}
