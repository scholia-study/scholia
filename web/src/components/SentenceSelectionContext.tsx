import { createContext, useContext } from 'react'
import type { SentenceResponse } from '../api/model'

interface SentenceSelectionContextValue {
  selectedSentenceId: string | null
  selectedSentence: SentenceResponse | null
  onSelectSentence: (sentence: SentenceResponse) => void
}

export const SentenceSelectionContext = createContext<SentenceSelectionContextValue>({
  selectedSentenceId: null,
  selectedSentence: null,
  onSelectSentence: () => {},
})

export function useSentenceSelection() {
  return useContext(SentenceSelectionContext)
}
