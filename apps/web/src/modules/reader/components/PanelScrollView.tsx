import { Paper } from "@mui/material";
import {
    keepPreviousData,
    useQuery,
    useSuspenseInfiniteQuery,
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
import { getNodePageSuspenseQueryOptions } from "../nodePageQuery";
import type { MarginSettings } from "./BlockRenderer";
import { Block } from "./BlockRenderer";
import { InterleavedNodeRenderer } from "./InterleavedNodeRenderer";

export interface PanelScrollViewHandle {
    getSentencesInRange: (start: number, end: number) => SentenceResponse[];
}

interface PanelScrollViewProps {
    bookSlug: string;
    initialNodeSlug: string | undefined;
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
    const [targetNodeSlug, setTargetNodeSlug] = useState<string | undefined>(
        initialNodeSlug,
    );

    useEffect(() => {
        if (initialNodeSlug != null && targetNodeSlug == null) {
            setTargetNodeSlug(initialNodeSlug);
        }
    }, [initialNodeSlug, targetNodeSlug]);

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
    } = useSuspenseInfiniteQuery(
        getNodePageSuspenseQueryOptions({
            bookSlug,
            showOriginal,
            targetNodeSlug,
        }),
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
    // Mask the reading column until the initial deep-link/refresh scroll lands,
    // so we never flash the top of the prefetch buffer before jumping to target.
    // One-time: only the first load masks; in-reader navigation never does.
    const [initialScrollPending, setInitialScrollPending] = useState(
        initialNodeSlug != null,
    );
    // Sentence to focus on initial load from URL params (seeded once at mount).
    const pendingSentenceScroll = useRef<string | null>(
        selectedSentenceId ?? null,
    );
    if (initialNodeSlug !== prevNodeSlug) {
        setPrevNodeSlug(initialNodeSlug);
        const isLoaded = nodes.some((n) => n.slug === initialNodeSlug);
        if (!isLoaded && initialNodeSlug != null) {
            setTargetNodeSlug(initialNodeSlug);
            setPendingScrollTarget(initialNodeSlug);
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
    // scroll it to the top of the viewport — in a layout effect, before the
    // browser paints, so the target is already in place on the first frame
    // (no top-of-buffer flash, then jump). Reveals the masked column too.
    const targetLoaded =
        pendingScrollTarget != null &&
        nodes.some((n) => n.slug === pendingScrollTarget);
    useLayoutEffect(() => {
        if (!pendingScrollTarget || !targetLoaded) return;
        const el = scrollerRef.current?.querySelector(
            `[data-node-slug="${CSS.escape(pendingScrollTarget)}"]`,
        );
        if (!(el instanceof HTMLElement)) return;
        el.scrollIntoView({ block: "start", behavior: "auto" });
        // If a sentence also needs focusing, stay masked until that lands
        // (handled below) so we don't reveal at the node top then jump.
        if (!pendingSentenceScroll.current) setInitialScrollPending(false);
        const timer = setTimeout(() => setPendingScrollTarget(null), 50);
        return () => clearTimeout(timer);
    }, [pendingScrollTarget, targetLoaded]);

    // Safety: never leave the column masked if the target can't be reached
    // (e.g. an unknown slug). The layout effect above still scrolls before
    // paint once data arrives, so revealing early causes no jump.
    useEffect(() => {
        if (!initialScrollPending) return;
        const t = setTimeout(() => setInitialScrollPending(false), 1500);
        return () => clearTimeout(t);
    }, [initialScrollPending]);

    // After landing on the target node, scroll to the selected sentence
    // (only on initial load from URL params). A layout effect, before paint,
    // and it reveals the masked column only once positioned — so the focus
    // scroll is never visible as a jump.
    useLayoutEffect(() => {
        if (pendingScrollTarget) return;
        if (!pendingSentenceScroll.current) return;
        const key = pendingSentenceScroll.current;
        const el = document.querySelector(
            `[data-sentence-key="${CSS.escape(key)}"]`,
        );
        if (!el) return;
        pendingSentenceScroll.current = null;
        for (const node of nodes) {
            for (const block of node.blocks) {
                for (const sentence of block.sentences) {
                    if (
                        sentence.id === key ||
                        (sentence.sentence_number != null &&
                            String(sentence.sentence_number) === key) ||
                        (sentence.figure_number != null &&
                            `fig${sentence.figure_number}` === key)
                    ) {
                        onSelectSentence(sentence, false);
                        break;
                    }
                }
            }
        }
        // Land the sentence ~20% below the top of the reading viewport (not
        // dead-center). Scoped to our own scroller so it never scrolls
        // <main>/window.
        const scroller = scrollerRef.current;
        if (scroller) {
            const delta =
                el.getBoundingClientRect().top -
                scroller.getBoundingClientRect().top -
                scroller.clientHeight * 0.2;
            scroller.scrollTop += delta;
        } else {
            el.scrollIntoView({ block: "start" });
        }
        setInitialScrollPending(false);
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
              : "max-w-[var(--reader-width)] mx-auto px-8";

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
                style={{
                    overflowAnchor: "none",
                    fontSize: "var(--reader-font-size, 1rem)",
                    opacity: initialScrollPending ? 0 : 1,
                }}
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
                    // A play node — detected by the presence of speaker labels —
                    // indents its dialogue under the flush-left speaker column.
                    const nodeIsDrama = node.blocks.some(
                        (b) => b.block_type === "speaker",
                    );
                    return (
                        <div
                            key={node.id}
                            data-node-slug={node.slug}
                            className={`flow-root ${containerClass}`}
                        >
                            <div
                                className={`py-8 border-b border-stone-100 ${
                                    hasActiveMargins && !isInterleaved
                                        ? "max-w-[var(--reader-width)] mx-auto px-8"
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
                                            inDrama={nodeIsDrama}
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
