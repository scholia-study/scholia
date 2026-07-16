import { Link } from "@tanstack/react-router";
import { useListArticleReferencesInfinite } from "../../../api/articles/articles";
import type {
    FootnoteSentenceResponse,
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
export function ArticleReferencesView({
    bookSlug,
    selectedSentence,
}: ArticleReferencesViewProps) {
    const range = getSentenceRange(selectedSentence);

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
                    <div className="text-sm font-medium text-stone-800 group-hover:underline">
                        {article.title}
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
