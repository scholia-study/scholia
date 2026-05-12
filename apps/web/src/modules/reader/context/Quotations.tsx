import { createContext, useContext, useMemo } from "react";
import type { PageMarkerResponse, QuotationResponse } from "../../../api/model";

interface QuotationContextValue {
    quotations: QuotationResponse[];
    showBookmarks: boolean;
    /**
     * Whether a sentence carries a saved-quotation marker. Two paths:
     *
     * 1. Verse-key projection (Bible) — saved verses land in a Set
     *    keyed by `${projected_source_ref}::${verse_ref_value}`. The
     *    sentence's own verse markers are looked up against that set.
     *    The API resolves drift across translations (Romans doxology,
     *    DARBY Psalm titles) into target-local coords before we get
     *    here, so the match is direct (PLAN_BIG_BOOKS.md Q7 — visual
     *    hint only).
     * 2. sentence_number fallback (Kant) — quotations without verse
     *    keys fall back to the existing within-book sentence_number
     *    range match.
     */
    isSentenceSaved: (
        sentenceNumber: number | null | undefined,
        nodeSourceRef: string | null | undefined,
        pageMarkers: PageMarkerResponse[] | null | undefined,
    ) => boolean;
}

const QuotationContext = createContext<QuotationContextValue>({
    quotations: [],
    showBookmarks: true,
    isSentenceSaved: () => false,
});

/** Parse a verse `ref_value` like "5:10" into [chapter, verse]. */
function parseVerseRef(ref: string): [number, number] | null {
    const m = ref.match(/^(\d+):(\d+)$/);
    if (!m) return null;
    const chapter = Number.parseInt(m[1], 10);
    const verse = Number.parseInt(m[2], 10);
    if (Number.isNaN(chapter) || Number.isNaN(verse)) return null;
    return [chapter, verse];
}

/** Expand a verse range like (5:1, 5:3) into ["5:1", "5:2", "5:3"]. */
function expandVerseRange(start: string, end: string | null): string[] {
    if (!end || end === start) return [start];
    const a = parseVerseRef(start);
    const b = parseVerseRef(end);
    if (!a || !b) return [start, end];
    // We don't expand across chapters — current schema's anchor_node_id
    // pins both endpoints to one chapter, so a.chapter == b.chapter.
    if (a[0] !== b[0]) return [start, end];
    const out: string[] = [];
    for (let v = a[1]; v <= b[1]; v++) out.push(`${a[0]}:${v}`);
    return out;
}

export function QuotationProvider({
    quotations,
    showBookmarks,
    children,
}: {
    quotations: QuotationResponse[];
    showBookmarks: boolean;
    children: React.ReactNode;
}) {
    const value = useMemo(() => {
        // Build the verse-key set up front (cheap; ~tens of entries).
        // Use projected_* (target-local coords resolved by the API
        // through cross_translation_alignments). For older clients or
        // endpoints that haven't been wired yet, anchor_* is the
        // fallback — they coincide for non-drifting chapters.
        const savedVerseKeys = new Set<string>();
        for (const q of quotations) {
            const sourceRef = q.projected_source_ref ?? q.anchor_source_ref;
            const verseStart = q.projected_verse_start ?? q.anchor_verse_start;
            const verseEnd = q.projected_verse_end ?? q.anchor_verse_end;
            if (!sourceRef || !verseStart) continue;
            const verses = expandVerseRange(verseStart, verseEnd ?? null);
            for (const v of verses) {
                savedVerseKeys.add(`${sourceRef}::${v}`);
            }
        }
        const isSentenceSaved = (
            sentenceNumber: number | null | undefined,
            nodeSourceRef: string | null | undefined,
            pageMarkers: PageMarkerResponse[] | null | undefined,
        ) => {
            // 1. Verse-key match — Bible visual projection.
            if (nodeSourceRef && pageMarkers && pageMarkers.length > 0) {
                for (const m of pageMarkers) {
                    if (m.system_slug !== "verse") continue;
                    if (
                        savedVerseKeys.has(`${nodeSourceRef}::${m.ref_value}`)
                    ) {
                        return true;
                    }
                }
            }
            // 2. sentence_number range match — Kant within-book.
            if (sentenceNumber == null) return false;
            return quotations.some((q) => {
                // Anchor-verse covers the verse-key path; skip those here.
                if (q.projected_verse_start ?? q.anchor_verse_start) {
                    return false;
                }
                const start = q.anchor_sentence_start_number;
                const end = q.anchor_sentence_end_number ?? start;
                return sentenceNumber >= start && sentenceNumber <= end;
            });
        };
        return { quotations, showBookmarks, isSentenceSaved };
    }, [quotations, showBookmarks]);

    return (
        <QuotationContext.Provider value={value}>
            {children}
        </QuotationContext.Provider>
    );
}

export function useQuotationContext() {
    return useContext(QuotationContext);
}
