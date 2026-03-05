import type { SentenceResponse } from '../api/model'

interface SentenceDetailProps {
  sentence: SentenceResponse
  onClose: () => void
}

export function SentenceDetail({ sentence, onClose }: SentenceDetailProps) {
  return (
    <aside className="w-64 border-l border-stone-200 overflow-y-auto bg-white shrink-0">
      <div className="flex items-center justify-between px-3 py-2 border-b border-stone-200">
        <h2 className="font-semibold text-sm text-stone-900">Sentence</h2>
        <button
          onClick={onClose}
          className="text-stone-400 hover:text-stone-600 text-lg leading-none"
        >
          &times;
        </button>
      </div>
      <div className="px-3 py-3 space-y-3 text-sm">
        {sentence.sentence_number != null && (
          <Field label="Sentence #" value={sentence.sentence_number} />
        )}
        <Field label="Position" value={sentence.position} />
        <Field label="ID" value={sentence.id} />
        {sentence.page_markers.length > 0 && (
          <div>
            <dt className="text-stone-500 mb-1">Page Markers</dt>
            <dd className="space-y-1">
              {sentence.page_markers.map((pm, i) => (
                <div key={i} className="text-stone-800 font-mono text-xs">
                  {pm.system_slug}: {pm.ref_value}
                </div>
              ))}
            </dd>
          </div>
        )}
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
      <dd className="text-stone-800 font-mono text-xs">{String(value)}</dd>
    </div>
  )
}
