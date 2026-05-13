import type {
    InfiniteData,
    QueryFunction,
    UseInfiniteQueryOptions,
} from "@tanstack/react-query";
import { getNodePage, type getNodePageResponse } from "../../api/nodes/nodes";

export type PageCursor = { after: number } | { before: number };

/** Buffer of chapters loaded above the target on initial render — gives the
 *  user somewhere to scroll up to before the reverse fetch fires. */
export const PREFETCH_BUFFER = 5;

interface NodePageQueryArgs {
    bookSlug: string;
    showOriginal: boolean;
    /** The target chapter's sort_order. The query window is centered on
     *  it (a few chapters above as prefetch buffer, plus the target and
     *  some chapters after). When undefined the query won't fire. */
    startSortOrder: number | undefined;
}

/** Shared options for the reader's bidirectional infinite chapter query.
 *  The route loader can prefetch with `queryClient.prefetchInfiniteQuery(opts)`
 *  so chapter content is dehydrated into the prerendered HTML (visible to
 *  crawlers); PanelScrollView passes the same options into useInfiniteQuery so
 *  it warm-loads from cache instead of re-fetching on hydration. */
export function getNodePageQueryOptions({
    bookSlug,
    showOriginal,
    startSortOrder,
}: NodePageQueryArgs): UseInfiniteQueryOptions<
    getNodePageResponse,
    Error,
    InfiniteData<getNodePageResponse, PageCursor | undefined>,
    Array<string>,
    PageCursor | undefined
> {
    const initialPageParam: PageCursor | undefined =
        startSortOrder != null
            ? { after: Math.max(0, startSortOrder - 1 - PREFETCH_BUFFER) }
            : undefined;

    const queryFn: QueryFunction<
        getNodePageResponse,
        Array<string>,
        PageCursor | undefined
    > = async ({ pageParam, signal }) => {
        const base = showOriginal
            ? { limit: 20, original: true }
            : { limit: 20 };
        const params = pageParam
            ? "after" in pageParam
                ? { ...base, after: pageParam.after }
                : { ...base, before: pageParam.before }
            : base;
        return getNodePage(bookSlug, params, { signal });
    };

    return {
        queryKey: [
            "node-page-bidir",
            bookSlug,
            String(startSortOrder),
            showOriginal ? "og" : "",
        ],
        queryFn,
        initialPageParam,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            const page = lastPage.data;
            if (!page.has_more || page.nodes.length === 0) return undefined;
            return { after: page.nodes[page.nodes.length - 1].sort_order };
        },
        getPreviousPageParam: (firstPage) => {
            if (firstPage.status !== 200) return undefined;
            const page = firstPage.data;
            if (!page.has_previous || page.nodes.length === 0) return undefined;
            return { before: page.nodes[0].sort_order };
        },
        enabled: startSortOrder != null,
    };
}
