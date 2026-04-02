import { createContext, useContext } from "react";
import type { FootnoteSentenceResponse } from "../api/model";

interface FootnoteSelectionState {
    selectedFootnoteSentenceId: string | undefined;
    onSelectFootnoteSentence: (sentence: FootnoteSentenceResponse) => void;
    onClearFootnoteSentence: () => void;
}

const FootnoteSelectionContext = createContext<FootnoteSelectionState>({
    selectedFootnoteSentenceId: undefined,
    onSelectFootnoteSentence: () => {},
    onClearFootnoteSentence: () => {},
});

export const FootnoteSelectionProvider = FootnoteSelectionContext.Provider;

export function useFootnoteSelection() {
    return useContext(FootnoteSelectionContext);
}
