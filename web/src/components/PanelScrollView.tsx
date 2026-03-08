import { useVirtualizer } from "@tanstack/react-virtual";
import {
    forwardRef,
    useEffect,
    useImperativeHandle,
    useMemo,
    useRef,
    useState,
} from "react";
import type { NodeDetail, SentenceResponse } from "../api/model";
import { useGetNodePageInfinite } from "../api/nodes/nodes";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";

export interface PanelScrollViewHandle {
    scrollToNode: (nodeSlug: string) => void;
}

interface PanelScrollViewProps {
    bookSlug: string;
    initialNodeSlug: string | undefined;
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
        selectedSentenceId,
        onSelectSentence,
        onVisibleNodeChange,
        onSystemsDiscovered,
        marginSettings,
    },
    ref,
) {
    const {
        data,
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
        isLoading,
        error,
    } = useGetNodePageInfinite(
        bookSlug,
        { limit: 20 },
        {
            query: {
                initialPageParam: undefined,
                getNextPageParam: (lastPage) => {
                    if (lastPage.status !== 200) return undefined;
                    const page = lastPage.data;
                    if (!page.has_more || page.nodes.length === 0)
                        return undefined;
                    return page.nodes[page.nodes.length - 1].sort_order;
                },
            },
        },
    );

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
            isFetchingNextPage={isFetchingNextPage}
            fetchNextPage={fetchNextPage}
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
    isFetchingNextPage: boolean;
    fetchNextPage: () => void;
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
        isFetchingNextPage,
        fetchNextPage,
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

    const hasActiveMargins =
        marginSettings && marginSettings.enabledSystems.size > 0;

    const virtualizer = useVirtualizer({
        count: nodes.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 400,
        overscan: 3,
    });

    const items = virtualizer.getVirtualItems();

    // Infinite scroll trigger
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

    // TOC scroll tracking via IntersectionObserver
    useEffect(() => {
        if (!onVisibleNodeChange || !parentRef.current) return;

        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting) {
                        const nodeSlug = (entry.target as HTMLElement).dataset
                            .nodeSlug;
                        if (nodeSlug) onVisibleNodeChange(nodeSlug);
                    }
                }
            },
            {
                root: parentRef.current,
                rootMargin: "-10% 0px -80% 0px",
            },
        );

        const container = parentRef.current;
        const nodeElements = container.querySelectorAll("[data-node-slug]");
        nodeElements.forEach((el) => observer.observe(el));

        return () => observer.disconnect();
    }, [items, onVisibleNodeChange]);

    // Scroll-to-node via imperative handle
    useImperativeHandle(
        ref,
        () => ({
            scrollToNode(nodeSlug: string) {
                const index = nodes.findIndex((n) => n.slug === nodeSlug);
                if (index >= 0) {
                    virtualizer.scrollToIndex(index, { align: "start" });
                } else {
                    setPendingScrollTarget(nodeSlug);
                }
            },
        }),
        [nodes, virtualizer],
    );

    // When nodes update, check if pending target is now loaded
    useEffect(() => {
        if (!pendingScrollTarget) return;
        const index = nodes.findIndex((n) => n.slug === pendingScrollTarget);
        if (index >= 0) {
            setPendingScrollTarget(null);
            virtualizer.scrollToIndex(index, { align: "start" });
        }
    }, [nodes, pendingScrollTarget, virtualizer]);

    // Progressively fetch until pending target is loaded
    useEffect(() => {
        if (!pendingScrollTarget) return;
        if (!isFetchingNextPage && hasNextPage) {
            fetchNextPage();
        }
    }, [pendingScrollTarget, isFetchingNextPage, hasNextPage, fetchNextPage]);

    return (
        <div ref={parentRef} className="h-full overflow-y-auto">
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
            {isFetchingNextPage && (
                <div className="flex justify-center py-8 text-stone-400">
                    <p>Loading more...</p>
                </div>
            )}
        </div>
    );
});
