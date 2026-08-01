import { Chip } from "@mui/material";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import {
    getListEditorialLabelsSuspenseQueryOptions,
    getListPublishedArticlesSuspenseQueryOptions,
    useListEditorialLabelsSuspense,
    useListPublishedArticlesSuspense,
} from "../api/articles/articles";
import type { EditorialLabelResponse, TopicResponse } from "../api/model";
import {
    getListSeriesSuspenseQueryOptions,
    useListSeriesSuspense,
} from "../api/series/series";
import {
    getListTopicsSuspenseQueryOptions,
    useListTopicsSuspense,
} from "../api/topics/topics";
import { ArticleCard } from "../modules/article";
import { SEO_COPY, seoHead } from "../modules/seo";
import { SeriesChipRow, SeriesSidebar } from "../modules/series";

type ArticlesSearch = {
    page?: number;
    topic_slug?: string;
    label_slug?: string;
};

type UpdateFilter = (
    key: "topic_slug" | "label_slug",
    value: string | undefined,
) => void;

export const Route = createFileRoute("/articles/")({
    component: ArticlesListingPage,
    // Canonical stays at the unfiltered list — page/topic/label filters
    // are views of the same collection, not distinct documents.
    head: () =>
        seoHead({
            title: SEO_COPY.articles.title,
            description: SEO_COPY.articles.description,
            path: "/articles",
        }),
    validateSearch: (search: Record<string, unknown>): ArticlesSearch => {
        const parsedPage = Number(search.page);
        return {
            // Absent = page 1. Keeping the default OUT of the URL means
            // /articles serves 200 directly instead of 307-redirecting
            // to /articles?page=1 (bad for crawlers and link equity).
            page:
                !Number.isNaN(parsedPage) && parsedPage > 1
                    ? parsedPage
                    : undefined,
            topic_slug:
                typeof search.topic_slug === "string"
                    ? search.topic_slug
                    : undefined,
            label_slug:
                typeof search.label_slug === "string"
                    ? search.label_slug
                    : undefined,
        };
    },
    loaderDeps: ({ search: { page, topic_slug, label_slug } }) => ({
        page,
        topic_slug,
        label_slug,
    }),
    loader: ({ context, deps }) => {
        context.queryClient.prefetchQuery(
            getListPublishedArticlesSuspenseQueryOptions({
                page: deps.page,
                per_page: 20,
                topic_slug: deps.topic_slug,
                label_slug: deps.label_slug,
            }),
        );
        context.queryClient.ensureQueryData(
            getListTopicsSuspenseQueryOptions(),
        );
        context.queryClient.ensureQueryData(
            getListEditorialLabelsSuspenseQueryOptions(),
        );
        context.queryClient.ensureQueryData(
            getListSeriesSuspenseQueryOptions(),
        );
    },
});

function ArticlesListingPage() {
    const {
        page,
        topic_slug: topicSlug,
        label_slug: labelSlug,
    } = Route.useSearch();
    const navigate = Route.useNavigate();

    const { data: topicsData } = useListTopicsSuspense();
    const topics = topicsData.data.topics;

    const { data: labelsData } = useListEditorialLabelsSuspense();
    const labels = labelsData.data.labels;

    const { data: seriesData } = useListSeriesSuspense();
    const series = seriesData.data.series;

    // startTransition keeps the current list on screen while the newly
    // filtered page loads, instead of flashing the suspense fallback.
    const updateFilter: UpdateFilter = (key, value) => {
        navigate({
            search: (prev) => ({
                ...prev,
                [key]: value,
                page: 1,
            }),
            startTransition: true,
        });
    };

    return (
        // w-full matters: this sits in a flex column, where mx-auto
        // cancels the default stretch and the box would otherwise
        // shrink-to-fit its content — the grid must hold its width
        // whether the list is full, loading, or empty.
        <div className="w-full max-w-3xl lg:max-w-7xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-6">Articles</h1>

            <div className="lg:grid lg:grid-cols-[10.5rem_minmax(0,1fr)_14rem] lg:gap-10">
                <aside className="hidden lg:block">
                    <div className="sticky top-8">
                        <FilterSidebar
                            topics={topics}
                            labels={labels}
                            topicSlug={topicSlug}
                            labelSlug={labelSlug}
                            updateFilter={updateFilter}
                        />
                    </div>
                </aside>

                <div className="min-w-0">
                    {/* Small screens: filters + series collapse above the list */}
                    <div className="lg:hidden">
                        <FilterChips
                            topics={topics}
                            labels={labels}
                            topicSlug={topicSlug}
                            labelSlug={labelSlug}
                            updateFilter={updateFilter}
                        />
                        <SeriesChipRow series={series} />
                    </div>

                    <Suspense
                        fallback={
                            <p className="text-sm text-stone-400">
                                Loading articles...
                            </p>
                        }
                    >
                        <SuspendedArticleList
                            page={page}
                            topicSlug={topicSlug}
                            labelSlug={labelSlug}
                        />
                    </Suspense>
                </div>

                <aside className="hidden lg:block">
                    <div className="sticky top-8">
                        <SeriesSidebar series={series} />
                    </div>
                </aside>
            </div>
        </div>
    );
}

