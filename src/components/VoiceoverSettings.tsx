import { useState } from 'react'
import { t, translateMessage } from '../i18n'
import type { StudioController } from '../useStudio'
import { voices } from '../voices'
import VoiceSelect from './VoiceSelect'

type Props = Pick<StudioController, 'defaultVoice' | 'saveSpeechPreferences' | 'operationPending'>

export default function VoiceoverSettings({
  defaultVoice,
  saveSpeechPreferences,
  operationPending,
}: Props) {
  const [voice, setVoice] = useState(defaultVoice)
  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  const changed = voice !== defaultVoice
  const save = async () => {
    setSaving(true)
    setSaveError(null)
    setSaveError(await saveSpeechPreferences(voice))
    setSaving(false)
  }
  return (
    <form
      className="settings-section"
      onSubmit={(event) => {
        event.preventDefault()
        if (changed && !saving) void save()
      }}
    >
      <h3>{t('settings.voiceover')}</h3>
      <label className="settings-field">
        <span>{t('settings.speechVoice')}</span>
        <VoiceSelect
          value={voice}
          voices={voices}
          label={t('settings.speechVoice')}
          onChange={setVoice}
        />
      </label>
      <p className="hint muted">{t('settings.voiceHint')}</p>
      {saveError && (
        <p className="settings-error" role="alert">
          {translateMessage(saveError)}
        </p>
      )}
      <button
        type="submit"
        className="button secondary"
        disabled={!changed || saving || operationPending}
      >
        {t('settings.saveVoiceover')}
      </button>
    </form>
  )
}
