import { useNavigate } from "@tanstack/react-router";
import { useCallback, useRef, useState } from "react";
import type { ReaderSearch } from "../routes/books.$bookSlug.$nodeSlug";
import { BookPickerPanel } from "./BookPickerPanel";
import { TextPanel } from "./TextPanel";

export interface PanelState {
    bookSlug: string;
    nodeSlug: string | undefined;
}

interface ReaderLayoutProps {
    panels: PanelState[];
    selections: Map<number, string>; // panelIndex -> sentenceId
}

/** Build search params from secondary panels and selections */
function buildSearch(
    panels: PanelState[],
    selections: Map<number, string>,
): ReaderSearch {
    const search: ReaderSearch = {};

    // Secondary panels: p2, p3, p4
    for (let i = 1; i < panels.length; i++) {
        const p = panels[i];
        const key = `p${i + 1}` as keyof ReaderSearch;
        search[key] = p.nodeSlug ? `${p.bookSlug}/${p.nodeSlug}` : p.bookSlug;
    }

    // Selections: s for primary, s2/s3/s4 for secondary
    for (const [idx, id] of selections) {
        if (idx === 0) search.s = id;
        else {
            const key = `s${idx + 1}` as keyof ReaderSearch;
            search[key] = id;
        }
    }

    return search;
}

let nextPanelId = 0;

