import type { InfiniteData } from "@tanstack/react-query";
import { useInfiniteQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
    forwardRef,
    useCallback,
    useEffect,
    useImperativeHandle,
    useLayoutEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import type { NodeDetail, SentenceResponse } from "../api/model";
import { getNodePage, type getNodePageResponse } from "../api/nodes/nodes";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";

type PageCursor = { after: number } | { before: number };

export interface PanelScrollViewHandle {
    scrollToNode: (nodeSlug: string, sortOrder?: number) => void;
}

interface PanelScrollViewProps {
    bookSlug: string;
    initialNodeSlug: string | undefined;
    initialSortOrder: number | undefined;
    selectedSentenceId: string | undefined;
    onSelectSentence: (sentence: SentenceResponse) => void;
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

    const initialPageParam: PageCursor | undefined =
        startSortOrder != null ? { after: startSortOrder - 1 } : undefined;

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
        queryKey: ["node-page-bidir", bookSlug, String(startSortOrder)],
        queryFn: async ({ pageParam, signal }) => {
            const params = pageParam
                ? "after" in pageParam
                    ? { after: pageParam.after, limit: 20 }
                    : { before: pageParam.before, limit: 20 }
                : { limit: 20 };
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
    onSelectSentence: (sentence: SentenceResponse) => void;
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

    const virtualizer = useVirtualizer({
        count: nodes.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 400,
        overscan: 3,
    });

    const items = virtualizer.getVirtualItems();

    // Maintain scroll position when items are prepended
    const prevFirstNodeIdRef = useRef<string | null>(null);
    useLayoutEffect(() => {
        const prevFirstId = prevFirstNodeIdRef.current;
        if (prevFirstId && nodes.length > 0 && nodes[0].id !== prevFirstId) {
            const newIndex = nodes.findIndex((n) => n.id === prevFirstId);
            if (newIndex > 0) {
                suppressObserver();
                virtualizer.scrollToIndex(newIndex, { align: "start" });
            }
        }
        prevFirstNodeIdRef.current = nodes.length > 0 ? nodes[0].id : null;
    }, [nodes, virtualizer, suppressObserver]);

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

    // Backward infinite scroll trigger
    useEffect(() => {
        if (!items.length) return;
        const firstItem = items[0];
        if (
            firstItem.index <= 3 &&
            hasPreviousPage &&
            !isFetchingPreviousPage
        ) {
            fetchPreviousPage();
        }
    }, [items, hasPreviousPage, isFetchingPreviousPage, fetchPreviousPage]);

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
                        if (slug) onVisibleNodeChangeRef.current?.(slug);
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
        if (index >= 0) {
            setPendingScrollTarget(null);
            suppressObserver();
            virtualizer.scrollToIndex(index, { align: "start" });
        }
    }, [nodes, pendingScrollTarget, virtualizer, suppressObserver]);

    return (
        <div ref={parentRef} className="h-full overflow-y-auto relative">
            {pendingScrollTarget && (
                <div className="absolute inset-0 z-10 flex items-center justify-center bg-stone-50">
                    <p className="text-stone-400">Loading...</p>
                </div>
            )}
            <div
                className="relative w-full"
                style={{ height: virtualizer.getTotalSize() }}
            >
                <div
                    className="absolute top-0 left-0 w-full"
                    style={{
                        transform: `translateY(${items[0]?.start ?? 0}px)`,
                    }}
                >
                    {items.map((virtualRow) => {
                        const node = nodes[virtualRow.index];
                        return (
                            <div
                                key={node.id}
                                data-index={virtualRow.index}
                                data-node-slug={node.slug}
                                ref={virtualizer.measureElement}
                                className={
                                    hasActiveMargins
                                        ? "max-w-4xl mx-auto"
                                        : "max-w-2xl mx-auto px-8"
                                }
                            >
                                <div
                                    className={`py-8 border-b border-stone-100 ${hasActiveMargins ? "max-w-2xl mx-auto px-8" : ""}`}
                                >
                                    {node.blocks.map((block) => (
                                        <Block
                                            key={block.id}
                                            block={block}
                                            selectedSentenceId={
                                                selectedSentenceId ?? null
                                            }
                                            onSelectSentence={onSelectSentence}
                                            marginSettings={marginSettings}
                                        />
                                    ))}
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
            {isFetchingNextPage && !pendingScrollTarget && (
                <div className="flex justify-center py-8 text-stone-400">
                    <p>Loading more...</p>
                </div>
            )}
        </div>
    );
});
