import { useState, useCallback } from 'react'
import type { ReaderSearch } from '../routes/__root'
import { TextPanel } from './TextPanel'
import { BookPickerPanel } from './BookPickerPanel'

export interface PanelState {
  bookSlug: string
  nodeSlug: string | undefined
}

interface ReaderLayoutProps {
  panels: PanelState[]
  selections: Map<number, string>  // panelIndex -> sentenceId
  onUpdateSearch: (search: ReaderSearch) => void
}

function serializeTexts(panels: PanelState[]): string {
  return panels
    .map((p) => (p.nodeSlug ? `${p.bookSlug}/${p.nodeSlug}` : p.bookSlug))
    .join(',')
}

function serializeSelections(selections: Map<number, string>): string | undefined {
  if (selections.size === 0) return undefined
  return Array.from(selections.entries())
    .map(([idx, id]) => `${idx}:${id}`)
    .join(',')
}

export function ReaderLayout({
  panels,
  selections,
  onUpdateSearch,
}: ReaderLayoutProps) {
  // Track which panels have TOC open (local state, not URL)
  const [tocOpen, setTocOpen] = useState<Set<number>>(() => new Set([0]))

  const handleNavigate = useCallback(
    (panelIndex: number, nodeSlug: string) => {
      const newPanels = panels.map((p, i) =>
        i === panelIndex ? { ...p, nodeSlug } : p,
      )
      onUpdateSearch({
        texts: serializeTexts(newPanels),
        s: serializeSelections(selections),
      })
    },
    [panels, selections, onUpdateSearch],
  )

  const handleSelectSentence = useCallback(
    (panelIndex: number, sentenceId: string) => {
      const newSelections = new Map(selections)
      if (newSelections.get(panelIndex) === sentenceId) {
        newSelections.delete(panelIndex)
      } else {
        newSelections.set(panelIndex, sentenceId)
      }
      onUpdateSearch({
        texts: serializeTexts(panels),
        s: serializeSelections(newSelections),
      })
    },
    [panels, selections, onUpdateSearch],
  )

  const handleDeselectSentence = useCallback(
    (panelIndex: number) => {
      const newSelections = new Map(selections)
      newSelections.delete(panelIndex)
      onUpdateSearch({
        texts: serializeTexts(panels),
        s: serializeSelections(newSelections),
      })
    },
    [panels, selections, onUpdateSearch],
  )

  const handleClosePanel = useCallback(
    (panelIndex: number) => {
      const newPanels = panels.filter((_, i) => i !== panelIndex)
      const newSelections = new Map<number, string>()
      for (const [idx, id] of selections) {
        if (idx < panelIndex) newSelections.set(idx, id)
        else if (idx > panelIndex) newSelections.set(idx - 1, id)
      }
      setTocOpen((prev) => {
        const next = new Set<number>()
        for (const idx of prev) {
          if (idx < panelIndex) next.add(idx)
          else if (idx > panelIndex) next.add(idx - 1)
        }
        return next
      })
      if (newPanels.length === 0) {
        onUpdateSearch({})
      } else {
        onUpdateSearch({
          texts: serializeTexts(newPanels),
          s: serializeSelections(newSelections),
        })
      }
    },
    [panels, selections, onUpdateSearch],
  )

  const handleToggleToc = useCallback((panelIndex: number) => {
    setTocOpen((prev) => {
      const next = new Set(prev)
      if (next.has(panelIndex)) next.delete(panelIndex)
      else next.add(panelIndex)
      return next
    })
  }, [])

  const handleAddPanel = useCallback(() => {
    // Add a picker panel (bookSlug = "_")
    const newPanels = [...panels, { bookSlug: '_', nodeSlug: undefined }]
    onUpdateSearch({
      texts: serializeTexts(newPanels),
      s: serializeSelections(selections),
    })
  }, [panels, selections, onUpdateSearch])

  const handlePickBook = useCallback(
    (panelIndex: number, bookSlug: string) => {
      const newPanels = panels.map((p, i) =>
        i === panelIndex ? { bookSlug, nodeSlug: undefined } : p,
      )
      // Open TOC for the new panel
      setTocOpen((prev) => new Set([...prev, panelIndex]))
      onUpdateSearch({
        texts: serializeTexts(newPanels),
        s: serializeSelections(selections),
      })
    },
    [panels, selections, onUpdateSearch],
  )

  return (
    <div className="flex h-screen">
      {panels.map((panel, idx) =>
        panel.bookSlug === '_' ? (
          <BookPickerPanel
            key={`picker-${idx}`}
            onPickBook={(slug) => handlePickBook(idx, slug)}
            onClose={panels.length > 1 ? () => handleClosePanel(idx) : undefined}
          />
        ) : (
          <TextPanel
            key={`${panel.bookSlug}-${idx}`}
            panelIndex={idx}
            bookSlug={panel.bookSlug}
            nodeSlug={panel.nodeSlug}
            tocOpen={tocOpen.has(idx)}
            selectedSentenceId={selections.get(idx)}
            onNavigate={(nodeSlug) => handleNavigate(idx, nodeSlug)}
            onSelectSentence={(sentenceId) => handleSelectSentence(idx, sentenceId)}
            onDeselectSentence={() => handleDeselectSentence(idx)}
            onToggleToc={() => handleToggleToc(idx)}
            onClose={panels.length > 1 ? () => handleClosePanel(idx) : undefined}
            isOnly={panels.length === 1}
          />
        ),
      )}
      <button
        onClick={handleAddPanel}
        className="flex items-center justify-center w-10 shrink-0 border-l border-stone-200 bg-white hover:bg-stone-50 text-stone-400 hover:text-stone-600 transition-colors"
        title="Add text panel"
      >
        <span className="text-xl">+</span>
      </button>
    </div>
  )
}
