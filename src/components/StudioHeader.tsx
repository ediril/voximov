import { localizedName, t } from '../i18n'
import { useEffect, useRef, useState } from 'react'
import {
  AudioLines,
  Download,
  Film,
  FolderOpen,
  Keyboard,
  Menu,
  Plus,
  Save,
  Settings,
  X,
} from 'lucide-react'
import type { StudioController } from '../useStudio'

type Props = Pick<
  StudioController,
  | 'recording'
  | 'busy'
  | 'importNative'
  | 'duration'
  | 'selectedTrack'
  | 'saveProject'
  | 'exportAudio'
  | 'exportVideo'
  | 'canExportVideo'
  | 'openNative'
  | 'snapshot'
  | 'setHelp'
  | 'setSettings'
  | 'chooseWorkingDirectory'
>

export default function StudioHeader({
  recording,
  busy,
  importNative,
  duration,
  selectedTrack,
  saveProject,
  exportAudio,
  exportVideo,
  canExportVideo,
  openNative,
  snapshot,
  setHelp,
  setSettings,
  chooseWorkingDirectory,
}: Props) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!open) return
    const outside = (event: PointerEvent) => {
      if (!ref.current?.contains(event.target as Node)) setOpen(false)
    }
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false)
    }
    document.addEventListener('pointerdown', outside)
    document.addEventListener('keydown', escape)
    return () => {
      document.removeEventListener('pointerdown', outside)
      document.removeEventListener('keydown', escape)
    }
  }, [open])
  const run = (action: () => unknown) => {
    setOpen(false)
    void action()
  }
  const disabled = recording || !!busy
  return (
    <div className="studio-menu" ref={ref}>
      <button
        className="menu-toggle"
        aria-label={t('menu.project')}
        aria-expanded={open}
        aria-controls="project-menu"
        onClick={() => setOpen(!open)}
      >
        {open ? <X size={17} /> : <Menu size={17} />}
        {snapshot?.config.dirty && <span className="unsaved-dot" />}
      </button>
      {open && (
        <nav id="project-menu" className="project-menu" aria-label={t('menu.actions')}>
          <div className="menu-brand">
            <AudioLines size={21} />
            <strong>Voximov</strong>
          </div>
          <p className="menu-state">
            {snapshot?.config.dirty ? t('project.unsaved') : t('project.saved')}
          </p>
          <button disabled={disabled} onClick={() => run(importNative)}>
            <Plus size={16} />
            {t('media.add')}{' '}
          </button>
          <button disabled={disabled} onClick={() => run(openNative)}>
            <FolderOpen size={16} />
            {t('project.open')}{' '}
          </button>
          <button disabled={disabled} onClick={() => run(() => saveProject())}>
            <Save size={16} />
            {t('project.save')}
            <kbd>Ctrl S</kbd>
          </button>
          <button disabled={disabled} onClick={() => run(() => saveProject(true))}>
            <Save size={16} />
            {t('project.saveAs')}{' '}
          </button>
          <button disabled={disabled} onClick={() => run(chooseWorkingDirectory)}>
            <FolderOpen size={16} />
            {t('project.workingDirectory')}{' '}
          </button>
          <button disabled={disabled || !canExportVideo} onClick={() => run(exportVideo)}>
            <Film size={16} />
            {t('export.video')}
          </button>
          <button disabled={disabled || !duration} onClick={() => run(() => exportAudio())}>
            <Download size={16} />
            {t('export.audio')}
          </button>
          {selectedTrack && (
            <>
              <span className="menu-section">
                {t('export.trackPrefix')} {localizedName(selectedTrack.name)}
              </span>
              <button
                disabled={disabled || !selectedTrack.clips.length}
                onClick={() => run(() => exportAudio(selectedTrack.id))}
              >
                <Download size={16} />
                {t('export.track')}
              </button>
            </>
          )}
          <button onClick={() => run(() => setSettings(true))}>
            <Settings size={16} />
            {t('settings.title')}
          </button>
          <button onClick={() => run(() => setHelp(true))}>
            <Keyboard size={16} />
            {t('help.menu')}{' '}
          </button>
        </nav>
      )}
    </div>
  )
}
