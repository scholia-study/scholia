import { useNavigate } from "@tanstack/react-router";
import { useCallback, useRef } from "react";
import type { ReaderSearch } from "../routes/books.$bookSlug.$nodeSlug";
import { TextPanel } from "./TextPanel";

export interface PanelState {
    bookSlug: string;
    nodeSlug: string | undefined;
}

interface ReaderLayoutProps {
    panels: PanelState[];
    selections: Map<number, string>; // panelIndex -> sentenceId
    resourcesOpen: Set<number>; // panelIndex -> resources panel open
    showOriginal: Set<number>;
    resourceViews: Map<number, string>; // panelIndex -> resource view (toc, compare, sentence)
}

/** Build search params from secondary panels, selections, and resources state */
function buildSearch(
    panels: PanelState[],
    selections: Map<number, string>,
    resourcesOpen: Set<number>,
    showOriginal: Set<number>,
    resourceViews: Map<number, string>,
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

    // Resources panel visibility: r, r2, r3, r4
    for (const idx of resourcesOpen) {
        if (idx === 0) search.r = "1";
        else {
            const key = `r${idx + 1}` as keyof ReaderSearch;
            search[key] = "1";
        }
    }

    // Show original text per panel: og, og2, og3, og4
    for (const idx of showOriginal) {
        if (idx === 0) search.og = "1";
        else {
            const key = `og${idx + 1}` as keyof ReaderSearch;
            search[key] = "1";
        }
    }

    // Resource view per panel: rv, rv2, rv3, rv4
    for (const [idx, view] of resourceViews) {
        if (idx === 0) search.rv = view;
        else {
            const key = `rv${idx + 1}` as keyof ReaderSearch;
            search[key] = view;
        }
    }

    return search;
}

let nextPanelId = 0;

