import type { SentenceKind } from "#/api/model";

/** The anchor fields every saved-quotation shape carries, whatever else
 *  differs between a quotation row and one of its notes. */
export interface AnchoredLike {
    sentence_kind: SentenceKind;
    anchor_sentence_start_number: number;
    anchor_sentence_end_number?: number | null;
    anchor_main_sentence_number?: number | null;
}

export interface ReaderAnchorSearch {
    s: string;
    fs?: string;
    r: string;
    rv: string;
}

export function sentenceLabel(a: AnchoredLike): string {
    const start = a.anchor_sentence_start_number;
    const end = a.anchor_sentence_end_number;
    if (a.sentence_kind === "figure") return `Figure ${start}`;
    const isFootnote = a.sentence_kind === "footnote";
    const single = isFootnote ? "Footnote sentence" : "Sentence";
    const plural = isFootnote ? "Footnote sentences" : "Sentences";
    if (end == null || end === start) return `${single} ${start}`;
    return `${plural} ${start}–${end}`;
}

export function readerAnchorSearch(a: AnchoredLike): ReaderAnchorSearch {
    const startStr = String(a.anchor_sentence_start_number);
    const rangeStr =
        a.anchor_sentence_end_number &&
        a.anchor_sentence_end_number !== a.anchor_sentence_start_number
            ? `${a.anchor_sentence_start_number}-${a.anchor_sentence_end_number}`
            : startStr;
    if (a.sentence_kind === "figure") {
        return {
            s: `fig${a.anchor_sentence_start_number}`,
            r: "1",
            rv: "notes",
        };
    }
    if (a.sentence_kind === "footnote" && a.anchor_main_sentence_number) {
        return {
            s: String(a.anchor_main_sentence_number),
            fs: rangeStr,
            r: "1",
            rv: "notes",
        };
    }
    return { s: rangeStr, r: "1", rv: "notes" };
}
