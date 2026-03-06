import type { NodeDetail, SentenceResponse } from '../api/model'
import { Block } from './BlockRenderer'

interface PanelContentProps {
  node: NodeDetail
  selectedSentenceId: string | undefined
  onSelectSentence: (sentence: SentenceResponse) => void
}

export function PanelContent({ node, selectedSentenceId, onSelectSentence }: PanelContentProps) {
  return (
    <article className="max-w-2xl mx-auto px-8 py-12">
      {node.blocks.map((block) => (
        <Block
          key={block.id}
          block={block}
          selectedSentenceId={selectedSentenceId ?? null}
          onSelectSentence={onSelectSentence}
        />
      ))}
    </article>
  )
}
