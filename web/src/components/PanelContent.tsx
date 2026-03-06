import type { NodeDetail, SentenceResponse } from '../api/model'
import { Block } from './BlockRenderer'
import type { MarginSettings } from './BlockRenderer'

interface PanelContentProps {
  node: NodeDetail
  selectedSentenceId: string | undefined
  onSelectSentence: (sentence: SentenceResponse) => void
  marginSettings?: MarginSettings
}

export function PanelContent({ node, selectedSentenceId, onSelectSentence, marginSettings }: PanelContentProps) {
  const hasActiveMargins = marginSettings && marginSettings.enabledSystems.size > 0

  return (
    <article className={hasActiveMargins ? 'max-w-4xl mx-auto py-12' : 'max-w-2xl mx-auto px-8 py-12'}>
      <div className={hasActiveMargins ? 'max-w-2xl mx-auto px-8' : undefined}>
        {node.blocks.map((block) => (
          <Block
            key={block.id}
            block={block}
            selectedSentenceId={selectedSentenceId ?? null}
            onSelectSentence={onSelectSentence}
            marginSettings={marginSettings}
          />
        ))}
      </div>
    </article>
  )
}
