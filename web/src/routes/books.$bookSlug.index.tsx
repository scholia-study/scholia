import { createFileRoute, Link } from "@tanstack/react-router";
import { getGetTocQueryOptions, useGetTocSuspense } from "../api/toc/toc";
import { PanelToc } from "../components/PanelToc";

export const Route = createFileRoute("/books/$bookSlug/")({
    loader: async ({ context, params }) => {
        await context.queryClient.ensureQueryData(
            getGetTocQueryOptions(params.bookSlug),
        );
    },
    component: BookPage,
});

function BookPage() {
    const { bookSlug } = Route.useParams();
    const { data: tocData, isLoading, error } = useGetTocSuspense(bookSlug);
    const toc = tocData?.data;

    return (
        <div className="flex h-screen">
            <div className="max-w-3xl mx-auto px-8 py-16 w-full">
                <Link
                    to="/books"
                    className="text-sm text-stone-500 hover:text-stone-700 mb-4 inline-block"
                >
                    &larr; All books
                </Link>
                <h1 className="text-3xl font-bold text-stone-900 mb-8">
                    {bookSlug}
                </h1>
                {isLoading && <p className="text-stone-400">Loading...</p>}
                {error ? (
                    <p className="text-red-500">
                        Failed to load table of contents.
                    </p>
                ) : null}
                {toc ? (
                    <PanelToc
                        toc={toc}
                        bookSlug={bookSlug}
                        activeNodeSlug={undefined}
                    />
                ) : null}
            </div>
        </div>
    );
}
