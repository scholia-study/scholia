import { Paper } from "@mui/material";
import {
    keepPreviousData,
    useInfiniteQuery,
    useQuery,
} from "@tanstack/react-query";
import {
    forwardRef,
    useCallback,
    useEffect,
    useImperativeHandle,
    useMemo,
    useRef,
    useState,
} from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import type { NodeDetail, SentenceResponse } from "../../../api/model";
import { getNodePage } from "../../../api/nodes/nodes";
import { getNodePageQueryOptions } from "../nodePageQuery";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";
import { InterleavedNodeRenderer } from "./InterleavedNodeRenderer";

// Virtuoso uses absolute item indices; when prepending we decrement
// `firstItemIndex` by the number of new items so the library can keep
// the user's visible content pinned. Starting high gives plenty of
// runway downward (a Bible has ~31k verses if every chapter were
// loaded; 100k buffer is more than enough).
const FIRST_ITEM_INDEX_INITIAL = 100_000;

export interface PanelScrollViewHandle {
    scrollToNode: (nodeSlug: string, sortOrder?: number) => void;
    getSentencesInRange: (start: number, end: number) => SentenceResponse[];
}

interface PanelScrollViewProps {
    bookSlug: string;
    initialNodeSlug: string | undefined;
    initialSortOrder: number | undefined;
    selectedSentenceId: string | undefined;
    showOriginal: boolean;
    viewMode?: string;
    viewLayout?: string;
    companionSlug?: string;
    primaryLabel?: string;
    companionLabel?: string;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    onVisibleNodeChange?: (nodeSlug: string) => void;
    onSystemsDiscovered?: (systems: string[]) => void;
    marginSettings?: MarginSettings;
}

export const PanelScrollView = forwardRef<
    PanelScrollViewHandle,
    PanelScrollViewProps
