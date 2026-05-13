import { Paper } from "@mui/material";
import {
    keepPreviousData,
    useInfiniteQuery,
    useQuery,
} from "@tanstack/react-query";
import {
    forwardRef,
    useEffect,
    useImperativeHandle,
    useLayoutEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import type { NodeDetail, SentenceResponse } from "../../../api/model";
import { getNodePage } from "../../../api/nodes/nodes";
import { getNodePageQueryOptions } from "../nodePageQuery";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";
import { InterleavedNodeRenderer } from "./InterleavedNodeRenderer";

export interface PanelScrollViewHandle {
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
    const [startSortOrder, setStartSortOrder] = useState<number | undefined>(
        initialSortOrder,
    );

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

    const lastEmittedSlugRef = useRef<string | null>(null);

    const [prevNodeSlug, setPrevNodeSlug] = useState(initialNodeSlug);
    const [pendingScrollTarget, setPendingScrollTarget] = useState<
        string | null
    >(initialNodeSlug ?? null);
    if (initialNodeSlug !== prevNodeSlug) {
        setPrevNodeSlug(initialNodeSlug);
        const isLoaded = nodes.some((n) => n.slug === initialNodeSlug);
        if (!isLoaded && initialSortOrder != null) {
            setStartSortOrder(initialSortOrder);
            setPendingScrollTarget(initialNodeSlug ?? null);
        } else if (isLoaded && initialNodeSlug) {
            if (initialNodeSlug !== lastEmittedSlugRef.current) {
                setPendingScrollTarget(initialNodeSlug);
            }
            lastEmittedSlugRef.current = null;
        }
    }

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

    // Companion data for interleaved view (Kant DE↔EN).
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

    const scrollerRef = useRef<HTMLDivElement | null>(null);

    // Preserve scroll position when previous-page items prepend. We
    // measure the scroller's height before and after the node list grows
    // upward and add the delta to scrollTop — keeping the user's visible
    // content fixed instead of letting the viewport snap back to the new
    // first item.
    const prevFirstNodeIdRef = useRef<string | null>(null);
    const prevScrollHeightRef = useRef<number>(0);
    useLayoutEffect(() => {
        const el = scrollerRef.current;
        if (!el || nodes.length === 0) {
            prevFirstNodeIdRef.current = null;
            prevScrollHeightRef.current = 0;
            return;
        }
        const newFirstId = nodes[0].id;
        if (
            prevFirstNodeIdRef.current &&
            prevFirstNodeIdRef.current !== newFirstId
        ) {
            const prevFirstIdx = nodes.findIndex(
                (n) => n.id === prevFirstNodeIdRef.current,
            );
            if (prevFirstIdx > 0) {
                const delta = el.scrollHeight - prevScrollHeightRef.current;
                el.scrollTop += delta;
            }
        }
        prevFirstNodeIdRef.current = newFirstId;
        prevScrollHeightRef.current = el.scrollHeight;
    }, [nodes]);

    // Initial / requested scroll-to-node. Once the target's <section>
    // exists in the DOM (re-checked whenever the loaded window changes),
    // scroll it to the top of the viewport.
    const targetLoaded =
        pendingScrollTarget != null &&
        nodes.some((n) => n.slug === pendingScrollTarget);
    useEffect(() => {
        if (!pendingScrollTarget || !targetLoaded) return;
        const el = scrollerRef.current?.querySelector(
            `[data-node-slug="${CSS.escape(pendingScrollTarget)}"]`,
        );
        if (!(el instanceof HTMLElement)) return;
        el.scrollIntoView({ block: "start", behavior: "auto" });
        const timer = setTimeout(() => setPendingScrollTarget(null), 50);
        return () => clearTimeout(timer);
    }, [pendingScrollTarget, targetLoaded]);

    // After landing on the target node, scroll to the selected sentence
    // (only on initial load from URL params).
    const pendingSentenceScroll = useRef<string | null>(
        selectedSentenceId ?? null,
    );
    useEffect(() => {
        if (pendingScrollTarget) return;
        if (!pendingSentenceScroll.current) return;
        const key = pendingSentenceScroll.current;
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
        }),
        [nodes],
    );

    // Visible-node tracking for URL sync. Top-slice rootMargin so only the
    // upper portion of the viewport counts.
    const onVisibleNodeChangeRef = useRef(onVisibleNodeChange);
    onVisibleNodeChangeRef.current = onVisibleNodeChange;

    useEffect(() => {
        const scroller = scrollerRef.current;
        if (!scroller) return;

        const intersecting = new Map<HTMLElement, number>();
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
                root: scroller,
                rootMargin: "-10% 0px -80% 0px",
            },
        );

        const observeAll = () => {
            observer.disconnect();
            intersecting.clear();
            const els = scroller.querySelectorAll("[data-node-slug]");
            for (const el of els) observer.observe(el);
        };

        observeAll();

        const mutationObserver = new MutationObserver(() => observeAll());
        mutationObserver.observe(scroller, {
            childList: true,
            subtree: true,
        });

        return () => {
            observer.disconnect();
            mutationObserver.disconnect();
            if (timer) clearTimeout(timer);
        };
    }, []);

    // Edge-fetch sentinels: when the first/last node enters the viewport,
    // page in the adjacent window. Replaces Virtuoso's start/endReached.
    const topSentinelRef = useRef<HTMLDivElement | null>(null);
    const bottomSentinelRef = useRef<HTMLDivElement | null>(null);

    const hasPrev = hasPreviousPage;
    const fetchingPrev = isFetchingPreviousPage;
    const fetchPrev = fetchPreviousPage;
    useEffect(() => {
        const el = topSentinelRef.current;
        const scroller = scrollerRef.current;
        if (!el || !scroller) return;
        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting && hasPrev && !fetchingPrev) {
                        fetchPrev();
                    }
                }
            },
            { root: scroller, rootMargin: "2000px 0px 0px 0px" },
        );
        observer.observe(el);
        return () => observer.disconnect();
    }, [hasPrev, fetchingPrev, fetchPrev]);

    const hasNext = hasNextPage;
    const fetchingNext = isFetchingNextPage;
    const fetchNext = fetchNextPage;
    useEffect(() => {
        const el = bottomSentinelRef.current;
        const scroller = scrollerRef.current;
        if (!el || !scroller) return;
        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting && hasNext && !fetchingNext) {
                        fetchNext();
                    }
                }
            },
            { root: scroller, rootMargin: "0px 0px 2000px 0px" },
        );
        observer.observe(el);
        return () => observer.disconnect();
    }, [hasNext, fetchingNext, fetchNext]);

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

    if (isLoading) {
        return (
            <Paper
                square
                elevation={0}
                className="flex items-center justify-center flex-1 min-h-0 text-stone-400"
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
                className="flex items-center justify-center flex-1 min-h-0 text-red-500"
            >
                <p>Failed to load content.</p>
            </Paper>
        );
    }

    return (
        // flex-1 min-h-0 (not h-full): in a flex column with a toolbar
        // sibling, h-full would size to 100% of the column AND let the
        // toolbar push total content past the column height, spawning a
        // second outer scrollbar. flex-1 takes only the remaining space.
        <Paper square elevation={0} className="flex-1 min-h-0 relative">
            <div
                ref={scrollerRef}
                className="h-full overflow-y-auto"
                style={{ overflowAnchor: "none" }}
            >
                <div ref={topSentinelRef} aria-hidden="true">
                    {isFetchingPreviousPage && (
                        <div className="flex justify-center py-4 text-stone-400">
                            <p>Loading earlier…</p>
                        </div>
                    )}
                </div>
                {nodes.map((node) => {
                    const companion = companionNodeMap?.get(node.id);
                    return (
                        <div
                            key={node.id}
                            data-node-slug={node.slug}
                            className={`flow-root ${containerClass}`}
                        >
                            <div
                                className={`py-8 border-b border-stone-100 ${
                                    hasActiveMargins && !isInterleaved
                                        ? "max-w-2xl mx-auto px-8"
                                        : ""
                                }`}
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
                                        primaryLabel={primaryLabel ?? "Source"}
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
                                            onSelectSentence={onSelectSentence}
                                            marginSettings={marginSettings}
                                            nodeSourceRef={node.source_ref}
                                        />
                                    ))
                                )}
                            </div>
                        </div>
                    );
                })}
                <div ref={bottomSentinelRef} aria-hidden="true">
                    {isFetchingNextPage && (
                        <div className="flex justify-center py-4 text-stone-400">
                            <p>Loading more…</p>
                        </div>
                    )}
                </div>
            </div>
        </Paper>
    );
});