function FilterSidebar({
    topics,
    labels,
    topicSlug,
    labelSlug,
    updateFilter,
}: {
    topics: TopicResponse[];
    labels: EditorialLabelResponse[];
    topicSlug?: string;
    labelSlug?: string;
    updateFilter: UpdateFilter;
}) {
    const itemClass = (active: boolean) =>
        `block w-full text-left px-2 py-1 rounded text-sm cursor-pointer ${
            active
                ? "text-stone-900 bg-stone-200 font-medium"
                : "text-stone-500 hover:text-stone-900 hover:bg-stone-100"
        }`;

    return (
        <div>
            {labels.length > 0 && (
                <>
                    <h2 className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                        Labels
                    </h2>
                    <ul className="space-y-0.5 mb-6">
                        {labels.map((l) => (
                            <li key={l.id}>
                                <button
                                    type="button"
                                    className={itemClass(labelSlug === l.slug)}
                                    onClick={() =>
                                        updateFilter(
                                            "label_slug",
                                            labelSlug === l.slug
                                                ? undefined
                                                : l.slug,
                                        )
                                    }
                                >
                                    {l.name}
                                </button>
                            </li>
                        ))}
                    </ul>
                </>
            )}

            <h2 className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                Topics
            </h2>
            <ul className="space-y-0.5">
                <li>
                    <button
                        type="button"
                        className={itemClass(!topicSlug)}
                        onClick={() => updateFilter("topic_slug", undefined)}
                    >
                        All
                    </button>
                </li>
                {topics.map((t) => (
                    <li key={t.id}>
                        <button
                            type="button"
                            className={itemClass(topicSlug === t.slug)}
                            onClick={() =>
                                updateFilter(
                                    "topic_slug",
                                    topicSlug === t.slug ? undefined : t.slug,
                                )
                            }
                        >
                            {t.name}
                        </button>
                    </li>
                ))}
            </ul>
        </div>
    );
}

function FilterChips({
    topics,
    labels,
    topicSlug,
    labelSlug,
    updateFilter,
}: {
    topics: TopicResponse[];
    labels: EditorialLabelResponse[];
    topicSlug?: string;
    labelSlug?: string;
    updateFilter: UpdateFilter;
}) {
    return (
        <>
            {/* Editorial label filters */}
            {labels.length > 0 && (
                <div className="flex flex-wrap gap-1.5 mb-3">
                    {labels.map((l) => (
                        <Chip
                            key={l.id}
                            label={l.name}
                            size="small"
                            color={labelSlug === l.slug ? "primary" : "default"}
                            variant={
                                labelSlug === l.slug ? "filled" : "outlined"
                            }
                            onClick={() =>
                                updateFilter(
                                    "label_slug",
                                    labelSlug === l.slug ? undefined : l.slug,
                                )
                            }
                            sx={{ fontSize: "0.75rem" }}
                        />
                    ))}
                </div>
            )}

            {/* Topic filters */}
            {topics.length > 0 && (
                <div className="flex flex-wrap gap-1.5 mb-6">
                    <Chip
                        label="All"
                        size="small"
                        variant={!topicSlug ? "filled" : "outlined"}
                        onClick={() => updateFilter("topic_slug", undefined)}
                        sx={{ fontSize: "0.75rem" }}
                    />
                    {topics.map((t) => (
                        <Chip
                            key={t.id}
                            label={t.name}
                            size="small"
                            color={topicSlug === t.slug ? "primary" : "default"}
                            variant={
                                topicSlug === t.slug ? "filled" : "outlined"
                            }
                            onClick={() =>
                                updateFilter(
                                    "topic_slug",
                                    topicSlug === t.slug ? undefined : t.slug,
                                )
                            }
                            sx={{ fontSize: "0.75rem" }}
                        />
                    ))}
                </div>
            )}
        </>
    );
}

function SuspendedArticleList({
    page,
    topicSlug,
    labelSlug,
}: {
    page?: number;
    topicSlug?: string;
    labelSlug?: string;
}) {
    const navigate = Route.useNavigate();

    const { data: articlesData } = useListPublishedArticlesSuspense({
        page,
        per_page: 20,
        topic_slug: topicSlug,
        label_slug: labelSlug,
    });

    const articles = articlesData?.data?.articles ?? [];
    const total = articlesData?.data?.total ?? 0;
    const totalPages = Math.ceil(total / 20);

    const filtered = Boolean(topicSlug || labelSlug);

    return (
        <>
            {articles.length === 0 &&
                (filtered ? (
                    <p className="text-sm text-stone-400">
                        No articles match this filter.{" "}
                        <button
                            type="button"
                            onClick={() =>
                                navigate({
                                    search: () => ({}),
                                })
                            }
                            className="text-stone-600 underline hover:text-stone-900 cursor-pointer"
                        >
                            Clear filters
                        </button>
                    </p>
                ) : (
                    <p className="text-sm text-stone-400">
                        No published articles yet.
                    </p>
                ))}

            <div className="space-y-4">
                {articles.map((article) => (
                    <ArticleCard key={article.id} article={article} />
                ))}
            </div>

            {/* Pagination Controls */}
            {totalPages > 1 && (
                <div className="flex justify-center gap-2 mt-8">
                    {Array.from({ length: totalPages }, (_, i) => i + 1).map(
                        (p) => (
                            <button
                                key={p}
                                type="button"
                                onClick={() =>
                                    navigate({
                                        search: (prev) => ({
                                            ...prev,
                                            page: p,
                                        }),
                                        startTransition: true,
                                    })
                                }
                                className={`px-3 py-1 text-sm rounded ${
                                    p === (page ?? 1)
                                        ? "bg-stone-800 text-white"
                                        : "text-stone-500 hover:bg-stone-100"
                                }`}
                            >
                                {p}
                            </button>
                        ),
                    )}
                </div>
            )}
        </>
    );
}
