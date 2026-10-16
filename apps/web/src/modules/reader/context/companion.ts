import { createContext, useContext } from "react";

const CompanionSentencesContext = createContext<ReadonlySet<string>>(new Set());

/** Ids of the sentences rendered as the companion translation in the
 *  interleaved view. They mirror the primary text, so a saved quotation shows
 *  on them in a paler tint than on the text it was saved from. */
export const CompanionSentencesProvider = CompanionSentencesContext.Provider;

export function useIsCompanionSentence(sentenceId: string): boolean {
    return useContext(CompanionSentencesContext).has(sentenceId);
}
