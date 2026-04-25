import { useNavigate } from "@tanstack/react-router";
import { useCallback, useRef } from "react";
import { createPanel, type Panel } from "./state";
import { TextPanel } from "./TextPanel";
import { encode } from "./url";

interface ReaderLayoutProps {
    panels: Panel[];
}

let nextPanelId = 0;

export function ReaderLayout({ panels }: ReaderLayoutProps) {
    const navigate = useNavigate();

    // Suppress scroll-driven URL updates once a close/navigation is in progress,
    // preventing observer-fired navigations from racing with the close navigation.
    const closingRef = useRef(false);

    // Stable keys for panel React reconciliation across index shifts
    const panelKeysRef = useRef<string[]>([]);
    while (panelKeysRef.current.length < panels.length) {
        panelKeysRef.current.push(`panel-${nextPanelId++}`);
    }

    const goto = useCallback(
        (
            newPanels: Panel[],
            opts?: { replace?: boolean; pathChanged?: boolean },
        ) => {
            const url = encode({ panels: newPanels });
            if (opts?.pathChanged) {
                if (url.nodeSlug) {
                    navigate({
                        to: "/books/$bookSlug/$nodeSlug",
                        params: {
                            bookSlug: url.bookSlug,
                            nodeSlug: url.nodeSlug,
                        },
                        search: url.search,
                        replace: opts?.replace,
                    });
                } else {
                    navigate({
                        to: "/books/$bookSlug",
                        params: { bookSlug: url.bookSlug },
                    });
                }
            } else {
                navigate({
                    to: ".",
                    search: url.search,
                    replace: opts?.replace,
                });
            }
        },
        [navigate],
    );

    const updatePanel = useCallback(
        (idx: number, updates: Partial<Panel>): Panel[] =>
            panels.map((p, i) => (i === idx ? { ...p, ...updates } : p)),
        [panels],
    );

    const handleSelectSentence = useCallback(
        (panelIndex: number, sentenceId: string) => {
            const current = panels[panelIndex];
            const next =
                current.selectedSentenceId === sentenceId
                    ? undefined
                    : sentenceId;
            goto(
                updatePanel(panelIndex, {
                    selectedSentenceId: next,
                    resourcesOpen: true,
                    footnoteSentenceId: undefined,
                }),
            );
        },
        [panels, updatePanel, goto],
    );

    const handleSelectFootnoteSentence = useCallback(
        (panelIndex: number, sentenceId: string | undefined) => {
            goto(
                updatePanel(panelIndex, {
                    footnoteSentenceId: sentenceId,
                    resourcesOpen: sentenceId
                        ? true
                        : panels[panelIndex].resourcesOpen,
                }),
            );
        },
        [panels, updatePanel, goto],
    );

    const handleClosePanel = useCallback(
        (panelIndex: number) => {
            closingRef.current = true;
            // Reset after navigation settles so scroll updates resume for remaining panels
            setTimeout(() => {
                closingRef.current = false;
            }, 500);

            const newPanels = panels.filter((_, i) => i !== panelIndex);
            panelKeysRef.current = panelKeysRef.current.filter(
                (_, i) => i !== panelIndex,
            );

            if (newPanels.length === 0) {
                navigate({
                    to: "/books/$bookSlug",
                    params: { bookSlug: panels[0].bookSlug },
                });
                return;
            }

            const primary = newPanels[0];
            if (primary.nodeSlug) {
                goto(newPanels, { pathChanged: true });
            } else {
                navigate({
                    to: "/books/$bookSlug",
                    params: { bookSlug: primary.bookSlug },
                });
            }
        },
        [panels, navigate, goto],
    );

    const handleCloseResources = useCallback(
        (panelIndex: number) => {
            // Closing resources also clears selection + footnote selection + view
            goto(
                updatePanel(panelIndex, {
                    resourcesOpen: false,
                    selectedSentenceId: undefined,
                    footnoteSentenceId: undefined,
                    resourceView: undefined,
                }),
            );
        },
        [updatePanel, goto],
    );

    const handleAddComparisonPanel = useCallback(
        (afterIndex: number, bookSlug: string, nodeSlug: string) => {
            const insertAt = afterIndex + 1;
            // The source panel that triggered the add: clear its selection/resources/view
            const cleared: Panel = {
                ...panels[afterIndex],
                selectedSentenceId: undefined,
                resourcesOpen: false,
                resourceView: undefined,
                footnoteSentenceId: undefined,
            };
            const newPanels: Panel[] = [
                ...panels.slice(0, afterIndex),
                cleared,
                createPanel(bookSlug, nodeSlug),
                ...panels.slice(insertAt),
            ];

            panelKeysRef.current = [
                ...panelKeysRef.current.slice(0, insertAt),
                `panel-${nextPanelId++}`,
                ...panelKeysRef.current.slice(insertAt),
            ];

            goto(newPanels);
        },
        [panels, goto],
    );

    /** Replace-navigate for scroll-driven URL updates (no history entry) */
    const handleScrollNavigate = useCallback(
        (panelIndex: number, nodeSlug: string) => {
            if (closingRef.current) return;
            // Layout shifts (e.g. UserSubnav appearing during a pending
            // navigation away from the reader) can fire the IntersectionObserver
            // with a different visible slug. If the URL has already moved off
            // the reader route, don't replace it back.
            if (
                !window.location.pathname.startsWith(
                    `/books/${panels[0].bookSlug}/`,
                )
            ) {
                return;
            }
            goto(updatePanel(panelIndex, { nodeSlug }), {
                replace: true,
                pathChanged: panelIndex === 0,
            });
        },
        [panels, updatePanel, goto],
    );

    const handleToggleOriginal = useCallback(
        (panelIndex: number) => {
            goto(
                updatePanel(panelIndex, {
                    showOriginal: !panels[panelIndex].showOriginal,
                }),
            );
        },
        [panels, updatePanel, goto],
    );

    const handleResourceViewChange = useCallback(
        (panelIndex: number, view: string | undefined) => {
            goto(updatePanel(panelIndex, { resourceView: view }));
        },
        [updatePanel, goto],
    );

    const handleViewModeChange = useCallback(
        (
            panelIndex: number,
            mode: string,
            companionSlug?: string,
            targetNodeSlug?: string,
        ) => {
            const current = panels[panelIndex];

            if (mode === "st") {
                goto(
                    updatePanel(panelIndex, {
                        viewMode: "st",
                        viewLayout: current.viewLayout ?? "sp",
                        companionSlug: companionSlug ?? current.companionSlug,
                    }),
                );
                return;
            }

            if (companionSlug) {
                // "s" or "t" with target — switch panel to the companion book.
                // Primary panel: drop all secondary state (matches prior behavior).
                if (panelIndex === 0) {
                    if (targetNodeSlug) {
                        navigate({
                            to: "/books/$bookSlug/$nodeSlug",
                            params: {
                                bookSlug: companionSlug,
                                nodeSlug: targetNodeSlug,
                            },
                        });
                    } else {
                        navigate({
                            to: "/books/$bookSlug",
                            params: { bookSlug: companionSlug },
                        });
                    }
                    return;
                }
                goto(
                    panels.map((p, i) =>
                        i === panelIndex
                            ? {
                                  ...p,
                                  bookSlug: companionSlug,
                                  nodeSlug: targetNodeSlug,
                                  viewMode: undefined,
                                  viewLayout: undefined,
                                  companionSlug: undefined,
                              }
                            : p,
                    ),
                );
                return;
            }

            // "s" or "t" without target — clear interleaved state, keep book/node
            goto(
                updatePanel(panelIndex, {
                    viewMode: undefined,
                    viewLayout: undefined,
                    companionSlug: undefined,
                }),
            );
        },
        [panels, updatePanel, navigate, goto],
    );

    const handleViewLayoutChange = useCallback(
        (panelIndex: number, layout: string) => {
            goto(
                updatePanel(panelIndex, {
                    viewLayout: layout as Panel["viewLayout"],
                }),
            );
        },
        [updatePanel, goto],
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
                    resourcesOpen={panel.resourcesOpen}
                    resourceView={panel.resourceView}
                    selectedSentenceId={panel.selectedSentenceId}
                    showOriginal={panel.showOriginal}
                    viewMode={panel.viewMode}
                    viewLayout={panel.viewLayout}
                    companionSlug={panel.companionSlug}
                    footnoteSentenceId={panel.footnoteSentenceId}
                    onSelectSentence={(sentenceId) =>
                        handleSelectSentence(idx, sentenceId)
                    }
                    onSelectFootnoteSentence={(id) =>
                        handleSelectFootnoteSentence(idx, id)
                    }
                    onToggleOriginal={() => handleToggleOriginal(idx)}
                    onToggleResources={() => handleCloseResources(idx)}
                    onResourceViewChange={(view) =>
                        handleResourceViewChange(idx, view)
                    }
                    onViewModeChange={(mode, companionSlug, targetNodeSlug) =>
                        handleViewModeChange(
                            idx,
                            mode,
                            companionSlug,
                            targetNodeSlug,
                        )
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
