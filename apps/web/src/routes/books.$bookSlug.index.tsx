import { createFileRoute, Link, useLocation } from "@tanstack/react-router";
import {
    getGetBookSuspenseQueryOptions,
    useGetBookSuspense,
} from "../api/books/books";
import {
    getGetTocSuspenseQueryOptions,
    useGetTocSuspense,
} from "../api/toc/toc";
import { BibleShapeFullToc, PanelToc } from "../modules/reader";

export const Route = createFileRoute("/books/$bookSlug/")({
    loader: ({ context, params }) => {
        context.queryClient.prefetchQuery(
            getGetBookSuspenseQueryOptions(params.bookSlug),
        );
        context.queryClient.prefetchQuery(
            getGetTocSuspenseQueryOptions(params.bookSlug),
        );
    },
    component: BookPage,
});

function BookPage() {
    const { bookSlug } = Route.useParams();
    const { data: bookData } = useGetBookSuspense(bookSlug);
    const { data: tocData, isLoading, error } = useGetTocSuspense(bookSlug);
    const book = bookData.data;
    const toc = tocData.data;
    // URL fragment shortcut, e.g. /books/kjv-bible#john — used by the
    // library book pills to jump straight to a Bible-book section on
    // this TOC page.
    const { hash } = useLocation();
    const initialAnchor = hash ? hash.replace(/^#/, "") : undefined;

    // Same Bible-shape detection as the sidebar PanelToc — top-level
    // nodes are bibliographic anchors (Genesis, John).
    const isBibleShape =
        !!toc && toc.length > 0 && toc.every((n) => n.source_id);

    return (
        <div className="flex h-full bg-stone-50">
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
                {toc && isBibleShape ? (
                    <BibleShapeFullToc
                        toc={toc}
                        bookSlug={bookSlug}
                        initialAnchor={initialAnchor}
                    />
                ) : toc ? (
                    <PanelToc
                        toc={toc}
                        bookSlug={bookSlug}
                        activeNodeSlug={undefined}
                        scrollable={false}
                    />
                ) : null}
            </div>
        </div>
    );
}
