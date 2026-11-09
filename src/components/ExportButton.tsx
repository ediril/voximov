import { useEffect, useRef } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { LogicalPosition } from '@tauri-apps/api/dpi'
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { Download } from 'lucide-react'
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
            text: t('export.mixWav'),
            action: () => void actions.current.exportAudio(null, 'wav'),
          }),
        ),
        keep(
          await MenuItem.new({
            text: t('export.mixMp3'),
            action: () => void actions.current.exportAudio(null, 'mp3'),
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
      className="icon-button export-button"
      aria-label={t('export.button')}
      aria-haspopup="menu"
      title={t('export.button')}
      disabled={disabled}
      onClick={(event) => void open(event.currentTarget)}
    >
      <Download size={15} />
    </button>
  )
}
