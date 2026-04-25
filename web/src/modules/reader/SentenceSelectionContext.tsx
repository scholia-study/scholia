import { createContext, useContext } from "react";
import type { SentenceResponse } from "../../api/model";
import { parseRangeKey, sentenceMatchesKey } from "./BlockRenderer";

interface SentenceSelectionState {
    /** The URL-friendly key (sentence_number, id, or range like "12-21") used for matching. */
    selectedKey: string | null;
    /** The UUID of the directly-clicked sentence (anchor). */
    clickedId: string | undefined;
}

const SentenceSelectionContext = createContext<SentenceSelectionState>({
    selectedKey: null,
    clickedId: undefined,
});

export const SentenceSelectionProvider = SentenceSelectionContext.Provider;

/**
 * Returns selection state for a given sentence:
 * - `isSelected`: true if this sentence is in the selection (single or range)
 * - `isCorrespondent`: true if this sentence matches by key but is not the one directly clicked
 *   (e.g. the aligned sentence in the other language)
 */
export function useSentenceSelection(sentence: SentenceResponse): {
    isSelected: boolean;
    isCorrespondent: boolean;
} {
    const { selectedKey, clickedId } = useContext(SentenceSelectionContext);
    const matches = sentenceMatchesKey(sentence, selectedKey);
    if (!matches) return { isSelected: false, isCorrespondent: false };
    // For ranges or when no clickedId, all matching sentences are "selected"
    const isRange = selectedKey ? parseRangeKey(selectedKey) !== null : false;
    if (!clickedId || isRange) return { isSelected: true, isCorrespondent: false };
    return {
        isSelected: sentence.id === clickedId,
        isCorrespondent: sentence.id !== clickedId,
    };
}
