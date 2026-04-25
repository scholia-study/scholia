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
    viewModes: Map<number, string>; // panelIndex -> "s" | "t" | "st"
    viewLayouts: Map<number, string>; // panelIndex -> "sp" | "ss" | "bpl" | "bpr" | "bsl" | "bsr"
    companionSlugs: Map<number, string>; // panelIndex -> companion book slug
    footnoteSentenceSelections: Map<number, string>; // panelIndex -> footnote sentence id
}

/** Build search params from secondary panels, selections, and resources state */
function buildSearch(
    panels: PanelState[],
    selections: Map<number, string>,
    resourcesOpen: Set<number>,
    showOriginal: Set<number>,
    resourceViews: Map<number, string>,
    viewModes?: Map<number, string>,
    viewLayouts?: Map<number, string>,
    companionSlugs?: Map<number, string>,
    footnoteSentenceSelections?: Map<number, string>,
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

    // View mode per panel: vm, vm2, vm3, vm4
    if (viewModes) {
        for (const [idx, mode] of viewModes) {
            if (idx === 0) search.vm = mode;
            else {
                const key = `vm${idx + 1}` as keyof ReaderSearch;
                search[key] = mode;
            }
        }
    }

    // View layout per panel: vl, vl2, vl3, vl4
    if (viewLayouts) {
        for (const [idx, layout] of viewLayouts) {
            if (idx === 0) search.vl = layout;
            else {
                const key = `vl${idx + 1}` as keyof ReaderSearch;
                search[key] = layout;
            }
        }
    }

    // Companion slug per panel: vt, vt2, vt3, vt4
    if (companionSlugs) {
        for (const [idx, slug] of companionSlugs) {
            if (idx === 0) search.vt = slug;
            else {
                const key = `vt${idx + 1}` as keyof ReaderSearch;
                search[key] = slug;
            }
        }
    }

    // Footnote sentence selections: fs, fs2, fs3, fs4
    if (footnoteSentenceSelections) {
        for (const [idx, id] of footnoteSentenceSelections) {
            if (idx === 0) search.fs = id;
            else {
                const key = `fs${idx + 1}` as keyof ReaderSearch;
                search[key] = id;
            }
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
    viewModes,
    viewLayouts,
    companionSlugs,
    footnoteSentenceSelections,
}: ReaderLayoutProps) {
    const navigate = useNavigate();

    // Suppress scroll-driven URL updates once a close/navigation is in progress,
    // preventing observer-fired navigations from racing with the close navigation.
    const closingRef = useRef(false);

    // Assign stable keys to panels that persist across index changes
    const panelKeysRef = useRef<string[]>([]);
    if (panelKeysRef.current.length < panels.length) {
        while (panelKeysRef.current.length < panels.length) {
            panelKeysRef.current.push(`panel-${nextPanelId++}`);
        }
    }

    interface NavigateOverrides {
        showOriginal?: Set<number>;
        resourceViews?: Map<number, string>;
        viewModes?: Map<number, string>;
        viewLayouts?: Map<number, string>;
        companionSlugs?: Map<number, string>;
        footnoteSentenceSelections?: Map<number, string>;
    }

    /** Navigate changing only search params (no path change) */
    const navigateSearch = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            newResourcesOpen: Set<number>,
            replace?: boolean,
            overrides?: NavigateOverrides,
        ) => {
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: panels[0].bookSlug,
                    nodeSlug: panels[0].nodeSlug!,
                },
                search: buildSearch(
                    newPanels, newSelections, newResourcesOpen,
                    overrides?.showOriginal ?? showOriginal,
                    overrides?.resourceViews ?? resourceViews,
                    overrides?.viewModes ?? viewModes,
                    overrides?.viewLayouts ?? viewLayouts,
                    overrides?.companionSlugs ?? companionSlugs,
                    overrides?.footnoteSentenceSelections ?? footnoteSentenceSelections,
                ),
                replace,
            });
        },
        [navigate, panels, showOriginal, resourceViews, viewModes, viewLayouts, companionSlugs, footnoteSentenceSelections],
    );

    /** Navigate changing the path (primary panel node changed) */
    const navigatePath = useCallback(
        (
            newPanels: PanelState[],
            newSelections: Map<number, string>,
            newResourcesOpen: Set<number>,
            replace?: boolean,
            overrides?: NavigateOverrides,
        ) => {
            const primary = newPanels[0];
            navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params: {
                    bookSlug: primary.bookSlug,
                    nodeSlug: primary.nodeSlug!,
                },
                search: buildSearch(
                    newPanels, newSelections, newResourcesOpen,
                    overrides?.showOriginal ?? showOriginal,
                    overrides?.resourceViews ?? resourceViews,
                    overrides?.viewModes ?? viewModes,
                    overrides?.viewLayouts ?? viewLayouts,
                    overrides?.companionSlugs ?? companionSlugs,
                    overrides?.footnoteSentenceSelections ?? footnoteSentenceSelections,
                ),
                replace,
            });
        },
        [navigate, showOriginal, resourceViews, viewModes, viewLayouts, companionSlugs, footnoteSentenceSelections],
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
            // Clear footnote sentence selection when selecting a new main sentence
            const newFsSelections = new Map(footnoteSentenceSelections);
            newFsSelections.delete(panelIndex);
            navigateSearch(panels, newSelections, newResourcesOpen, false, { footnoteSentenceSelections: newFsSelections });
        },
        [panels, selections, resourcesOpen, footnoteSentenceSelections, navigateSearch],
    );

    const handleSelectFootnoteSentence = useCallback(
        (panelIndex: number, sentenceId: string | undefined) => {
            const newFsSelections = new Map(footnoteSentenceSelections);
            if (sentenceId) {
                newFsSelections.set(panelIndex, sentenceId);
            } else {
                newFsSelections.delete(panelIndex);
            }
            // Open resources panel when selecting a footnote sentence
            const newResourcesOpen = new Set(resourcesOpen);
            if (sentenceId) {
                newResourcesOpen.add(panelIndex);
            }
            navigateSearch(panels, selections, newResourcesOpen, false, { footnoteSentenceSelections: newFsSelections });
        },
        [panels, selections, resourcesOpen, footnoteSentenceSelections, navigateSearch],
    );

    const handleClosePanel = useCallback(
        (panelIndex: number) => {
            closingRef.current = true;
            // Reset after navigation settles so scroll updates resume for remaining panels
            setTimeout(() => { closingRef.current = false; }, 500);
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

            // Shift view mode indices
            const newViewModes = new Map<number, string>();
            for (const [idx, mode] of viewModes) {
                if (idx < panelIndex) newViewModes.set(idx, mode);
                else if (idx > panelIndex) newViewModes.set(idx - 1, mode);
            }

            // Shift view layout indices
            const newViewLayouts = new Map<number, string>();
            for (const [idx, layout] of viewLayouts) {
                if (idx < panelIndex) newViewLayouts.set(idx, layout);
                else if (idx > panelIndex) newViewLayouts.set(idx - 1, layout);
            }

            // Shift companion slug indices
            const newCompanionSlugs = new Map<number, string>();
            for (const [idx, slug] of companionSlugs) {
                if (idx < panelIndex) newCompanionSlugs.set(idx, slug);
                else if (idx > panelIndex) newCompanionSlugs.set(idx - 1, slug);
            }

            // Shift footnote sentence selection indices
            const newFsSelections = new Map<number, string>();
            for (const [idx, id] of footnoteSentenceSelections) {
                if (idx < panelIndex) newFsSelections.set(idx, id);
                else if (idx > panelIndex) newFsSelections.set(idx - 1, id);
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
                    navigatePath(newPanels, newSelections, newResourcesOpen, false, {
                        showOriginal: newShowOriginal,
                        resourceViews: newResourceViews,
                        viewModes: newViewModes,
                        viewLayouts: newViewLayouts,
                        companionSlugs: newCompanionSlugs,
                        footnoteSentenceSelections: newFsSelections,
                    });
                } else {
                    navigate({
                        to: "/books/$bookSlug",
                        params: { bookSlug: primary.bookSlug },
                    });
                }
            }
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, viewModes, viewLayouts, companionSlugs, footnoteSentenceSelections, navigate, navigatePath],
    );

    const handleCloseResources = useCallback(
        (panelIndex: number) => {
            const newResourcesOpen = new Set(resourcesOpen);
            newResourcesOpen.delete(panelIndex);
            // Also deselect sentence and footnote sentence when closing resources
            const newSelections = new Map(selections);
            newSelections.delete(panelIndex);
            const newResourceViews = new Map(resourceViews);
            newResourceViews.delete(panelIndex);
            const newFsSelections = new Map(footnoteSentenceSelections);
            newFsSelections.delete(panelIndex);
            navigateSearch(panels, newSelections, newResourcesOpen, false, { resourceViews: newResourceViews, footnoteSentenceSelections: newFsSelections });
        },
        [panels, selections, resourcesOpen, resourceViews, footnoteSentenceSelections, navigateSearch],
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

            // Shift view mode indices
            const newViewModes = new Map<number, string>();
            for (const [idx, mode] of viewModes) {
                if (idx < insertAt) newViewModes.set(idx, mode);
                else newViewModes.set(idx + 1, mode);
            }

            // Shift view layout indices
            const newViewLayouts = new Map<number, string>();
            for (const [idx, layout] of viewLayouts) {
                if (idx < insertAt) newViewLayouts.set(idx, layout);
                else newViewLayouts.set(idx + 1, layout);
            }

            // Shift companion slug indices
            const newCompanionSlugs = new Map<number, string>();
            for (const [idx, slug] of companionSlugs) {
                if (idx < insertAt) newCompanionSlugs.set(idx, slug);
                else newCompanionSlugs.set(idx + 1, slug);
            }

            // Shift footnote sentence selection indices, clear source panel's
            const newFsSelections = new Map<number, string>();
            for (const [idx, id] of footnoteSentenceSelections) {
                if (idx === afterIndex) continue;
                if (idx < insertAt) newFsSelections.set(idx, id);
                else newFsSelections.set(idx + 1, id);
            }

            // Insert stable key
            panelKeysRef.current = [
                ...panelKeysRef.current.slice(0, insertAt),
                `panel-${nextPanelId++}`,
                ...panelKeysRef.current.slice(insertAt),
            ];

            navigateSearch(newPanels, newSelections, newResourcesOpen, false, {
                showOriginal: newShowOriginal,
                resourceViews: newResourceViews,
                viewModes: newViewModes,
                viewLayouts: newViewLayouts,
                companionSlugs: newCompanionSlugs,
                footnoteSentenceSelections: newFsSelections,
            });
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, viewModes, viewLayouts, companionSlugs, footnoteSentenceSelections, navigateSearch],
    );

    /** Replace-navigate for scroll-driven URL updates (no history entry) */
    const handleScrollNavigate = useCallback(
        (panelIndex: number, nodeSlug: string) => {
            if (closingRef.current) return;
            // Layout shifts (e.g. UserSubnav appearing during a pending
            // navigation away from the reader) can fire the IntersectionObserver
            // with a different visible slug. If the URL has already moved off
            // the reader route, don't replace it back.
            if (!window.location.pathname.startsWith(`/books/${panels[0].bookSlug}/`)) {
                return;
            }
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
            navigateSearch(panels, selections, resourcesOpen, false, { showOriginal: newShowOriginal });
        },
        [panels, selections, resourcesOpen, showOriginal, navigateSearch],
    );

    const handleResourceViewChange = useCallback(
        (panelIndex: number, view: string | undefined) => {
            const newResourceViews = new Map(resourceViews);
            if (view) {
                newResourceViews.set(panelIndex, view);
            } else {
                newResourceViews.delete(panelIndex);
            }
            navigateSearch(panels, selections, resourcesOpen, false, { resourceViews: newResourceViews });
        },
        [panels, selections, resourcesOpen, resourceViews, navigateSearch],
    );

    const handleViewModeChange = useCallback(
        (panelIndex: number, mode: string, companionSlug?: string, targetNodeSlug?: string) => {
            const newViewModes = new Map(viewModes);
            const newViewLayouts = new Map(viewLayouts);
            const newCompanionSlugs = new Map(companionSlugs);

            if (mode === "st") {
                newViewModes.set(panelIndex, "st");
                if (!newViewLayouts.has(panelIndex)) {
                    newViewLayouts.set(panelIndex, "sp");
                }
                if (companionSlug) {
                    newCompanionSlugs.set(panelIndex, companionSlug);
                }
                navigateSearch(panels, selections, resourcesOpen, false, {
                    viewModes: newViewModes,
                    viewLayouts: newViewLayouts,
                    companionSlugs: newCompanionSlugs,
                });
            } else if (companionSlug) {
                // "s" or "t" with a target — navigate to the target book, clear interleaved state
                newViewModes.delete(panelIndex);
                newViewLayouts.delete(panelIndex);
                newCompanionSlugs.delete(panelIndex);
                const newPanels = panels.map((p, i) =>
                    i === panelIndex ? { bookSlug: companionSlug, nodeSlug: targetNodeSlug } : p,
                );
                if (panelIndex === 0) {
                    if (targetNodeSlug) {
                        navigate({
                            to: "/books/$bookSlug/$nodeSlug",
                            params: { bookSlug: companionSlug, nodeSlug: targetNodeSlug },
                        });
                    } else {
                        navigate({
                            to: "/books/$bookSlug",
                            params: { bookSlug: companionSlug },
                        });
                    }
                } else {
                    navigateSearch(newPanels, selections, resourcesOpen, false, {
                        viewModes: newViewModes,
                        viewLayouts: newViewLayouts,
                        companionSlugs: newCompanionSlugs,
                    });
                }
            } else {
                // "s" or "t" without navigation — just clear interleaved state
                // Navigate to current path with updated search to preserve scroll position
                newViewModes.delete(panelIndex);
                newViewLayouts.delete(panelIndex);
                newCompanionSlugs.delete(panelIndex);
                navigate({
                    to: ".",
                    search: buildSearch(
                        panels, selections, resourcesOpen,
                        showOriginal, resourceViews,
                        newViewModes, newViewLayouts, newCompanionSlugs,
                        footnoteSentenceSelections,
                    ),
                });
            }
        },
        [panels, selections, resourcesOpen, showOriginal, resourceViews, viewModes, viewLayouts, companionSlugs, footnoteSentenceSelections, navigateSearch, navigate],
    );

    const handleViewLayoutChange = useCallback(
        (panelIndex: number, layout: string) => {
            const newViewLayouts = new Map(viewLayouts);
            newViewLayouts.set(panelIndex, layout);
            navigateSearch(panels, selections, resourcesOpen, false, { viewLayouts: newViewLayouts });
        },
        [panels, selections, resourcesOpen, viewLayouts, navigateSearch],
    );

    const canAddPanel = panels.length < 4;

    return (
        <div className="flex h-full">
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
                    viewMode={viewModes.get(idx)}
                    viewLayout={viewLayouts.get(idx)}
                    companionSlug={companionSlugs.get(idx)}
                    footnoteSentenceId={footnoteSentenceSelections.get(idx)}
                    onSelectSentence={(sentenceId) =>
                        handleSelectSentence(idx, sentenceId)
                    }
                    onSelectFootnoteSentence={(id) =>
                        handleSelectFootnoteSentence(idx, id)
                    }
                    onToggleOriginal={() => handleToggleOriginal(idx)}
                    onToggleResources={() => handleCloseResources(idx)}
                    onResourceViewChange={(view) => handleResourceViewChange(idx, view)}
                    onViewModeChange={(mode, companionSlug, targetNodeSlug) =>
                        handleViewModeChange(idx, mode, companionSlug, targetNodeSlug)
                    }
                    onViewLayoutChange={(layout) =>
                        handleViewLayoutChange(idx, layout)
                    }
                    onClose={
                        panels.length > 1
                            ? () => handleClosePanel(idx)
                            : () => {
                                  closingRef.current = true;
                                  navigate({ to: "/" });
                              }
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