export function ReaderLayout({
    panels,
    selections,
    resourcesOpen,
    showOriginal,
    resourceViews,
}: ReaderLayoutProps) {
    const navigate = useNavigate();

    // Assign stable keys to panels that persist across index changes
    const panelKeysRef = useRef<string[]>([]);
    if (panelKeysRef.current.length < panels.length) {
        while (panelKeysRef.current.length < panels.length) {
            panelKeysRef.current.push(`panel-${nextPanelId++}`);
        }
    }

    /** Navigate changing only search params (no path change) */
    const navigateSearch = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            newResourcesOpen: Set<number>,
            replace?: boolean,
            overrideShowOriginal?: Set<number>,
            overrideResourceViews?: Map<number, string>,
        ) => {
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: panels[0].bookSlug,
                    nodeSlug: panels[0].nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections, newResourcesOpen, overrideShowOriginal ?? showOriginal, overrideResourceViews ?? resourceViews),
                replace,
            });
        },
        [navigate, panels, showOriginal, resourceViews],
    );

    /** Navigate changing the path (primary panel node changed) */
    const navigatePath = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            newResourcesOpen: Set<number>,
            replace?: boolean,
            overrideShowOriginal?: Set<number>,
            overrideResourceViews?: Map<number, string>,
        ) => {
            const primary = newPanels[0];
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: primary.bookSlug,
                    nodeSlug: primary.nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections, newResourcesOpen, overrideShowOriginal ?? showOriginal, overrideResourceViews ?? resourceViews),
                replace,
            });
        },
        [navigate, showOriginal, resourceViews],
    );

    const handleSelectSentence = useCallback(
        (panelIndex: number, sentenceId: string) => {
            const newSelections = new Map(selections);
            if (newSelections.get(panelIndex) === sentenceId) {
                newSelections.delete(panelIndex);
            } else {
                newSelections.set(panelIndex, sentenceId);
            }
            // Open resources panel when selecting a sentence
            const newResourcesOpen = new Set(resourcesOpen);
            newResourcesOpen.add(panelIndex);
            navigateSearch(panels, newSelections, newResourcesOpen);
        },
        [panels, selections, resourcesOpen, navigateSearch],
    );

    const handleClosePanel = useCallback(
        (panelIndex: number) => {
            const newPanels = panels.filter((_, i) => i !== panelIndex);
            const newSelections = new Map<number, string>();
            for (const [idx, id] of selections) {
                if (idx < panelIndex) newSelections.set(idx, id);
                else if (idx > panelIndex) newSelections.set(idx - 1, id);
            }

            // Shift resources open indices
            const newResourcesOpen = new Set<number>();
            for (const idx of resourcesOpen) {
                if (idx < panelIndex) newResourcesOpen.add(idx);
                else if (idx > panelIndex) newResourcesOpen.add(idx - 1);
            }

            // Shift show-original indices
            const newShowOriginal = new Set<number>();
            for (const idx of showOriginal) {
                if (idx < panelIndex) newShowOriginal.add(idx);
                else if (idx > panelIndex) newShowOriginal.add(idx - 1);
            }

            // Shift resource view indices
            const newResourceViews = new Map<number, string>();
            for (const [idx, view] of resourceViews) {
                if (idx < panelIndex) newResourceViews.set(idx, view);
                else if (idx > panelIndex) newResourceViews.set(idx - 1, view);
            }

            // Update stable keys: remove the closed panel's key
            panelKeysRef.current = panelKeysRef.current.filter(
                (_, i) => i !== panelIndex,
            );

            if (newPanels.length === 0) {
                navigate({
                    to: "/books/$bookSlug",
                    params: { bookSlug: panels[0].bookSlug },
                });
            } else {
                const primary = newPanels[0];
                if (primary.nodeSlug) {
                    navigatePath(newPanels, newSelections, newResourcesOpen, false, newShowOriginal, newResourceViews);
                } else {
                    navigate({
                        to: "/books/$bookSlug",
                        params: { bookSlug: primary.bookSlug },
                    });
                }
            }
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, navigate, navigatePath],
    );

    const handleCloseResources = useCallback(
        (panelIndex: number) => {
            const newResourcesOpen = new Set(resourcesOpen);
            newResourcesOpen.delete(panelIndex);
            // Also deselect sentence when closing resources
            const newSelections = new Map(selections);
            newSelections.delete(panelIndex);
            const newResourceViews = new Map(resourceViews);
            newResourceViews.delete(panelIndex);
            navigateSearch(panels, newSelections, newResourcesOpen, false, undefined, newResourceViews);
        },
        [panels, selections, resourcesOpen, resourceViews, navigateSearch],
    );

    const handleAddComparisonPanel = useCallback(
        (afterIndex: number, bookSlug: string, nodeSlug: string) => {
            const insertAt = afterIndex + 1;
            const newPanels = [
                ...panels.slice(0, insertAt),
                { bookSlug, nodeSlug },
                ...panels.slice(insertAt),
            ];

            // Shift selections for indices >= insertAt, clear source panel's selection
            const newSelections = new Map<number, string>();
            for (const [idx, id] of selections) {
                if (idx === afterIndex) continue;
                if (idx < insertAt) newSelections.set(idx, id);
                else newSelections.set(idx + 1, id);
            }

            // Close source panel's resources
            const newResourcesOpen = new Set<number>();
            for (const idx of resourcesOpen) {
                if (idx === afterIndex) continue;
                if (idx < insertAt) newResourcesOpen.add(idx);
                else newResourcesOpen.add(idx + 1);
            }

            // Shift show-original indices
            const newShowOriginal = new Set<number>();
            for (const idx of showOriginal) {
                if (idx < insertAt) newShowOriginal.add(idx);
                else newShowOriginal.add(idx + 1);
            }

            // Shift resource view indices, clear source panel's view
            const newResourceViews = new Map<number, string>();
            for (const [idx, view] of resourceViews) {
                if (idx === afterIndex) continue;
                if (idx < insertAt) newResourceViews.set(idx, view);
                else newResourceViews.set(idx + 1, view);
            }

            // Insert stable key
            panelKeysRef.current = [
                ...panelKeysRef.current.slice(0, insertAt),
                `panel-${nextPanelId++}`,
                ...panelKeysRef.current.slice(insertAt),
            ];

            navigateSearch(newPanels, newSelections, newResourcesOpen, false, newShowOriginal, newResourceViews);
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, navigateSearch],
    );

    /** Replace-navigate for scroll-driven URL updates (no history entry) */
    const handleScrollNavigate = useCallback(
        (panelIndex: number, nodeSlug: string) => {
            const newPanels = panels.map((p, i) =>
                i === panelIndex ? { ...p, nodeSlug } : p,
            );
            if (panelIndex === 0) {
                navigatePath(newPanels, selections, resourcesOpen, true);
            } else {
                navigateSearch(newPanels, selections, resourcesOpen, true);
            }
        },
        [panels, selections, resourcesOpen, navigatePath, navigateSearch],
    );

    const handleToggleOriginal = useCallback(
        (panelIndex: number) => {
            const newShowOriginal = new Set(showOriginal);
            if (newShowOriginal.has(panelIndex)) newShowOriginal.delete(panelIndex);
            else newShowOriginal.add(panelIndex);
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: panels[0].bookSlug,
                    nodeSlug: panels[0].nodeSlug!,
                },
                search: buildSearch(panels, selections, resourcesOpen, newShowOriginal, resourceViews),
            });
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, navigate],
    );

    const handleResourceViewChange = useCallback(
        (panelIndex: number, view: string | undefined) => {
            const newResourceViews = new Map(resourceViews);
            if (view) {
                newResourceViews.set(panelIndex, view);
            } else {
                newResourceViews.delete(panelIndex);
            }
            navigateSearch(panels, selections, resourcesOpen, false, undefined, newResourceViews);
        },
        [panels, selections, resourcesOpen, resourceViews, navigateSearch],
    );

    const canAddPanel = panels.length < 4;

    return (
        <div className="flex h-screen">
            {panels.map((panel, idx) => (
                <TextPanel
                    key={panelKeysRef.current[idx] ?? `text-${idx}`}
                    panelIndex={idx}
                    bookSlug={panel.bookSlug}
                    nodeSlug={panel.nodeSlug}
                    resourcesOpen={resourcesOpen.has(idx)}
                    resourceView={resourceViews.get(idx)}
                    selectedSentenceId={selections.get(idx)}
                    showOriginal={showOriginal.has(idx)}
                    onSelectSentence={(sentenceId) =>
                        handleSelectSentence(idx, sentenceId)
                    }
                    onToggleOriginal={() => handleToggleOriginal(idx)}
                    onToggleResources={() => handleCloseResources(idx)}
                    onResourceViewChange={(view) => handleResourceViewChange(idx, view)}
                    onClose={
                        panels.length > 1
                            ? () => handleClosePanel(idx)
                            : () =>
                                  navigate({
                                      to: "/books",
                                  })
                    }
                    onScrollNavigate={(nodeSlug) =>
                        handleScrollNavigate(idx, nodeSlug)
                    }
                    onAddComparisonPanel={(bookSlug, nodeSlug) =>
                        handleAddComparisonPanel(idx, bookSlug, nodeSlug)
                    }
                    canAddPanel={canAddPanel}
                />
            ))}
        </div>
    );
}
