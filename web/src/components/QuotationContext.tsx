import { createContext, useContext, useMemo } from "react";
import type { QuotationResponse } from "../api/model";

interface QuotationContextValue {
    quotations: QuotationResponse[];
    showBookmarks: boolean;
    isSentenceSaved: (sentenceNumber: number) => boolean;
}

const QuotationContext = createContext<QuotationContextValue>({
    quotations: [],
    showBookmarks: true,
    isSentenceSaved: () => false,
});

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
        const isSentenceSaved = (sentenceNumber: number) => {
            return quotations.some((q) => {
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
