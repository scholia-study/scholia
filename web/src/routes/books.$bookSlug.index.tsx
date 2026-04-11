import { createFileRoute, Link } from "@tanstack/react-router";
import {
    getGetBookQueryOptions,
    useGetBookSuspense,
} from "../api/books/books";
import { getGetTocQueryOptions, useGetTocSuspense } from "../api/toc/toc";
import { PanelToc } from "../components/PanelToc";

export const Route = createFileRoute("/books/$bookSlug/")({
    loader: async ({ context, params }) => {
        await Promise.all([
            context.queryClient.ensureQueryData(
                getGetBookQueryOptions(params.bookSlug),
            ),
            context.queryClient.ensureQueryData(
                getGetTocQueryOptions(params.bookSlug),
            ),
        ]);
    },
    component: BookPage,
});

function BookPage() {
    const { bookSlug } = Route.useParams();
    const { data: bookData } = useGetBookSuspense(bookSlug);
    const { data: tocData, isLoading, error } = useGetTocSuspense(bookSlug);
    const book = bookData?.data;
    const toc = tocData?.data;

    return (
        <div className="flex h-[calc(100vh-3rem)]">
            <div className="max-w-3xl mx-auto px-8 py-16 w-full">
                <Link
                    to="/"
                    className="text-sm text-stone-500 hover:text-stone-700 mb-4 inline-block"
                >
                    &larr; Library
                </Link>
                <h1 className="text-3xl font-bold text-stone-900 mb-8">
                    {book?.title ?? bookSlug}
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
