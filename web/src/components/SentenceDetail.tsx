import type { FootnoteSentenceResponse, SentenceResponse } from "../api/model";

interface SentenceDetailProps {
    sentence: SentenceResponse | FootnoteSentenceResponse | (SentenceResponse | FootnoteSentenceResponse)[];
}

export function SentenceDetail({ sentence }: SentenceDetailProps) {
    if (Array.isArray(sentence)) {
        return <RangeDetail sentences={sentence} />;
    }
    return <SingleDetail sentence={sentence} />;
}

function RangeDetail({ sentences }: { sentences: (SentenceResponse | FootnoteSentenceResponse)[] }) {
    const numbers = sentences
        .map((s) => ("sentence_number" in s ? s.sentence_number : null))
        .filter((n): n is number => n != null);
    const rangeLabel = numbers.length > 0
        ? `Sentences ${Math.min(...numbers)}\u2013${Math.max(...numbers)}`
        : `${sentences.length} sentences`;

    return (
        <div className="flex-1 overflow-y-auto">
            <div className="px-3 py-3 space-y-3 text-sm">
                <Field label="Range" value={rangeLabel} />
                <Field label="Count" value={sentences.length} />
                <div>
                    <dt className="text-stone-500 mb-1">Text</dt>
                    <dd className="text-stone-800 leading-relaxed">
                        {sentences.map((s) => s.text).join(" ")}
                    </dd>
                </div>
            </div>
        </div>
    );
}

function SingleDetail({ sentence }: { sentence: SentenceResponse | FootnoteSentenceResponse }) {
    return (
        <div className="flex-1 overflow-y-auto">
            <div className="px-3 py-3 space-y-3 text-sm">
                {"sentence_number" in sentence &&
                    sentence.sentence_number != null && (
                        <Field
                            label="Sentence #"
                            value={sentence.sentence_number}
                        />
                    )}
                <Field label="Position" value={sentence.position} />
                <Field label="ID" value={sentence.id} />
                {"page_markers" in sentence && sentence.page_markers.length > 0 && (
                    <div>
                        <dt className="text-stone-500 mb-1">Page Markers</dt>
                        <dd className="space-y-1">
                            {sentence.page_markers.map((pm, i) => (
                                <div
                                    key={i}
                                    className="text-stone-800 font-mono text-xs"
                                >
                                    {pm.system_slug}: {pm.ref_value}
                                </div>
                            ))}
                        </dd>
                    </div>
                )}
                <div>
                    <dt className="text-stone-500 mb-1">Text</dt>
                    <dd className="text-stone-800 leading-relaxed">
                        {sentence.text}
                    </dd>
                </div>
            </div>
        </div>
    );
}

function Field({ label, value }: { label: string; value: string | number }) {
    return (
        <div>
            <dt className="text-stone-500 mb-1">{label}</dt>
            <dd className="text-stone-800 font-mono text-xs">
                {String(value)}
            </dd>
        </div>
    );
}
