import { createFileRoute } from "@tanstack/react-router";
import type { TocNodeResponse } from "../api/model";
import { getGetTocQueryOptions } from "../api/toc/toc";
import {
    decode,
    getNodePageQueryOptions,
    ReaderLayout,
    validateSearch,
} from "../modules/reader";

function findSortOrderInToc(
    nodes: TocNodeResponse[],
    slug: string,
): number | undefined {
    for (const n of nodes) {
        if (n.slug === slug) return n.sort_order;
        const found = findSortOrderInToc(n.children, slug);
        if (found != null) return found;
    }
    return undefined;
}

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch,
    loader: async ({ context, params }) => {
        // TOC is needed for sidebar nav AND to resolve the target chapter's
        // sort_order (so we can prefetch the right chapter window).
        const tocResponse = await context.queryClient.ensureQueryData(
            getGetTocQueryOptions(params.bookSlug),
        );
        const toc = tocResponse?.data ?? [];
        const startSortOrder = findSortOrderInToc(toc, params.nodeSlug);
        // Prefetch the chapter window so the actual verses end up in the
        // server-rendered HTML (vs "Loading…" on first paint).
        if (startSortOrder != null) {
            await context.queryClient.prefetchInfiniteQuery(
                getNodePageQueryOptions({
                    bookSlug: params.bookSlug,
                    showOriginal: false,
                    startSortOrder,
                }),
            );
        }
    },
    component: ReaderPage,
});

function ReaderPage() {
    const { bookSlug, nodeSlug } = Route.useParams();
    const search = Route.useSearch();
    const { panels } = decode({ bookSlug, nodeSlug, search });
    return <ReaderLayout panels={panels} />;
}
