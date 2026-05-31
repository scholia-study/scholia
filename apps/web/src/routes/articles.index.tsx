import { Chip } from "@mui/material";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import {
    getListEditorialLabelsSuspenseQueryOptions,
    getListPublishedArticlesSuspenseQueryOptions,
    useListEditorialLabelsSuspense,
    useListPublishedArticlesSuspense,
} from "../api/articles/articles";
import {
    getListTopicsSuspenseQueryOptions,
    useListTopicsSuspense,
} from "../api/topics/topics";
import { ArticleCard } from "../modules/article";

type ArticlesSearch = {
    page?: number;
    topic_slug?: string;
    label_slug?: string;
};

export const Route = createFileRoute("/articles/")({
    component: ArticlesListingPage,
    validateSearch: (search: Record<string, unknown>): ArticlesSearch => {
        const parsedPage = Number(search.page);
        return {
            page: !Number.isNaN(parsedPage) && parsedPage > 0 ? parsedPage : 1,
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

    const updateFilter = (
        key: "topic_slug" | "label_slug",
        value: string | undefined,
    ) => {
        navigate({
            search: (prev) => ({
                ...prev,
                [key]: value,
                page: 1,
            }),
        });
    };

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-6">Articles</h1>

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

    return (
        <>
            {articles.length === 0 && (
                <p className="text-sm text-stone-400">
                    No published articles yet.
                </p>
            )}

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
                                        }), // FIXED: you had prev.page + 1 here hardcoded!
                                        startTransition: true,
                                    })
                                }
                                className={`px-3 py-1 text-sm rounded ${
                                    p === page
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