export function ReaderLayout({ panels, selections }: ReaderLayoutProps) {
    const navigate = useNavigate();

    // Assign stable keys to panels that persist across index changes
    const panelKeysRef = useRef<string[]>([]);
    if (panelKeysRef.current.length < panels.length) {
        // New panels added — assign fresh IDs
        while (panelKeysRef.current.length < panels.length) {
            panelKeysRef.current.push(`panel-${nextPanelId++}`);
        }
    } else if (panelKeysRef.current.length > panels.length) {
        // Panels removed — will be trimmed by handleClosePanel
    }

    // Track which panels have TOC open (local state, not URL)
    const [tocOpen, setTocOpen] = useState<Set<number>>(() => new Set([0]));

    /** Navigate changing only search params (no path change) */
    const navigateSearch = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            replace?: boolean,
        ) => {
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: panels[0].bookSlug,
                    nodeSlug: panels[0].nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections),
                replace,
            });
        },
        [navigate, panels],
    );

    /** Navigate changing the path (primary panel node changed) */
    const navigatePath = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            replace?: boolean,
        ) => {
            const primary = newPanels[0];
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: primary.bookSlug,
                    nodeSlug: primary.nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections),
                replace,
            });
        },
        [navigate],
    );

    const handleNavigate = useCallback(
        (panelIndex: number, nodeSlug: string) => {
            const newPanels = panels.map((p, i) =>
                i === panelIndex ? { ...p, nodeSlug } : p,
            );
            if (panelIndex === 0) {
                navigatePath(newPanels, selections);
            } else {
                navigateSearch(newPanels, selections);
            }
        },
        [panels, selections, navigatePath, navigateSearch],
    );

    const handleSelectSentence = useCallback(
        (panelIndex: number, sentenceId: string) => {
            const newSelections = new Map(selections);
            if (newSelections.get(panelIndex) === sentenceId) {
                newSelections.delete(panelIndex);
            } else {
                newSelections.set(panelIndex, sentenceId);
            }
            navigateSearch(panels, newSelections);
        },
        [panels, selections, navigateSearch],
    );

    const handleDeselectSentence = useCallback(
        (panelIndex: number) => {
            const newSelections = new Map(selections);
            newSelections.delete(panelIndex);
            navigateSearch(panels, newSelections);
        },
        [panels, selections, navigateSearch],
    );

    const handleClosePanel = useCallback(
        (panelIndex: number) => {
            const newPanels = panels.filter((_, i) => i !== panelIndex);
            const newSelections = new Map<number, string>();
            for (const [idx, id] of selections) {
                if (idx < panelIndex) newSelections.set(idx, id);
                else if (idx > panelIndex) newSelections.set(idx - 1, id);
            }

            // Update stable keys: remove the closed panel's key
            panelKeysRef.current = panelKeysRef.current.filter(
                (_, i) => i !== panelIndex,
            );

            setTocOpen((prev) => {
                const next = new Set<number>();
                for (const idx of prev) {
                    if (idx < panelIndex) next.add(idx);
                    else if (idx > panelIndex) next.add(idx - 1);
                }
                return next;
            });

            if (newPanels.length === 0) {
                navigate({
                    to: "/books/$bookSlug",
                    params: { bookSlug: panels[0].bookSlug },
                });
            } else {
                const primary = newPanels[0];
                if (primary.nodeSlug) {
                    navigatePath(newPanels, newSelections);
                } else {
                    navigate({
                        to: "/books/$bookSlug",
                        params: { bookSlug: primary.bookSlug },
                    });
                }
            }
        },
        [panels, selections, navigate, navigatePath],
    );

    const handleToggleToc = useCallback((panelIndex: number) => {
        setTocOpen((prev) => {
            const next = new Set(prev);
            if (next.has(panelIndex)) next.delete(panelIndex);
            else next.add(panelIndex);
            return next;
        });
    }, []);

    const handleAddPanel = useCallback(() => {
        const newPanels = [...panels, { bookSlug: "_", nodeSlug: undefined }];
        navigateSearch(newPanels, selections);
    }, [panels, selections, navigateSearch]);

    const handlePickBook = useCallback(
        (panelIndex: number, bookSlug: string) => {
            const newPanels = panels.map((p, i) =>
                i === panelIndex ? { bookSlug, nodeSlug: undefined } : p,
            );
            setTocOpen((prev) => new Set([...prev, panelIndex]));
            if (panelIndex === 0) {
                navigatePath(newPanels, selections);
            } else {
                navigateSearch(newPanels, selections);
            }
        },
        [panels, selections, navigatePath, navigateSearch],
    );

    /** Replace-navigate for scroll-driven URL updates (no history entry) */
    const handleScrollNavigate = useCallback(
        (panelIndex: number, nodeSlug: string) => {
            const newPanels = panels.map((p, i) =>
                i === panelIndex ? { ...p, nodeSlug } : p,
            );
            if (panelIndex === 0) {
                navigatePath(newPanels, selections, true);
            } else {
                navigateSearch(newPanels, selections, true);
            }
        },
        [panels, selections, navigatePath, navigateSearch],
    );

    return (
        <div className="flex h-screen">
            {panels.map((panel, idx) =>
                panel.bookSlug === "_" ? (
                    <BookPickerPanel
                        key={panelKeysRef.current[idx] ?? `picker-${idx}`}
                        onPickBook={(slug) => handlePickBook(idx, slug)}
                        onClose={
                            panels.length > 1
                                ? () => handleClosePanel(idx)
                                : undefined
                        }
                    />
                ) : (
                    <TextPanel
                        key={panelKeysRef.current[idx] ?? `text-${idx}`}
                        panelIndex={idx}
                        bookSlug={panel.bookSlug}
                        nodeSlug={panel.nodeSlug}
                        tocOpen={tocOpen.has(idx)}
                        selectedSentenceId={selections.get(idx)}
                        onNavigate={(nodeSlug) => handleNavigate(idx, nodeSlug)}
                        onSelectSentence={(sentenceId) =>
                            handleSelectSentence(idx, sentenceId)
                        }
                        onDeselectSentence={() => handleDeselectSentence(idx)}
                        onToggleToc={() => handleToggleToc(idx)}
                        onClose={
                            panels.length > 1
                                ? () => handleClosePanel(idx)
                                : undefined
                        }
                        onScrollNavigate={(nodeSlug) =>
                            handleScrollNavigate(idx, nodeSlug)
                        }
                        isOnly={panels.length === 1}
                    />
                ),
            )}
            <button
                onClick={handleAddPanel}
                className="flex items-center justify-center w-10 shrink-0 border-l border-stone-200 bg-white hover:bg-stone-50 text-stone-400 hover:text-stone-600 transition-colors"
                title="Add text panel"
            >
                <span className="text-xl">+</span>
            </button>
        </div>
    );
}
