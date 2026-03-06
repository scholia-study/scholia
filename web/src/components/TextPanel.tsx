import { useState, useCallback, useRef } from 'react'
import { useGetToc } from '../api/toc/toc'
import { useGetNode } from '../api/nodes/nodes'
import { PanelToc } from './PanelToc'
import { PanelContent } from './PanelContent'
import { PanelScrollView } from './PanelScrollView'
import type { PanelScrollViewHandle } from './PanelScrollView'
import { SentenceDetail } from './SentenceDetail'
import type { SentenceResponse } from '../api/model'

type ViewMode = 'section' | 'scroll'

interface TextPanelProps {
  panelIndex: number
  bookSlug: string
  nodeSlug: string | undefined
  tocOpen: boolean
  selectedSentenceId: string | undefined
  onNavigate: (nodeSlug: string) => void
  onSelectSentence: (sentenceId: string) => void
  onDeselectSentence: () => void
  onToggleToc: () => void
  onClose: (() => void) | undefined
  onScrollNavigate: (nodeSlug: string) => void
  isOnly: boolean
}

export function TextPanel({
  bookSlug,
  nodeSlug,
  tocOpen,
  selectedSentenceId,
  onNavigate,
  onSelectSentence,
  onDeselectSentence,
  onToggleToc,
  onClose,
  onScrollNavigate,
}: TextPanelProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('section')
  const [visibleSlug, setVisibleSlug] = useState<string | undefined>()
  const [selectedSentence, setSelectedSentence] = useState<SentenceResponse | undefined>()
  const scrollViewRef = useRef<PanelScrollViewHandle>(null)

  const handleVisibleNodeChange = useCallback((slug: string) => {
    setVisibleSlug(slug)
    onScrollNavigate(slug)
  }, [onScrollNavigate])

  const { data: tocData } = useGetToc(bookSlug)
  const toc = tocData?.data

  // In section mode, fetch the specific node
  const { data: nodeData, isLoading, error } = useGetNode(bookSlug, nodeSlug ?? '', {
    query: { enabled: !!nodeSlug && viewMode === 'section' },
  })
  const node = (nodeSlug && viewMode === 'section' && nodeData?.status === 200) ? nodeData.data : undefined

  const activeNodeSlug = viewMode === 'scroll' ? visibleSlug : nodeSlug

  // Only show detail when stored sentence matches the URL selection
  const showSentenceDetail = selectedSentence != null && selectedSentence.id === selectedSentenceId

  const handleSelectSentence = useCallback((sentence: SentenceResponse) => {
    setSelectedSentence(sentence)
    onSelectSentence(sentence.id)
  }, [onSelectSentence])

  const handleDeselectSentence = useCallback(() => {
    setSelectedSentence(undefined)
    onDeselectSentence()
  }, [onDeselectSentence])

  const handleToggleView = useCallback(() => {
    setViewMode((prev) => {
      if (prev === 'scroll' && visibleSlug) {
        // When switching from scroll to section, navigate to visible node
        onNavigate(visibleSlug)
      }
      return prev === 'section' ? 'scroll' : 'section'
    })
  }, [visibleSlug, onNavigate])

  const handleTocNavigate = useCallback((slug: string) => {
    if (viewMode === 'scroll') {
      scrollViewRef.current?.scrollToNode(slug)
    } else {
      onNavigate(slug)
    }
  }, [viewMode, onNavigate])

  return (
    <div className="flex flex-1 min-w-0 border-r border-stone-200 last:border-r-0">
      {/* TOC sidebar */}
      {tocOpen && toc ? (
        <PanelToc
          toc={toc}
          activeNodeSlug={activeNodeSlug}
          onNavigate={handleTocNavigate}
        />
      ) : null}

      {/* Main content area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 border-b border-stone-200 bg-white shrink-0">
          <button
            onClick={onToggleToc}
            className="text-xs px-2 py-1 rounded border border-stone-300 text-stone-600 hover:bg-stone-100 transition-colors"
            title={tocOpen ? 'Hide TOC' : 'Show TOC'}
          >
            {tocOpen ? '\u25C0' : '\u2630'}
          </button>
          <span className="text-sm text-stone-500 truncate flex-1">
            {node?.label ?? bookSlug}
          </span>
          <button
            onClick={handleToggleView}
            className="text-xs px-2 py-1 rounded border border-stone-300 text-stone-600 hover:bg-stone-100 transition-colors"
            title={viewMode === 'section' ? 'Switch to scroll view' : 'Switch to section view'}
          >
            {viewMode === 'section' ? 'Scroll' : 'Section'}
          </button>
          {onClose && (
            <button
              onClick={onClose}
              className="text-stone-400 hover:text-stone-600 text-lg leading-none"
              title="Close panel"
            >
              &times;
            </button>
          )}
        </div>

        {/* Content */}
        {viewMode === 'scroll' ? (
          <PanelScrollView
            ref={scrollViewRef}
            bookSlug={bookSlug}
            selectedSentenceId={selectedSentenceId}
            onSelectSentence={handleSelectSentence}
            onVisibleNodeChange={handleVisibleNodeChange}
          />
        ) : (
          <div className="flex-1 overflow-y-auto">
            {!nodeSlug ? (
              <div className="flex items-center justify-center h-full text-stone-400">
                <p>Select a section from the table of contents.</p>
              </div>
            ) : isLoading ? (
              <div className="flex items-center justify-center h-full text-stone-400">
                <p>Loading...</p>
              </div>
            ) : error ? (
              <div className="flex items-center justify-center h-full text-red-500">
                <p>Failed to load content.</p>
              </div>
            ) : node ? (
              <PanelContent
                node={node}
                selectedSentenceId={selectedSentenceId}
                onSelectSentence={handleSelectSentence}
              />
            ) : null}
          </div>
        )}
      </div>

      {/* Sentence detail */}
      {showSentenceDetail && (
        <SentenceDetail
          sentence={selectedSentence}
          onClose={handleDeselectSentence}
        />
      )}
    </div>
  )
}
