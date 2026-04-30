import {
    createContext,
    type ReactNode,
    useCallback,
    useContext,
    useMemo,
    useState,
} from "react";

interface FeedbackContextValue {
    open: boolean;
    draft: string;
    openModal: () => void;
    closeModal: () => void;
    setDraft: (next: string) => void;
    clearDraft: () => void;
}

const FeedbackContext = createContext<FeedbackContextValue | null>(null);

/**
 * Holds the modal's open state and the in-progress draft body across
 * route changes. Draft persists when the user closes the modal to check
 * something (e.g. reproduce a bug) and reopens it; it's cleared
 * explicitly on successful submit.
 *
 * Intentionally in-memory only (no localStorage) — drafts shouldn't
 * survive a tab reload.
 */
export function FeedbackProvider({ children }: { children: ReactNode }) {
    const [open, setOpen] = useState(false);
    const [draft, setDraft] = useState("");

    const openModal = useCallback(() => setOpen(true), []);
    const closeModal = useCallback(() => setOpen(false), []);
    const clearDraft = useCallback(() => setDraft(""), []);

    const value = useMemo(
        () => ({ open, draft, openModal, closeModal, setDraft, clearDraft }),
        [open, draft, openModal, closeModal, clearDraft],
    );

    return (
        <FeedbackContext.Provider value={value}>
            {children}
        </FeedbackContext.Provider>
    );
}

export function useFeedback() {
    const ctx = useContext(FeedbackContext);
    if (!ctx) {
        throw new Error("useFeedback must be used within a FeedbackProvider");
    }
    return ctx;
}