>(function PanelScrollView(
    {
        bookSlug,
        initialNodeSlug,
        initialSortOrder,
        selectedSentenceId,
        showOriginal,
        viewMode,
        viewLayout,
        companionSlug,
        primaryLabel,
        companionLabel,
        onSelectSentence,
        onVisibleNodeChange,
        onSystemsDiscovered,
        marginSettings,
    },
    ref,
) {
    // startSortOrder drives the initial page param for the query.
    // Changing it restarts the query from a new position.
    const [startSortOrder, setStartSortOrder] = useState<number | undefined>(
        initialSortOrder,
    );

    // If initialSortOrder arrives after mount (toc loaded late), update so
    // the query starts at the right position.
    useEffect(() => {
        if (initialSortOrder != null && startSortOrder == null) {
            setStartSortOrder(initialSortOrder);
        }
    }, [initialSortOrder, startSortOrder]);

    const {
        data,
        hasNextPage,
        hasPreviousPage,
        isFetchingNextPage,
        isFetchingPreviousPage,
        fetchNextPage,
        fetchPreviousPage,
        isLoading,
        error,
    } = useInfiniteQuery(
        getNodePageQueryOptions({ bookSlug, showOriginal, startSortOrder }),
    );

    const nodes = useMemo(
        () =>
            data?.pages.flatMap((page) =>
                page.status === 200 ? page.data.nodes : [],
            ) ?? [],
        [data],
    );

    // Track which slugs we emitted via rangeChanged. When initialNodeSlug
    // changes back to one we just emitted (URL sync from our own scroll),
    // we know it's a self-update — don't trigger a scroll.
    const lastEmittedSlugRef = useRef<string | null>(null);

    // Restart query when the user navigates to a node not in the current
    // data window (e.g. TOC click to a far-away chapter).
    const [prevNodeSlug, setPrevNodeSlug] = useState(initialNodeSlug);
    const [pendingScrollTarget, setPendingScrollTarget] = useState<
        string | null
    >(null);
    if (initialNodeSlug !== prevNodeSlug) {
        setPrevNodeSlug(initialNodeSlug);
        const isLoaded = nodes.some((n) => n.slug === initialNodeSlug);
        if (!isLoaded && initialSortOrder != null) {
            setStartSortOrder(initialSortOrder);
            setPendingScrollTarget(initialNodeSlug ?? null);
        } else if (isLoaded && initialNodeSlug) {
            // Same query; just scroll. Whether to actually scroll or
            // not is decided by `lastEmittedSlugRef` — if this URL
            // change came from the user scrolling into the node, we
            // should NOT snap them.
            if (initialNodeSlug !== lastEmittedSlugRef.current) {
                setPendingScrollTarget(initialNodeSlug);
            }
            lastEmittedSlugRef.current = null;
        }
    }

    // Discover reference systems from loaded nodes (drives margin settings).
    useEffect(() => {
        if (!onSystemsDiscovered || nodes.length === 0) return;
        const systems = new Set<string>();
        for (const node of nodes) {
            for (const block of node.blocks) {
                for (const sentence of block.sentences) {
                    for (const pm of sentence.page_markers) {
                        systems.add(pm.system_slug);
                    }
                }
            }
        }
        if (systems.size > 0) onSystemsDiscovered(Array.from(systems));
    }, [nodes, onSystemsDiscovered]);

    // Companion data fetching for interleaved view (Kant DE↔EN).
    const companionFetchParams = useMemo(() => {
        if (viewMode !== "st" || !companionSlug || nodes.length === 0)
            return null;
        const primaryIsTranslation = nodes.some((n) => n.source_node_id);
        if (primaryIsTranslation) {
            const ids = nodes
                .map((n) => n.source_node_id)
                .filter(Boolean)
                .join(",");
            return ids ? { key: "node_ids" as const, ids } : null;
        }
        const ids = nodes.map((n) => n.id).join(",");
        return { key: "source_nodes" as const, ids };
    }, [viewMode, companionSlug, nodes]);

    const { data: companionData } = useQuery({
        enabled: !!companionFetchParams,
        queryKey: [
            "companion-nodes",
            companionSlug,
            companionFetchParams?.key,
            companionFetchParams?.ids,
            showOriginal ? "og" : "",
        ],
        queryFn: async ({ signal }) => {
            const base = showOriginal ? { original: true } : {};
            const params =
                companionFetchParams!.key === "node_ids"
                    ? { ...base, node_ids: companionFetchParams!.ids }
                    : { ...base, source_nodes: companionFetchParams!.ids };
            return getNodePage(companionSlug!, params, { signal });
        },
        placeholderData: keepPreviousData,
    });

    const companionNodeMap = useMemo(() => {
        if (!companionData || companionData.status !== 200) return undefined;
        const map = new Map<string, NodeDetail>();
        const primaryIsTranslation = nodes.some((n) => n.source_node_id);
        for (const node of companionData.data.nodes) {
            if (primaryIsTranslation) {
                for (const pn of nodes) {
                    if (pn.source_node_id === node.id) {
                        map.set(pn.id, node);
                    }
                }
            } else if (node.source_node_id) {
                map.set(node.source_node_id, node);
            }
        }
        return map;
    }, [companionData, nodes]);

    // ── Virtuoso bookkeeping ────────────────────────────────────
    //
    // `firstItemIndex` is the absolute index of the first item in `data`.
    // We decrement it whenever items are prepended so Virtuoso can
    // preserve the user's visible content. Tracked via a ref so we can
    // update it synchronously during render (it must move in lockstep
    // with the `data` array — otherwise Virtuoso mis-anchors).
    const firstItemIndexRef = useRef(FIRST_ITEM_INDEX_INITIAL);
    const prevFirstNodeIdRef = useRef<string | null>(null);
    if (nodes.length > 0) {
        const newFirstId = nodes[0].id;
        if (prevFirstNodeIdRef.current === null) {
            prevFirstNodeIdRef.current = newFirstId;
        } else if (prevFirstNodeIdRef.current !== newFirstId) {
            // First node changed — items were either prepended OR the
            // query restarted. Find where the previous first node now
            // lives in the new data; that's the prepend count.
            const prevFirstNewIdx = nodes.findIndex(
                (n) => n.id === prevFirstNodeIdRef.current,
            );
            if (prevFirstNewIdx > 0) {
                firstItemIndexRef.current -= prevFirstNewIdx;
            } else {
                // Previous first not present (full restart); reset.
                firstItemIndexRef.current = FIRST_ITEM_INDEX_INITIAL;
            }
            prevFirstNodeIdRef.current = newFirstId;
        }
    } else {
        prevFirstNodeIdRef.current = null;
    }
    const firstItemIndex = firstItemIndexRef.current;

    const virtuosoRef = useRef<VirtuosoHandle>(null);

    // Initial scroll target. Virtuoso's `initialTopMostItemIndex` is
    // **relative** to the `data` array (not factoring `firstItemIndex`).
    // Computed once when nodes are first available; subsequent
    // navigation uses `scrollToIndex` via the imperative handle.
    const initialIndexRef = useRef<number | null>(null);
    if (initialIndexRef.current === null && nodes.length > 0) {
        if (initialNodeSlug) {
            const relIdx = nodes.findIndex((n) => n.slug === initialNodeSlug);
            initialIndexRef.current = relIdx >= 0 ? relIdx : 0;
        } else {
            initialIndexRef.current = 0;
        }
    }

    // Scroll to a target node once it's loaded into `nodes`. Used both
    // for cross-data-window jumps (after `setStartSortOrder` restarts
    // the query) and for clicks within the current window. Virtuoso's
    // `scrollToIndex` takes an index **relative** to `data`.
    useEffect(() => {
        if (!pendingScrollTarget) return;
        const relIdx = nodes.findIndex((n) => n.slug === pendingScrollTarget);
        if (relIdx < 0) return; // wait for data
        virtuosoRef.current?.scrollToIndex({
            index: relIdx,
            align: "start",
            behavior: "auto",
        });
        // Brief delay to let Virtuoso settle, then drop the loading curtain.
        const timer = setTimeout(() => {
            setPendingScrollTarget(null);
        }, 50);
        return () => clearTimeout(timer);
    }, [pendingScrollTarget, nodes]);

    // After landing on the target node, scroll to the selected sentence
    // (only on initial load from URL params).
    const pendingSentenceScroll = useRef<string | null>(
        selectedSentenceId ?? null,
    );
    useEffect(() => {
        if (pendingScrollTarget) return;
        if (!pendingSentenceScroll.current) return;
        const key = pendingSentenceScroll.current;
        // Wait a frame for Virtuoso to commit the scrolled-to node.
        const raf = requestAnimationFrame(() => {
            const el = document.querySelector(
                `[data-sentence-key="${CSS.escape(key)}"]`,
            );
            if (el) {
                pendingSentenceScroll.current = null;
                for (const node of nodes) {
                    for (const block of node.blocks) {
                        for (const sentence of block.sentences) {
                            if (
                                sentence.id === key ||
                                (sentence.sentence_number != null &&
                                    String(sentence.sentence_number) === key)
                            ) {
                                onSelectSentence(sentence, false);
                                break;
                            }
                        }
                    }
                }
                el.scrollIntoView({ block: "center" });
            }
        });
        return () => cancelAnimationFrame(raf);
    }, [pendingScrollTarget, nodes, onSelectSentence]);

    // Imperative handle.
    useImperativeHandle(
        ref,
        () => ({
            getSentencesInRange(start: number, end: number) {
                const result: SentenceResponse[] = [];
                for (const node of nodes) {
                    for (const block of node.blocks) {
                        for (const sentence of block.sentences) {
                            if (
                                sentence.sentence_number != null &&
                                sentence.sentence_number >= start &&
                                sentence.sentence_number <= end
                            ) {
                                result.push(sentence);
                            }
                        }
                    }
                }
                return result.sort(
                    (a, b) => a.sentence_number! - b.sentence_number!,
                );
            },
            scrollToNode(nodeSlug: string, sortOrder?: number) {
                const relIdx = nodes.findIndex((n) => n.slug === nodeSlug);
                if (relIdx >= 0) {
                    virtuosoRef.current?.scrollToIndex({
                        index: relIdx,
                        align: "start",
                        behavior: "auto",
                    });
                } else if (sortOrder != null) {
                    setStartSortOrder(sortOrder);
                    setPendingScrollTarget(nodeSlug);
                } else {
                    setPendingScrollTarget(nodeSlug);
                }
            },
        }),
        [nodes],
    );

    // Visible-node tracking for URL sync. Virtuoso's `rangeChanged`
    // reports the first *partially* visible item — that lags by one
    // section because the previous chapter's tail is still touching the
    // top while the user has already scrolled into the next. Use an
    // IntersectionObserver with a top-slice rootMargin so only the
    // upper portion of the viewport counts, matching the previous
    // pre-Virtuoso behavior.
    const onVisibleNodeChangeRef = useRef(onVisibleNodeChange);
    onVisibleNodeChangeRef.current = onVisibleNodeChange;
    const [scrollerEl, setScrollerEl] = useState<HTMLElement | null>(null);

    useEffect(() => {
        if (!scrollerEl) return;

        // Track every item currently intersecting the slice (not just the
        // ones that transitioned in this callback). At a section boundary
        // both the previous chapter's tail and the next chapter's heading
        // can be in the slice simultaneously; we pick the one whose
        // *top* is highest in the viewport — that's the section whose
        // heading just came into reading view, i.e. what the user is now
        // at. Debounced so brief scroll-jitter at the threshold doesn't
        // ping-pong the URL.
        const intersecting = new Map<HTMLElement, number>(); // el → top
        let pendingSlug: string | null = null;
        let timer: ReturnType<typeof setTimeout> | null = null;

        const flush = () => {
            timer = null;
            if (pendingSlug && pendingSlug !== lastEmittedSlugRef.current) {
                lastEmittedSlugRef.current = pendingSlug;
                onVisibleNodeChangeRef.current?.(pendingSlug);
            }
        };

        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    const el = entry.target as HTMLElement;
                    if (entry.isIntersecting) {
                        intersecting.set(el, entry.boundingClientRect.top);
                    } else {
                        intersecting.delete(el);
                    }
                }
                if (intersecting.size === 0) return;
                // Pick the entry with the largest `top` — most recently
                // scrolled into the reading slice from below.
                let best: HTMLElement | null = null;
                let bestTop = -Infinity;
                for (const [el, top] of intersecting) {
                    if (top > bestTop) {
                        bestTop = top;
                        best = el;
                    }
                }
                const slug = best?.dataset.nodeSlug;
                if (!slug) return;
                pendingSlug = slug;
                if (timer) clearTimeout(timer);
                timer = setTimeout(flush, 150);
            },
            {
                root: scrollerEl,
                rootMargin: "-10% 0px -80% 0px",
            },
        );

        const observeAll = () => {
            observer.disconnect();
            intersecting.clear();
            const els = scrollerEl.querySelectorAll("[data-node-slug]");
            for (const el of els) observer.observe(el);
        };

        observeAll();

        // Watch for new items mounting/unmounting as the user scrolls.
        const mutationObserver = new MutationObserver(() => observeAll());
        mutationObserver.observe(scrollerEl, {
            childList: true,
            subtree: true,
        });

        return () => {
            observer.disconnect();
            mutationObserver.disconnect();
            if (timer) clearTimeout(timer);
        };
    }, [scrollerEl]);

    const handleStartReached = useCallback(() => {
        if (hasPreviousPage && !isFetchingPreviousPage) {
            fetchPreviousPage();
        }
    }, [hasPreviousPage, isFetchingPreviousPage, fetchPreviousPage]);

    const handleEndReached = useCallback(() => {
        if (hasNextPage && !isFetchingNextPage) {
            fetchNextPage();
        }
    }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

    const hasActiveMargins =
        marginSettings && marginSettings.enabledSystems.size > 0;
    const isInterleaved = viewMode === "st";
    const isSideBySide =
        viewLayout === "bpl" ||
        viewLayout === "bpr" ||
        viewLayout === "bsl" ||
        viewLayout === "bsr";

    const containerClass =
        isInterleaved && isSideBySide
            ? "max-w-5xl mx-auto px-8"
            : hasActiveMargins
              ? "max-w-4xl mx-auto"
              : "max-w-2xl mx-auto px-8";

    const renderItem = useCallback(
        (_index: number, node: NodeDetail) => {
            const companion = companionNodeMap?.get(node.id);
            return (
                <div
                    data-node-slug={node.slug}
                    // `flow-root` establishes a block formatting context
                    // so any margin-top/bottom from descendants stays
                    // inside this box. Virtuoso measures items via
                    // ResizeObserver's `contentRect`, which excludes
                    // escaped margins — so trapping them here keeps
                    // measurements honest. (See virtuoso troubleshooting
                    // §2: "Items jump / list won't scroll to bottom".)
                    className={`flow-root ${containerClass}`}
                >
                    <div
                        className={`py-8 border-b border-stone-100 ${hasActiveMargins && !isInterleaved ? "max-w-2xl mx-auto px-8" : ""}`}
                    >
                        {isInterleaved ? (
                            <InterleavedNodeRenderer
                                primaryNode={node}
                                companionNode={companion}
                                viewLayout={
                                    (viewLayout ?? "sp") as
                                        | "sp"
                                        | "ss"
                                        | "bpl"
                                        | "bpr"
                                        | "bsl"
                                        | "bsr"
                                }
                                selectedSentenceId={selectedSentenceId ?? null}
                                showOriginal={showOriginal}
                                onSelectSentence={onSelectSentence}
                                marginSettings={marginSettings}
                                primaryLabel={primaryLabel ?? "Source"}
                                companionLabel={companionLabel ?? "Translation"}
                            />
                        ) : (
                            node.blocks.map((block) => (
                                <Block
                                    key={block.id}
                                    block={block}
                                    selectedSentenceId={
                                        selectedSentenceId ?? null
                                    }
                                    showOriginal={showOriginal}
                                    onSelectSentence={onSelectSentence}
                                    marginSettings={marginSettings}
                                    nodeSourceRef={node.source_ref}
                                />
                            ))
                        )}
                    </div>
                </div>
            );
        },
        [
            companionNodeMap,
            containerClass,
            hasActiveMargins,
            isInterleaved,
            viewLayout,
            selectedSentenceId,
            showOriginal,
            onSelectSentence,
            marginSettings,
            primaryLabel,
            companionLabel,
        ],
    );

    if (isLoading) {
        return (
            <Paper
                square
                elevation={0}
                className="flex items-center justify-center h-full text-stone-400"
            >
                <p>Loading...</p>
            </Paper>
        );
    }

    if (error) {
        return (
            <Paper
                square
                elevation={0}
                className="flex items-center justify-center h-full text-red-500"
            >
                <p>Failed to load content.</p>
            </Paper>
        );
    }

    return (
        <Paper square elevation={0} className="h-full relative">
            {pendingScrollTarget && (
                <div className="absolute inset-0 z-10 flex items-center justify-center bg-stone-50">
                    <p className="text-stone-400">Loading...</p>
                </div>
            )}
            <Virtuoso
                ref={virtuosoRef}
                data={nodes}
                firstItemIndex={firstItemIndex}
                initialTopMostItemIndex={initialIndexRef.current ?? 0}
                computeItemKey={(_index, node) => node.id}
                startReached={handleStartReached}
                endReached={handleEndReached}
                scrollerRef={(el) => {
                    setScrollerEl(el instanceof HTMLElement ? el : null);
                }}
                itemContent={renderItem}
                // Skip the "probe" render Virtuoso does with the first
                // item to figure out a default height. Without this,
                // there's a brief layout pass where unmeasured items
                // are sized at 0 and the page can flash white.
                defaultItemHeight={800}
                // Render a generous zone above and below the viewport
                // so prepended chunks are already rendered (and
                // measured) before the user scrolls into them. Eliminates
                // the gap between Virtuoso saying "you've reached the
                // top, fetching more" and the new content actually
                // appearing.
                increaseViewportBy={{ top: 2000, bottom: 2000 }}
                style={{ height: "100%" }}
                components={{
                    Header: () =>
                        isFetchingPreviousPage ? (
                            <div className="flex justify-center py-4 text-stone-400">
                                <p>Loading earlier…</p>
                            </div>
                        ) : null,
                    Footer: () =>
                        isFetchingNextPage ? (
                            <div className="flex justify-center py-4 text-stone-400">
                                <p>Loading more…</p>
                            </div>
                        ) : null,
                }}
            />
        </Paper>
    );
});
