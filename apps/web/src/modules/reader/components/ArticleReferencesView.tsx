import { Link } from "@tanstack/react-router";
import { useListArticleReferencesInfinite } from "../../../api/articles/articles";
import { useGetBook } from "../../../api/books/books";
import type {
    FootnoteSentenceResponse,
    PassageArticleOrigin,
    SentenceResponse,
} from "../../../api/model";
import { getSentenceRange } from "./CommentaryView";

const PAGE_SIZE = 20;

interface ArticleReferencesViewProps {
    bookSlug: string;
    selectedSentence:
        | SentenceResponse
        | FootnoteSentenceResponse
        | (SentenceResponse | FootnoteSentenceResponse)[]
        | undefined;
}

/**
 * Published articles on the platform that quote the selected passage,
 * across all translations of the same work. One entry per article,
 * newest first.
 */
/** Cross-edition origin chips, mirroring CommentaryEntry's badge model:
 *  prominent language chip when the article quotes another language's
 *  edition, subtle "other ed." when it quotes a same-language sibling.
 *  Nothing renders for articles quoting the current book directly. */
function OriginChips({
    origins,
    bookSlug,
    bookLanguage,
}: {
    origins: PassageArticleOrigin[];
    bookSlug: string;
    bookLanguage?: string;
}) {
    const foreign = origins.filter((o) => o.book_slug !== bookSlug);
    if (foreign.length === 0) return null;
    return (
        <>
            {foreign.map((o) =>
                o.language !== bookLanguage ? (
                    <span
                        key={o.book_slug}
                        className="shrink-0 text-[10px] uppercase tracking-wide font-medium text-indigo-700 bg-indigo-50 rounded px-1 py-0.5"
                        title={`Quotes the ${o.book_slug} edition`}
                    >
                        {o.language}
                    </span>
                ) : (
                    <span
                        key={o.book_slug}
                        className="shrink-0 text-[10px] uppercase tracking-wide text-stone-500 border border-stone-300 rounded px-1 py-0.5"
                        title={`Quotes the ${o.book_slug} edition`}
                    >
                        other ed.
                    </span>
                ),
            )}
        </>
    );
}

export function ArticleReferencesView({
    bookSlug,
    selectedSentence,
}: ArticleReferencesViewProps) {
    const range = getSentenceRange(selectedSentence);

    // Warm cache hit — TextPanel fetches the same book detail.
    const { data: bookData } = useGetBook(bookSlug);
    const bookLanguage = bookData?.data?.language;

    const { data, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } =
        useListArticleReferencesInfinite(
            bookSlug,
            {
                start: range?.start ?? 0,
                end: range?.end ?? 0,
                kind: range?.kind ?? "body",
                limit: PAGE_SIZE,
            },
            {
                query: {
                    enabled: !!range,
                    initialPageParam: 0,
                    getNextPageParam: (lastPage, allPages) => {
                        const fetched = allPages.reduce(
                            (n, page) => n + (page.data?.articles.length ?? 0),
                            0,
                        );
                        const total = lastPage.data?.total ?? 0;
                        return fetched < total ? fetched : undefined;
                    },
                },
            },
        );

    if (!range) {
        return (
            <div className="flex-1 overflow-y-auto p-4">
                <p className="text-sm text-stone-400">
                    Select a sentence to view articles quoting it.
                </p>
            </div>
        );
    }

    const articles =
        data?.pages.flatMap((page) => page.data?.articles ?? []) ?? [];
    const total = data?.pages.at(-1)?.data?.total ?? 0;

    return (
        <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
            {isLoading && (
                <p className="text-sm text-stone-400 p-2">Loading...</p>
            )}

            {!isLoading && articles.length === 0 && (
                <p className="text-sm text-stone-400 p-2">
                    No articles quote this selection.
                </p>
            )}

            {articles.map((article) => (
                <Link
                    key={article.id}
                    to="/articles/$slug"
                    params={{ slug: article.slug }}
                    target="_blank"
                    rel="noopener"
                    className="block p-2 border border-stone-100 rounded bg-white group hover:border-stone-300"
                >
                    <div className="flex items-center gap-1.5">
                        <span className="text-sm font-medium text-stone-800 group-hover:underline truncate">
                            {article.title}
                        </span>
                        <OriginChips
                            origins={article.origins}
                            bookSlug={bookSlug}
                            bookLanguage={bookLanguage}
                        />
                    </div>
                    <div className="flex items-center gap-2 text-xs text-stone-400 mt-0.5">
                        <span>{article.author_display_name}</span>
                        {article.published_at && (
                            <>
                                <span>&middot;</span>
                                <span>
                                    {new Date(
                                        article.published_at,
                                    ).toLocaleDateString(undefined, {
                                        month: "long",
                                        day: "numeric",
                                        year: "numeric",
                                    })}
                                </span>
                            </>
                        )}
                    </div>
                </Link>
            ))}

            {hasNextPage && (
                <button
                    type="button"
                    onClick={() => fetchNextPage()}
                    disabled={isFetchingNextPage}
                    className="w-full text-xs text-stone-500 hover:text-stone-700 py-1.5 disabled:opacity-50"
                >
                    {isFetchingNextPage
                        ? "Loading..."
                        : `Show more (${total - articles.length} remaining)`}
                </button>
            )}
        </div>
    );
}
