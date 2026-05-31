import { createFileRoute } from "@tanstack/react-router";
import { getGetBookSuspenseQueryOptions } from "#/api/books/books";
import { getGetTocSuspenseQueryOptions } from "../api/toc/toc";
import {
    decode,
    getNodePageSuspenseQueryOptions,
    ReaderLayout,
    validateSearch,
} from "../modules/reader";

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch,
    loader: ({ context, params }) => {
        context.queryClient.prefetchQuery(
            getGetTocSuspenseQueryOptions(params.bookSlug),
        );
        context.queryClient.prefetchInfiniteQuery(
            getNodePageSuspenseQueryOptions({
                bookSlug: params.bookSlug,
                showOriginal: false,
                targetNodeSlug: params.nodeSlug,
            }),
        );
        context.queryClient.prefetchQuery(
            getGetBookSuspenseQueryOptions(params.bookSlug),
        );
    },
    component: ReaderPage,
});

function ReaderPage() {
    const { bookSlug, nodeSlug } = Route.useParams();
    const search = Route.useSearch();
    const { panels } = decode({ bookSlug, nodeSlug, search });
    return <ReaderLayout panels={panels} />;
}
