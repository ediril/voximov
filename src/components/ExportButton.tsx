import { useEffect, useRef } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { LogicalPosition } from '@tauri-apps/api/dpi'
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { Share } from 'lucide-react'
import { t } from '../i18n'
import type { StudioController } from '../useStudio'

type Props = Pick<
  StudioController,
  'exportAudio' | 'exportVideo' | 'canExportVideo' | 'busy' | 'recording' | 'duration'
>

interface NativeResource {
  close: () => Promise<void>
}

const closeAll = (resources: readonly NativeResource[]) =>
  Promise.all(resources.map((resource) => resource.close().catch(() => {})))

export default function ExportButton({
  exportAudio,
  exportVideo,
  canExportVideo,
  busy,
  recording,
  duration,
}: Props) {
  const shown = useRef<NativeResource[]>([])
  const opening = useRef(false)
  const actions = useRef({ exportAudio, exportVideo })
  useEffect(() => {
    actions.current = { exportAudio, exportVideo }
  })
  useEffect(
    () => () => {
      void closeAll(shown.current)
      shown.current = []
    },
    [],
  )
  const disabled = !!busy || recording || !duration
  const open = async (trigger: HTMLButtonElement) => {
    if (disabled || opening.current || !isTauri()) return
    opening.current = true
    const created: NativeResource[] = []
    const keep = <T extends NativeResource>(resource: T) => {
      created.push(resource)
      return resource
    }
    try {
      await closeAll(shown.current)
      shown.current = created
      const items = [
        keep(
          await MenuItem.new({
            text: t('export.video'),
            enabled: canExportVideo,
            action: () => void actions.current.exportVideo(),
          }),
        ),
        keep(await PredefinedMenuItem.new({ item: 'Separator' })),
        keep(
          await MenuItem.new({
            text: t('export.audio'),
            action: () => void actions.current.exportAudio(null),
          }),
        ),
      ]
      const menu = keep(await Menu.new({ items }))
      const bounds = trigger.getBoundingClientRect()
      await menu.popup(new LogicalPosition(bounds.left, bounds.bottom))
    } catch {
      await closeAll(created)
    } finally {
      opening.current = false
    }
  }
  return (
    <button
      className="play-button export-button"
      aria-label={t('export.button')}
      aria-haspopup="menu"
      title={t('export.button')}
      disabled={disabled}
      onClick={(event) => void open(event.currentTarget)}
    >
      <Share size={14} />
      <span>{t('export.button')}</span>
    </button>
  )
}
