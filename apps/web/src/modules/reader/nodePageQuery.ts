import type {
    InfiniteData,
    QueryFunction,
    UseInfiniteQueryOptions,
} from "@tanstack/react-query";
import { getNodePage, type getNodePageResponse } from "../../api/nodes/nodes";

export type PageCursor =
    | { after: number }
    | { before: number }
    | { at: string; back: number };

/** Buffer of chapters loaded above the target on initial render — gives the
 *  user somewhere to scroll up to before the reverse fetch fires. */
export const PREFETCH_BUFFER = 5;

interface NodePageQueryArgs {
    bookSlug: string;
    showOriginal: boolean;
    /** The target chapter's slug. The API resolves it to a sort_order and
     *  returns a window centered on it (PREFETCH_BUFFER chapters above plus
     *  the target and the rest of the page size after). When undefined the
     *  query won't fire. */
    targetNodeSlug: string | undefined;
}

/** Shared options for the reader's bidirectional infinite chapter query.
 *  The route loader can prefetch with `queryClient.prefetchInfiniteQuery(opts)`
 *  so chapter content is dehydrated into the prerendered HTML (visible to
 *  crawlers); PanelScrollView passes the same options into useInfiniteQuery so
 *  it warm-loads from cache instead of re-fetching on hydration.
 *
 *  Keyed by `targetNodeSlug` (not sort_order) so the loader can prefetch
 *  without first fetching the TOC to resolve sort_order — the API resolves
 *  the slug server-side. */
export function getNodePageQueryOptions({
    bookSlug,
    showOriginal,
    targetNodeSlug,
}: NodePageQueryArgs): UseInfiniteQueryOptions<
    getNodePageResponse,
    Error,
    InfiniteData<getNodePageResponse, PageCursor | undefined>,
    Array<string>,
    PageCursor | undefined
> {
    const initialPageParam: PageCursor | undefined =
        targetNodeSlug != null
            ? { at: targetNodeSlug, back: PREFETCH_BUFFER }
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
            ? "at" in pageParam
                ? { ...base, at: pageParam.at, back: pageParam.back }
                : "after" in pageParam
                  ? { ...base, after: pageParam.after }
                  : { ...base, before: pageParam.before }
            : base;
        return getNodePage(bookSlug, params, { signal });
    };

    return {
        queryKey: [
            "node-page-bidir",
            bookSlug,
            targetNodeSlug ?? "",
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
        enabled: targetNodeSlug != null,
    };
}
