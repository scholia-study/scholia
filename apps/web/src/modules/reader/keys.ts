import type {
    FootnoteSentenceResponse,
    SentenceResponse,
} from "../../api/model";

/** URL-friendly key for a sentence: `fig{N}` for a figure anchor,
 *  else sentence_number if available, otherwise the UUID. */
export function sentenceKey(s: SentenceResponse): string {
    if (s.figure_number != null) return `fig${s.figure_number}`;
    return s.sentence_number != null ? String(s.sentence_number) : s.id;
}

/** URL-friendly key for a footnote sentence: sentence_number if available, otherwise ID. */
export function footnoteSentenceKey(s: FootnoteSentenceResponse): string {
    return s.sentence_number != null ? String(s.sentence_number) : s.id;
}

/** Parse a figure key like "fig3" into the figure number, or null. */
export function parseFigureKey(key: string): number | null {
    const m = /^fig(\d+)$/.exec(key);
    return m ? Number(m[1]) : null;
}

/** Parse a range key like "12-21" into [start, end] or null. */
export function parseRangeKey(key: string): [number, number] | null {
    const dashIdx = key.indexOf("-");
    if (dashIdx <= 0) return null;
    const start = Number(key.slice(0, dashIdx));
    const end = Number(key.slice(dashIdx + 1));
    if (Number.isNaN(start) || Number.isNaN(end)) return null;
    return [start, end];
}

/** Check if a sentence matches a URL key (`fig{N}`, sentence_number, ID, or range like "12-21"). */
export function sentenceMatchesKey(
    s: SentenceResponse,
    key: string | undefined | null,
): boolean {
    if (!key) return false;
    if (s.figure_number != null) return key === `fig${s.figure_number}`;
    const range = parseRangeKey(key);
    if (range && s.sentence_number != null) {
        return s.sentence_number >= range[0] && s.sentence_number <= range[1];
    }
    return (
        s.id === key ||
        (s.sentence_number != null && String(s.sentence_number) === key)
    );
}

/** Check if a footnote sentence matches a URL key (sentence_number, ID, or range like "5-8"). */
export function footnoteSentenceMatchesKey(
    s: FootnoteSentenceResponse,
    key: string | undefined | null,
): boolean {
    if (!key) return false;
    const range = parseRangeKey(key);
    if (range && s.sentence_number != null) {
        return s.sentence_number >= range[0] && s.sentence_number <= range[1];
    }
    return (
        s.id === key ||
        (s.sentence_number != null && String(s.sentence_number) === key)
    );
}
