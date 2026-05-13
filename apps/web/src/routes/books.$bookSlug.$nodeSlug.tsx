import { createFileRoute } from "@tanstack/react-router";
import { getGetTocQueryOptions } from "../api/toc/toc";
import {
    decode,
    getNodePageQueryOptions,
    ReaderLayout,
    validateSearch,
} from "../modules/reader";

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch,
    loader: async ({ context, params }) => {
        // TOC + node-page run in parallel: the API resolves the target
        // node's slug → sort_order server-side, so the node-page prefetch
        // no longer depends on the TOC response.
        await Promise.all([
            context.queryClient.ensureQueryData(
                getGetTocQueryOptions(params.bookSlug),
            ),
            context.queryClient.prefetchInfiniteQuery(
                getNodePageQueryOptions({
                    bookSlug: params.bookSlug,
                    showOriginal: false,
                    targetNodeSlug: params.nodeSlug,
                }),
            ),
        ]);
    },
    component: ReaderPage,
});

function ReaderPage() {
    const { bookSlug, nodeSlug } = Route.useParams();
    const search = Route.useSearch();
    const { panels } = decode({ bookSlug, nodeSlug, search });
    return <ReaderLayout panels={panels} />;
}
