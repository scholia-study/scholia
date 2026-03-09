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
}

/** Build search params from secondary panels, selections, and resources state */
function buildSearch(
    panels: PanelState[],
    selections: Map<number, string>,
    resourcesOpen: Set<number>,
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

    return search;
}

let nextPanelId = 0;

export function ReaderLayout({
    panels,
    selections,
    resourcesOpen,
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
        ) => {
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: panels[0].bookSlug,
                    nodeSlug: panels[0].nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections, newResourcesOpen),
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
            newResourcesOpen: Set<number>,
            replace?: boolean,
        ) => {
            const primary = newPanels[0];
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: primary.bookSlug,
                    nodeSlug: primary.nodeSlug!,
                },
                search: buildSearch(newPanels, newSelections, newResourcesOpen),
                replace,
            });
        },
        [navigate],
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
                    navigatePath(newPanels, newSelections, newResourcesOpen);
                } else {
                    navigate({
                        to: "/books/$bookSlug",
                        params: { bookSlug: primary.bookSlug },
                    });
                }
            }
        },
        [panels, selections, resourcesOpen, navigate, navigatePath],
    );

    const handleCloseResources = useCallback(
        (panelIndex: number) => {
            const newResourcesOpen = new Set(resourcesOpen);
            newResourcesOpen.delete(panelIndex);
            // Also deselect sentence when closing resources
            const newSelections = new Map(selections);
            newSelections.delete(panelIndex);
            navigateSearch(panels, newSelections, newResourcesOpen);
        },
        [panels, selections, resourcesOpen, navigateSearch],
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

            // Insert stable key
            panelKeysRef.current = [
                ...panelKeysRef.current.slice(0, insertAt),
                `panel-${nextPanelId++}`,
                ...panelKeysRef.current.slice(insertAt),
            ];

            navigateSearch(newPanels, newSelections, newResourcesOpen);
        },
        [panels, selections, resourcesOpen, navigateSearch],
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
                    selectedSentenceId={selections.get(idx)}
                    onSelectSentence={(sentenceId) =>
                        handleSelectSentence(idx, sentenceId)
                    }
                    onToggleResources={() => handleCloseResources(idx)}
                    onClose={
                        panels.length > 1
                            ? () => handleClosePanel(idx)
                            : undefined
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
