import { Paper } from "@mui/material";
import type { InfiniteData } from "@tanstack/react-query";
import {
    keepPreviousData,
    useInfiniteQuery,
    useQuery,
} from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
    forwardRef,
    useCallback,
    useEffect,
    useImperativeHandle,
    useMemo,
    useRef,
    useState,
} from "react";
import type { NodeDetail, SentenceResponse } from "../../../api/model";
import {
    getNodePage,
    type getNodePageResponse,
} from "../../../api/nodes/nodes";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";
import { InterleavedNodeRenderer } from "./InterleavedNodeRenderer";

type PageCursor = { after: number } | { before: number };

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

    const [prevNodeSlug, setPrevNodeSlug] = useState(initialNodeSlug);

    // If initialSortOrder arrives after mount (toc loaded late),
    // update so the query starts at the right position.
    useEffect(() => {
        if (initialSortOrder != null && startSortOrder == null) {
            setStartSortOrder(initialSortOrder);
        }
    }, [initialSortOrder, startSortOrder]);

    // Don't fire the query until we know where to start.
    // If there's a target node but no sort order yet, the toc is still loading.
    const waitingForSortOrder =
        initialNodeSlug != null && startSortOrder == null;

    // Load a few nodes before the target so scrolling up doesn't immediately
    // trigger a backward fetch (which fights with scroll anchoring after TOC jumps).
    const PREFETCH_BUFFER = 5;
    const initialPageParam: PageCursor | undefined =
        startSortOrder != null
            ? { after: Math.max(0, startSortOrder - 1 - PREFETCH_BUFFER) }
            : undefined;

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
    } = useInfiniteQuery<
        getNodePageResponse,
        Error,
        InfiniteData<getNodePageResponse, PageCursor | undefined>,
        string[],
        PageCursor | undefined
    >({
        enabled: !waitingForSortOrder,
        queryKey: [
            "node-page-bidir",
            bookSlug,
            String(startSortOrder),
            showOriginal ? "og" : "",
        ],
        queryFn: async ({ pageParam, signal }) => {
            const base = showOriginal
                ? { limit: 20, original: true }
                : { limit: 20 };
            const params = pageParam
                ? "after" in pageParam
                    ? { ...base, after: pageParam.after }
                    : { ...base, before: pageParam.before }
                : base;
            return getNodePage(bookSlug, params, { signal });
        },
        initialPageParam,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            const page = lastPage.data;
            if (!page.has_more || page.nodes.length === 0) return undefined;
            return {
                after: page.nodes[page.nodes.length - 1].sort_order,
            };
        },
        getPreviousPageParam: (firstPage) => {
            if (firstPage.status !== 200) return undefined;
            const page = firstPage.data;
            if (!page.has_previous || page.nodes.length === 0) return undefined;
            return { before: page.nodes[0].sort_order };
        },
    });

    const nodes = useMemo(
        () =>
            data?.pages.flatMap((page) =>
                page.status === 200 ? page.data.nodes : [],
            ) ?? [],
        [data],
    );

    if (initialNodeSlug !== prevNodeSlug) {
        setPrevNodeSlug(initialNodeSlug);

        // If the user navigates to a node that isn't in our current infinite list,
        // it's a major jump. Update startSortOrder immediately to trigger a new query
        // and prevent the flash of old content.
        const isLoaded = nodes.some((n) => n.slug === initialNodeSlug);
        if (!isLoaded && initialSortOrder != null) {
            setStartSortOrder(initialSortOrder);
        }
    }

    // Discover reference systems from loaded nodes
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

    // Companion data fetching for interleaved view
    // Determine fetch direction: if primary nodes have source_node_id, primary is a translation
    // and we fetch the source by node_ids. Otherwise, primary is a source and we fetch
    // the translation by source_nodes.
    const companionFetchParams = useMemo(() => {
        if (viewMode !== "st" || !companionSlug || nodes.length === 0)
            return null;
        const primaryIsTranslation = nodes.some((n) => n.source_node_id);
        if (primaryIsTranslation) {
            // Primary's source_node_id values point to the companion (source) nodes
            const ids = nodes
                .map((n) => n.source_node_id)
                .filter(Boolean)
                .join(",");
            return ids ? { key: "node_ids" as const, ids } : null;
        }
        // Primary is source — companion's source_node_id points to primary
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
                // Companion is source — map by companion's own id (primary's source_node_id points here)
                // We need reverse lookup: find primary node whose source_node_id = companion.id
                for (const pn of nodes) {
                    if (pn.source_node_id === node.id) {
                        map.set(pn.id, node);
                    }
                }
            } else if (node.source_node_id) {
                // Companion is translation — map by companion's source_node_id (= primary's id)
                map.set(node.source_node_id, node);
            }
        }
        return map;
    }, [companionData, nodes]);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-full text-stone-400">
                <p>Loading...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex items-center justify-center h-full text-red-500">
                <p>Failed to load content.</p>
            </div>
        );
    }

    return (
        <VirtualizedScroll
            ref={ref}
            nodes={nodes}
            initialNodeSlug={initialNodeSlug}
            hasNextPage={hasNextPage ?? false}
            hasPreviousPage={hasPreviousPage ?? false}
            isFetchingNextPage={isFetchingNextPage}
            isFetchingPreviousPage={isFetchingPreviousPage}
            fetchNextPage={fetchNextPage}
            fetchPreviousPage={fetchPreviousPage}
            setStartSortOrder={setStartSortOrder}
            selectedSentenceId={selectedSentenceId}
            showOriginal={showOriginal}
            viewMode={viewMode}
            viewLayout={viewLayout}
            companionNodeMap={companionNodeMap}
            primaryLabel={primaryLabel}
            companionLabel={companionLabel}
            onSelectSentence={onSelectSentence}
            onVisibleNodeChange={onVisibleNodeChange}
            marginSettings={marginSettings}
        />
    );
});

