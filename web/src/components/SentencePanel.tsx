import type { SentenceResponse } from '../api/model'

interface SentencePanelProps {
  sentence: SentenceResponse
  onClose: () => void
}

export function SentencePanel({ sentence, onClose }: SentencePanelProps) {
  return (
    <aside className="w-80 border-l border-stone-200 overflow-y-auto bg-white shrink-0">
      <div className="flex items-center justify-between px-4 py-3 border-b border-stone-200">
        <h2 className="font-semibold text-stone-900">Sentence Detail</h2>
        <button
          onClick={onClose}
          className="text-stone-400 hover:text-stone-600 text-xl leading-none"
        >
          &times;
        </button>
      </div>
      <div className="px-4 py-4 space-y-4 text-sm">
        <Field label="Sentence #" value={sentence.sentence_number} />
        <Field label="Position" value={sentence.position} />
        <Field label="ID" value={sentence.id} />
        <div>
          <dt className="text-stone-500 mb-1">Text</dt>
          <dd className="text-stone-800 leading-relaxed">{sentence.text}</dd>
        </div>
      </div>
    </aside>
  )
}

function Field({ label, value }: { label: string; value: string | number }) {
  return (
    <div>
      <dt className="text-stone-500 mb-1">{label}</dt>
      <dd className="text-stone-800 font-mono">{String(value)}</dd>
    </div>
  )
}
