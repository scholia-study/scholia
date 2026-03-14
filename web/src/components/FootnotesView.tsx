import parse from "html-react-parser";
import type { SentenceResponse } from "../api/model";

interface FootnotesViewProps {
    sentence: SentenceResponse;
}

export function FootnotesView({ sentence }: FootnotesViewProps) {
    const footnotes = sentence.footnotes ?? [];

    if (footnotes.length === 0) {
        return (
            <div className="p-4 text-sm text-stone-400">
                No footnotes for this sentence.
            </div>
        );
    }

    return (
        <div className="flex-1 overflow-y-auto">
            <div className="px-3 py-3 space-y-4">
                {footnotes.map((fn) => (
                    <div key={fn.id}>
                        <div className="text-xs text-stone-500 mb-1.5">
                            Footnote {fn.number}
                        </div>
                        <p className="text-sm text-stone-800 leading-relaxed">
                            {parse(
                                fn.sentences
                                    .map((s) => s.html)
                                    .join(" "),
                            )}
                        </p>
                    </div>
                ))}
            </div>
        </div>
    );
}
