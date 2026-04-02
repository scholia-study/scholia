import { createContext, useContext } from "react";
import type { SentenceResponse } from "../api/model";
import { sentenceMatchesKey } from "./BlockRenderer";

interface SentenceSelectionState {
    /** The URL-friendly key (sentence_number or id) used for matching. */
    selectedKey: string | null;
    /** The UUID of the directly-clicked sentence. */
    clickedId: string | undefined;
}

const SentenceSelectionContext = createContext<SentenceSelectionState>({
    selectedKey: null,
    clickedId: undefined,
});

export const SentenceSelectionProvider = SentenceSelectionContext.Provider;

/**
 * Returns selection state for a given sentence:
 * - `isSelected`: true if this is the directly-clicked sentence
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
    if (!clickedId) return { isSelected: true, isCorrespondent: false };
    return {
        isSelected: sentence.id === clickedId,
        isCorrespondent: sentence.id !== clickedId,
    };
}
