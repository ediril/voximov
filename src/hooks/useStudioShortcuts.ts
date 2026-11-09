import { useEffect } from 'react'
import { repeatableShortcuts, shortcutFor, targetOwnsKey } from '../shortcuts'
import type { TimelineTool } from '../timeline/types'
import { stepMultiplier } from '../timeline/timeDisplay'

interface Actions {
  play: () => Promise<void>
  record: () => Promise<void>
  saveProject: (saveAs?: boolean) => Promise<boolean>
  undo: () => void
  redo: () => void
  removeClip: () => void
  copyClips: () => void
  pasteClips: () => Promise<void>
  seek: (position: number) => void
  setTool: (tool: TimelineTool) => void
  splitAtPlayhead: () => void
  cycleTimeMode: () => void
  stepPlayhead: (direction: 1 | -1, multiplier: number) => void
  blocked: boolean
}

export function useStudioShortcuts(actions: Actions) {
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (actions.blocked || event.defaultPrevented || targetOwnsKey(event.target, event.code))
        return
      const shortcut = shortcutFor(event)
      if (!shortcut) return
      if (shortcut === 'zoomIn' || shortcut === 'zoomOut') return
      event.preventDefault()
      if (event.repeat && !repeatableShortcuts.includes(shortcut)) return
      const multiplier = event.shiftKey ? stepMultiplier : 1
      if (shortcut === 'play') void actions.play()
      if (shortcut === 'record') void actions.record()
      if (shortcut === 'save' || shortcut === 'saveAs')
        void actions.saveProject(shortcut === 'saveAs')
      if (shortcut === 'undo') actions.undo()
      if (shortcut === 'redo') actions.redo()
      if (shortcut === 'delete') actions.removeClip()
      if (shortcut === 'copy') actions.copyClips()
      if (shortcut === 'paste') void actions.pasteClips()
      if (shortcut === 'home') actions.seek(0)
      if (shortcut === 'select') actions.setTool('select')
      if (shortcut === 'split') actions.splitAtPlayhead()
      if (shortcut === 'timeMode') actions.cycleTimeMode()
      if (shortcut === 'stepBack') actions.stepPlayhead(-1, multiplier)
      if (shortcut === 'stepForward') actions.stepPlayhead(1, multiplier)
    }
    window.addEventListener('keydown', keydown)
    return () => window.removeEventListener('keydown', keydown)
  }, [actions])
}
