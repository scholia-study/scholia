import { createContext, useContext } from "react";
import type {
    FootnoteResponse,
    FootnoteSentenceResponse,
    SentenceResponse,
} from "../../../api/model";
import {
    footnoteSentenceMatchesKey,
    parseRangeKey,
    sentenceMatchesKey,
} from "../keys";

interface SelectionState {
    /** Main-body sentence selection. */
    main: {
        /** URL-friendly key (sentence_number, id, or range like "12-21"). */
        key: string | null;
        /** UUID of the directly-clicked sentence (anchor). Undefined for ranges. */
        clickedId: string | undefined;
    };
    /** Footnote-sentence selection (nested inside a main sentence). */
    footnote: {
        key: string | undefined;
        select: (sentence: FootnoteSentenceResponse, shiftKey: boolean) => void;
        clear: () => void;
    };
}

const SelectionContext = createContext<SelectionState>({
    main: { key: null, clickedId: undefined },
    footnote: {
        key: undefined,
        select: () => {},
        clear: () => {},
    },
});

export const SelectionProvider = SelectionContext.Provider;

/**
 * Selection state for a main-body sentence:
 * - `isSelected`: this sentence is in the selection (single or range)
 * - `isCorrespondent`: matches by key but is not the directly-clicked anchor
 *   (e.g. the aligned sentence in the other language)
 */
export function useSentenceSelection(sentence: SentenceResponse): {
    isSelected: boolean;
    isCorrespondent: boolean;
} {
    const { main } = useContext(SelectionContext);
    if (!sentenceMatchesKey(sentence, main.key)) {
        return { isSelected: false, isCorrespondent: false };
    }
    const isRange = main.key ? parseRangeKey(main.key) !== null : false;
    if (!main.clickedId || isRange) {
        return { isSelected: true, isCorrespondent: false };
    }
    return {
        isSelected: sentence.id === main.clickedId,
        isCorrespondent: sentence.id !== main.clickedId,
    };
}

/** Whether this footnote sentence is the (or part of the) selected one. */
export function useFootnoteSelection(sentence: FootnoteSentenceResponse): {
    isSelected: boolean;
} {
    const { footnote } = useContext(SelectionContext);
    return { isSelected: footnoteSentenceMatchesKey(sentence, footnote.key) };
}

/** Whether any footnote sentence inside the given footnotes is selected.
 *  Used to highlight the main sentence anchoring the selection. */
export function useFootnoteAnchor(
    footnotes: FootnoteResponse[] | undefined,
): boolean {
    const { footnote } = useContext(SelectionContext);
    if (!footnote.key || !footnotes) return false;
    return footnotes.some((fn) =>
        fn.sentences.some((s) => footnoteSentenceMatchesKey(s, footnote.key)),
    );
}

/** Action handlers for footnote selection (clicking a sentence in a footnote popover). */
export function useFootnoteActions(): {
    select: SelectionState["footnote"]["select"];
    clear: SelectionState["footnote"]["clear"];
} {
    const { footnote } = useContext(SelectionContext);
    return { select: footnote.select, clear: footnote.clear };
}
