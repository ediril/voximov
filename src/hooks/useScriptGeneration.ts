import { useEffect, useState, type MutableRefObject } from 'react'
import type { GenerationEvent } from '../protocol'
import type { StudioClient } from '../services/StudioClient'
import { isGenerating, withGenerationEvent, type ActiveGenerations } from '../timeline/script'

export function useScriptGeneration(
  engineRef: MutableRefObject<StudioClient | null>,
  onFailure: (error: string) => void,
  onGuessed: (words: string[]) => void,
) {
  const [generations, setGenerations] = useState<ActiveGenerations>({})
  useEffect(() => {
    const client = engineRef.current
    if (!client) return
    client.onGeneration = (event: GenerationEvent) => {
      setGenerations((active) => withGenerationEvent(active, event))
      if (event.state === 'failed' && event.error) onFailure(event.error)
      if (event.state === 'done' && event.guessed?.length) onGuessed(event.guessed)
    }
    return () => {
      client.onGeneration = () => {}
    }
  }, [engineRef, onFailure, onGuessed])
  return { generations, generating: isGenerating(generations) }
}
