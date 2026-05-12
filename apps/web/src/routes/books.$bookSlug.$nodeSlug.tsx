import { createFileRoute } from "@tanstack/react-router";
import { getGetTocQueryOptions } from "../api/toc/toc";
import { decode, ReaderLayout, validateSearch } from "../modules/reader";

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch,
    loader: async ({ context, params }) => {
        await context.queryClient.ensureQueryData(
            getGetTocQueryOptions(params.bookSlug),
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