// --- Virtualized scroll inner component ---

interface VirtualizedScrollProps {
    nodes: NodeDetail[];
    initialNodeSlug: string | undefined;
    hasNextPage: boolean;
    hasPreviousPage: boolean;
    isFetchingNextPage: boolean;
    isFetchingPreviousPage: boolean;
    fetchNextPage: () => void;
    fetchPreviousPage: () => void;
    setStartSortOrder: (sortOrder: number | undefined) => void;
    selectedSentenceId: string | undefined;
    showOriginal: boolean;
    viewMode?: string;
    viewLayout?: string;
    companionNodeMap?: Map<string, NodeDetail>;
    primaryLabel?: string;
    companionLabel?: string;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    onVisibleNodeChange?: (nodeSlug: string) => void;
    marginSettings?: MarginSettings;
}

const VirtualizedScroll = forwardRef<
    PanelScrollViewHandle,
    VirtualizedScrollProps
>(function VirtualizedScroll(
    {
        nodes,
        initialNodeSlug,
        hasNextPage,
        hasPreviousPage,
        isFetchingNextPage,
        isFetchingPreviousPage,
        fetchNextPage,
        fetchPreviousPage,
        setStartSortOrder,
        selectedSentenceId,
        showOriginal,
        viewMode,
        viewLayout,
        companionNodeMap,
        primaryLabel,
        companionLabel,
        onSelectSentence,
        onVisibleNodeChange,
        marginSettings,
    },
    ref,
) {
    const parentRef = useRef<HTMLDivElement>(null);
    const [pendingScrollTarget, setPendingScrollTarget] = useState<
        string | null
    >(initialNodeSlug ?? null);

    // Track whether we still need to scroll to the initial sentence from URL params.
    const pendingSentenceScroll = useRef<string | null>(
        selectedSentenceId ?? null,
    );

    const lastEmittedSlugRef = useRef<string | null>(null);

    const [prevInitialSlug, setPrevInitialSlug] = useState(initialNodeSlug);
    if (initialNodeSlug !== prevInitialSlug) {
        setPrevInitialSlug(initialNodeSlug);

        // 2. CHANGE THIS: Differentiate between a scroll and a TOC click
        if (initialNodeSlug === lastEmittedSlugRef.current) {
            // The user smoothly scrolled into this section. Do NOT snap to the top.
            lastEmittedSlugRef.current = null;
        } else {
            // The user clicked a TOC link or hit the Back button. Show overlay and jump.
            setPendingScrollTarget(initialNodeSlug ?? null);
            lastEmittedSlugRef.current = null;
        }
    }

    // Suppress IntersectionObserver during programmatic scrolls to avoid
    // spurious URL updates that re-trigger the route loader.
    const suppressObserverRef = useRef(false);
    const suppressTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

    const suppressObserver = useCallback(() => {
        suppressObserverRef.current = true;
        clearTimeout(suppressTimerRef.current);
        suppressTimerRef.current = setTimeout(() => {
            suppressObserverRef.current = false;
        }, 300);
    }, []);

    const hasActiveMargins =
        marginSettings && marginSettings.enabledSystems.size > 0;

    const isInterleaved = viewMode === "st";
    const isSideBySide =
        viewLayout === "bpl" ||
        viewLayout === "bpr" ||
        viewLayout === "bsl" ||
        viewLayout === "bsr";

    // Wide overscan so prepended items (above the viewport) actually
    // render — that's what triggers `measureElement` → `resizeItem`,
    // which is the library's built-in scroll-anchor mechanism. With
    // `overscan: 3` the prepended items sit outside the render window,
    // never measure, and the library has no way to know they're bigger
    // than the estimate. 25 covers a standard 20-item page-fetch with
    // headroom; total mounted DOM stays well under what the browser
    // handles smoothly. (See virtual-core/index.js `resizeItem` —
    // when a measured item's `start` is above the current scroll
    // offset, the library auto-adjusts `scrollTop` by the size delta.)
    const virtualizer = useVirtualizer({
        count: nodes.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 700,
        overscan: 25,
        // Stable keys so the measurementsCache survives prepends and
        // shifted indices map to the same physical item.
        getItemKey: (index) => nodes[index]?.id ?? index,
    });

    const items = virtualizer.getVirtualItems();

    // Custom scroll-anchor logic was removed. The previous Case A / Case B
    // / deferred-correction code was double-correcting against the
    // library's built-in `resizeItem` adjustments — visible as violent
    // mid-scroll jumps. Tanstack Virtual already handles the prepend
    // case: when `measureElement` reports a new size for an item above
    // the current scroll offset, the library shifts `scrollTop` by the
    // delta. Stable item keys + sufficient overscan are all that's
    // required.

    // Forward infinite scroll trigger
    useEffect(() => {
        if (!items.length) return;
        const lastItem = items[items.length - 1];
        if (
            lastItem.index >= nodes.length - 5 &&
            hasNextPage &&
            !isFetchingNextPage
        ) {
            fetchNextPage();
        }
    }, [items, nodes.length, hasNextPage, isFetchingNextPage, fetchNextPage]);

    // Track when the last TOC jump settled so we can suppress backward fetching
    // during the settling period (prevents prepend from fighting with scroll anchoring).
    const scrollSettledRef = useRef<number>(0);

    // True once the user has scrolled at least once. Prevents the
    // initial-mount auto-prefetch that would fire because firstItem.index
    // is 0 right after `scrollToIndex` lands the user. Programmatic
    // scrolls are filtered via `suppressObserverRef`.
    const hasUserScrolledRef = useRef(false);
    useEffect(() => {
        const parent = parentRef.current;
        if (!parent) return;
        const onScroll = () => {
            if (suppressObserverRef.current) return;
            hasUserScrolledRef.current = true;
        };
        parent.addEventListener("scroll", onScroll, { passive: true });
        return () => parent.removeEventListener("scroll", onScroll);
    }, []);

    const lastBackwardFetchAtRef = useRef<number>(0);

    // Pre-prepend bookkeeping — captured at trigger time, consumed
    // once measurements settle.
    //
    // Why we need this: the library's built-in `resizeItem` adjustment
    // only compensates for the SIZE DELTA between estimate and measured
    // for items above the scroll offset (virtual-core/index.js:578-598).
    // It does NOT compensate for the FULL PREPENDED SIZE itself —
    // there's no API for "items were inserted at the start, please
    // shift my scroll by their total height." We have to do that
    // ourselves.
    //
    // Anchor strategy: record the CURRENT first-loaded node's id (it
    // will be at index 0 pre-prepend). After the prepend lands, find
    // its new index. The virtualizer's `items[newIndex].start` equals
    // the sum of prepended item sizes — exactly the shift we need —
    // BUT only if those items have all been measured. If they're still
    // at estimate sizes the value will be ~`prepended × estimate` and
    // we'll under-shift (the symptom: jumps to mid-Genesis).
    //
    // So we poll `getTotalSize()` after the fetch lands, waiting for
    // it to stabilize across a few frames — that's the signal that
    // ResizeObserver has finished firing for every prepended item.
    // Then we read the anchor's true `items[].start` and apply one
    // exact shift. (Sefaria does the same thing differently: they
    // wait for `.basetext.loading` markers to clear before computing.)
    const pendingPrependRef = useRef<{
        firstNodeId: string;
        oldScrollOffsetWithinFirst: number;
    } | null>(null);

    // Backward infinite-scroll trigger.
    useEffect(() => {
        if (!items.length) return;
        if (pendingScrollTarget) return;
        if (Date.now() - scrollSettledRef.current < 200) return;
        if (!hasUserScrolledRef.current) return;
        if (Date.now() - lastBackwardFetchAtRef.current < 1000) return;
        if (pendingPrependRef.current) return;
        const firstItem = items[0];
        if (
            firstItem.index <= 3 &&
            hasPreviousPage &&
            !isFetchingPreviousPage
        ) {
            const parent = parentRef.current;
            const firstNode = nodes[0];
            if (!parent || !firstNode) return;
            // Distance from start of `firstNode` (idx 0) to the user's
            // current scroll position. Preserved across the prepend so
            // the user's exact viewport content stays put.
            const oldScrollOffsetWithinFirst = parent.scrollTop;
            pendingPrependRef.current = {
                firstNodeId: firstNode.id,
                oldScrollOffsetWithinFirst,
            };
            lastBackwardFetchAtRef.current = Date.now();
            fetchPreviousPage();
        }
    }, [
        items,
        nodes,
        hasPreviousPage,
        isFetchingPreviousPage,
        fetchPreviousPage,
        pendingScrollTarget,
    ]);

    // Apply the deferred shift once measurements have settled.
    useEffect(() => {
        if (!pendingPrependRef.current) return;
        if (isFetchingPreviousPage) return;

        let raf: number | null = null;
        let lastSize = -1;
        let stableFrames = 0;
        let totalFrames = 0;
        const STABILITY_THRESHOLD = 3;
        const MAX_FRAMES = 30; // ~500ms cap; fail-safe

        const tick = () => {
            totalFrames++;
            const size = virtualizer.getTotalSize();
            if (size === lastSize) {
                stableFrames++;
            } else {
                stableFrames = 0;
                lastSize = size;
            }

            if (
                stableFrames >= STABILITY_THRESHOLD ||
                totalFrames >= MAX_FRAMES
            ) {
                const pending = pendingPrependRef.current;
                const parent = parentRef.current;
                if (!pending || !parent) {
                    pendingPrependRef.current = null;
                    return;
                }
                const newIndex = nodes.findIndex(
                    (n) => n.id === pending.firstNodeId,
                );
                if (newIndex < 0) {
                    pendingPrependRef.current = null;
                    return;
                }
                // Heuristic: was the user actively scrolling upward
                // *during* the fetch? If their scrollTop has dropped
                // meaningfully since the trigger fired, they've moved
                // toward the top of the (then-) loaded content
                // expecting more above. Snapping them back to where
                // they triggered the fetch is jarring — they wanted
                // to keep going up, and the new prepended content is
                // now sitting at scrollTop ≈ 0, exactly where they
                // intended to be. Skip the shift in that case.
                //
                // If they stayed roughly where they were, they're
                // mid-read and we DO want to compensate so the prepend
                // doesn't push their content out of view.
                const SCROLL_TOLERANCE = 80;
                const userScrolledUpward =
                    parent.scrollTop <
                    pending.oldScrollOffsetWithinFirst - SCROLL_TOLERANCE;
                if (!userScrolledUpward) {
                    // virtualizer.items[newIndex].start = sum of all
                    // prepended items' sizes. After settle, this is real.
                    const offset = virtualizer.getOffsetForIndex(
                        newIndex,
                        "start",
                    );
                    if (offset) {
                        const newStart = offset[0];
                        // Target scrollTop = position of (former first
                        // node) + user's prior offset within that node.
                        // Keeps the exact same content under the user's
                        // eye when they weren't actively moving.
                        const target =
                            newStart + pending.oldScrollOffsetWithinFirst;
                        const scrollDiff = target - parent.scrollTop;
                        if (scrollDiff !== 0) {
                            suppressObserver();
                            parent.scrollTop = target;
                        }
                    }
                }
                pendingPrependRef.current = null;
                return;
            }

            raf = requestAnimationFrame(tick);
        };

        raf = requestAnimationFrame(tick);
        return () => {
            if (raf !== null) cancelAnimationFrame(raf);
        };
    }, [isFetchingPreviousPage, nodes, virtualizer, suppressObserver]);

    // TOC scroll tracking via IntersectionObserver.
    // Use a stable callback ref so we don't tear down/recreate the observer
    // on every virtual-items change — only when the scroll container mounts.
    const onVisibleNodeChangeRef = useRef(onVisibleNodeChange);
    onVisibleNodeChangeRef.current = onVisibleNodeChange;

    useEffect(() => {
        const container = parentRef.current;
        if (!container) return;

        const observer = new IntersectionObserver(
            (entries) => {
                if (suppressObserverRef.current) return;
                for (const entry of entries) {
                    if (entry.isIntersecting) {
                        const slug = (entry.target as HTMLElement).dataset
                            .nodeSlug;
                        if (slug) {
                            // 3. ADD THIS: Record that we caused this URL update
                            lastEmittedSlugRef.current = slug;
                            onVisibleNodeChangeRef.current?.(slug);
                        }
                    }
                }
            },
            {
                root: container,
                rootMargin: "-10% 0px -80% 0px",
            },
        );

        // Observe existing and future [data-node-slug] elements via MutationObserver
        const observeAll = () => {
            observer.disconnect();
            const els = container.querySelectorAll("[data-node-slug]");
            for (const el of els) observer.observe(el);
        };

        observeAll();

        const mutationObserver = new MutationObserver(() => observeAll());
        mutationObserver.observe(container, { childList: true, subtree: true });

        return () => {
            observer.disconnect();
            mutationObserver.disconnect();
        };
    }, []); // stable — only runs on mount

    // Scroll-to-node via imperative handle
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
                const index = nodes.findIndex((n) => n.slug === nodeSlug);
                if (index >= 0) {
                    suppressObserver();
                    virtualizer.scrollToIndex(index, { align: "start" });
                } else if (sortOrder != null) {
                    // Node not in current data — restart query from new position
                    suppressObserver();
                    setPendingScrollTarget(nodeSlug);
                    setStartSortOrder(sortOrder);
                } else {
                    // No sort_order available, just set pending
                    setPendingScrollTarget(nodeSlug);
                }
            },
        }),
        [nodes, virtualizer, setStartSortOrder, suppressObserver],
    );

    // When nodes update, check if pending target is now loaded
    useEffect(() => {
        if (!pendingScrollTarget) return;

        const index = nodes.findIndex((n) => n.slug === pendingScrollTarget);
        if (index === -1) return; // Wait for data to load

        suppressObserver();
        virtualizer.scrollToIndex(index, { align: "start" });

        // Check if TanStack Virtual has actually placed the item in the DOM
        const targetIsRendered = items.some((v) => v.index === index);

        if (targetIsRendered) {
            // It is measured and rendered!
            // Add a tiny 20ms buffer to ensure the browser has painted the DOM updates
            // before we drop the curtain.
            const timer = setTimeout(() => {
                setPendingScrollTarget(null);
                scrollSettledRef.current = Date.now();
            }, 20);
            return () => clearTimeout(timer);
        }
    }, [nodes, pendingScrollTarget, virtualizer, suppressObserver, items]);

    // After the node-level scroll finishes, scroll to the selected sentence
    // (only on initial load from URL params).
    useEffect(() => {
        if (pendingScrollTarget) return; // node scroll not done yet
        if (!pendingSentenceScroll.current) return; // no sentence to scroll to

        const container = parentRef.current;
        if (!container) return;

        const key = pendingSentenceScroll.current;
        const el = container.querySelector(
            `[data-sentence-key="${CSS.escape(key)}"]`,
        );
        if (el) {
            pendingSentenceScroll.current = null;

            // Find the full sentence object and notify the parent so the
            // resource panel can display sentence details on initial load.
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

            // Small delay to let the virtualizer settle after node scroll
            requestAnimationFrame(() => {
                el.scrollIntoView({ block: "center" });
            });
        }
    }, [pendingScrollTarget, nodes, onSelectSentence]);

    return (
        <Paper
            ref={parentRef}
            square
            elevation={0}
            className="h-full overflow-y-auto relative"
        >
            {pendingScrollTarget && (
                <div className="absolute inset-0 z-10 flex items-center justify-center bg-stone-50">
                    <p className="text-stone-400">Loading...</p>
                </div>
            )}
            <div
                className="relative w-full"
                style={{ height: virtualizer.getTotalSize() }}
            >
                {items.map((virtualRow) => {
                    const node = nodes[virtualRow.index];
                    const companion = companionNodeMap?.get(node.id);
                    const containerClass =
                        isInterleaved && isSideBySide
                            ? "max-w-5xl mx-auto px-8"
                            : hasActiveMargins
                              ? "max-w-4xl mx-auto"
                              : "max-w-2xl mx-auto px-8";
                    return (
                        <div
                            key={node.id}
                            data-index={virtualRow.index}
                            data-node-slug={node.slug}
                            ref={virtualizer.measureElement}
                            className="absolute top-0 left-0 w-full"
                            style={{
                                transform: `translateY(${virtualRow.start}px)`,
                            }}
                        >
                            <div className={containerClass}>
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
                                            selectedSentenceId={
                                                selectedSentenceId ?? null
                                            }
                                            showOriginal={showOriginal}
                                            onSelectSentence={onSelectSentence}
                                            marginSettings={marginSettings}
                                            primaryLabel={
                                                primaryLabel ?? "Source"
                                            }
                                            companionLabel={
                                                companionLabel ?? "Translation"
                                            }
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
                                                onSelectSentence={
                                                    onSelectSentence
                                                }
                                                marginSettings={marginSettings}
                                                nodeSourceRef={node.source_ref}
                                            />
                                        ))
                                    )}
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
            {isFetchingNextPage && !pendingScrollTarget && (
                <div className="flex justify-center py-8 text-stone-400">
                    <p>Loading more...</p>
                </div>
            )}
        </Paper>
    );
});
