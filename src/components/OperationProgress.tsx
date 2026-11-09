import type { StudioController } from '../useStudio'
import { t, translateMessage } from '../i18n'

type Props = Pick<StudioController, 'busy' | 'progress' | 'cancelVideoExport'>

export default function OperationProgress({ busy, progress, cancelVideoExport }: Props) {
  if (!busy) return null
  const label = translateMessage(progress?.label ?? busy)
  return (
    <div className="operation-progress" role="status" aria-live="polite">
      <span className="spinner" />
      <span className="notice-content">
        {label}
        {progress && ` · ${Math.floor(progress.percent)}%`}
        {progress && <progress max={100} value={progress.percent} aria-label={label} />}
      </span>
      {cancelVideoExport && (
        <button className="operation-cancel" onClick={cancelVideoExport}>
          {t('common.cancel')}
        </button>
      )}
    </div>
  )
}
