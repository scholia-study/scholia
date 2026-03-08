import type { SentenceResponse } from '../api/model'

interface SentenceDetailProps {
  sentence: SentenceResponse
}

export function SentenceDetail({ sentence }: SentenceDetailProps) {
  return (
    <div className="flex-1 overflow-y-auto">
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
    </div>
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
