import { createContext, useContext, useMemo } from "react";
import type { PageMarkerResponse, QuotationResponse } from "../../../api/model";

/** A sentence's standing among the reader's own quotations: highlighted when
 *  saved, and additionally marked in the margin when a quotation covering it
 *  carries notes. */
export type SavedState = "none" | "saved" | "noted";

interface QuotationContextValue {
    quotations: QuotationResponse[];
    showBookmarks: boolean;
    showHighlights: boolean;
    /**
     * Whether a sentence is covered by a saved quotation. Two paths:
     *
     * 1. Verse-key projection (Bible) — saved verses land in a Set
     *    keyed by `${projected_source_ref}::${verse_ref_value}`. The
     *    sentence's own verse markers are looked up against that set.
     *    The API resolves drift across translations (Romans doxology,
     *    DARBY Psalm titles) into target-local coords before we get
     *    here, so the match is direct (visual
     *    hint only).
     * 2. sentence_number fallback (Kant) — quotations without verse
     *    keys fall back to the existing within-book sentence_number
     *    range match.
     */
    sentenceSavedState: (
        sentenceNumber: number | null | undefined,
        nodeSourceRef: string | null | undefined,
        pageMarkers: PageMarkerResponse[] | null | undefined,
    ) => SavedState;
    /** The same for a figure block. Figures are addressed by figure_number
     *  (kind 'figure'), a numbering space separate from body sentences. */
    figureSavedState: (figureNumber: number | null | undefined) => SavedState;
}

const QuotationContext = createContext<QuotationContextValue>({
    quotations: [],
    showBookmarks: true,
    showHighlights: true,
    sentenceSavedState: () => "none",
    figureSavedState: () => "none",
});

function savedStateOf(matches: QuotationResponse[]): SavedState {
    if (matches.length === 0) return "none";
    return matches.some((q) => q.note_count > 0) ? "noted" : "saved";
}

function mergeSavedState(a: SavedState, b: SavedState): SavedState {
    if (a === "noted" || b === "noted") return "noted";
    if (a === "saved" || b === "saved") return "saved";
    return "none";
}

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
    showHighlights,
    children,
}: {
    quotations: QuotationResponse[];
    showBookmarks: boolean;
    showHighlights: boolean;
    children: React.ReactNode;
}) {
    const value = useMemo(() => {
        // Build the verse-key set up front (cheap; ~tens of entries).
        // Use projected_* (target-local coords resolved by the API
        // through cross_translation_alignments). For older clients or
        // endpoints that haven't been wired yet, anchor_* is the
        // fallback — they coincide for non-drifting chapters.
        const savedVerseKeys = new Map<string, SavedState>();
        for (const q of quotations) {
            const sourceRef = q.projected_source_ref ?? q.anchor_source_ref;
            const verseStart = q.projected_verse_start ?? q.anchor_verse_start;
            const verseEnd = q.projected_verse_end ?? q.anchor_verse_end;
            if (!sourceRef || !verseStart) continue;
            const verses = expandVerseRange(verseStart, verseEnd ?? null);
            for (const v of verses) {
                const key = `${sourceRef}::${v}`;
                savedVerseKeys.set(
                    key,
                    mergeSavedState(
                        savedVerseKeys.get(key) ?? "none",
                        savedStateOf([q]),
                    ),
                );
            }
        }
        const sentenceSavedState = (
            sentenceNumber: number | null | undefined,
            nodeSourceRef: string | null | undefined,
            pageMarkers: PageMarkerResponse[] | null | undefined,
        ): SavedState => {
            let state: SavedState = "none";
            // 1. Verse-key match — Bible visual projection.
            if (nodeSourceRef && pageMarkers && pageMarkers.length > 0) {
                for (const m of pageMarkers) {
                    if (m.system_slug !== "verse") continue;
                    state = mergeSavedState(
                        state,
                        savedVerseKeys.get(
                            `${nodeSourceRef}::${m.ref_value}`,
                        ) ?? "none",
                    );
                }
            }
            // 2. sentence_number range match — Kant within-book.
            if (sentenceNumber == null) return state;
            const matches = quotations.filter((q) => {
                // Anchor-verse covers the verse-key path; skip those here.
                if (q.projected_verse_start ?? q.anchor_verse_start) {
                    return false;
                }
                // Figure quotations carry the figure_number, which lives in
                // a different numbering space than body sentences.
                if (q.sentence_kind === "figure") return false;
                const start = q.anchor_sentence_start_number;
                const end = q.anchor_sentence_end_number ?? start;
                return sentenceNumber >= start && sentenceNumber <= end;
            });
            return mergeSavedState(state, savedStateOf(matches));
        };
        const figureSavedState = (
            figureNumber: number | null | undefined,
        ): SavedState => {
            if (figureNumber == null) return "none";
            return savedStateOf(
                quotations.filter(
                    (q) =>
                        q.sentence_kind === "figure" &&
                        q.anchor_sentence_start_number === figureNumber,
                ),
            );
        };
        return {
            quotations,
            showBookmarks,
            showHighlights,
            sentenceSavedState,
            figureSavedState,
        };
    }, [quotations, showBookmarks, showHighlights]);

    return (
        <QuotationContext.Provider value={value}>
            {children}
        </QuotationContext.Provider>
    );
}

export function useQuotationContext() {
    return useContext(QuotationContext);
}
