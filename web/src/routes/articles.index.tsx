import { Chip } from "@mui/material";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
    useListEditorialLabels,
    useListPublishedArticles,
} from "../api/articles/articles";
import { useListTopics } from "../api/topics/topics";
import { ArticleCard } from "../modules/article";

interface ArticlesSearch {
    topic_slug?: string;
    label_slug?: string;
}

export const Route = createFileRoute("/articles/")({
    component: ArticlesListingPage,
    validateSearch: (search: Record<string, unknown>): ArticlesSearch => ({
        topic_slug:
            typeof search.topic_slug === "string"
                ? search.topic_slug
                : undefined,
        label_slug:
            typeof search.label_slug === "string"
                ? search.label_slug
                : undefined,
    }),
});

function ArticlesListingPage() {
    const search = Route.useSearch();
    const navigate = Route.useNavigate();
    const topicSlug = search.topic_slug;
    const labelSlug = search.label_slug;
    const [page, setPage] = useState(1);

    const setTopicSlug = (next: string | undefined) => {
        navigate({ search: (s) => ({ ...s, topic_slug: next }) });
        setPage(1);
    };
    const setLabelSlug = (next: string | undefined) => {
        navigate({ search: (s) => ({ ...s, label_slug: next }) });
        setPage(1);
    };

    const { data: articlesData, isLoading } = useListPublishedArticles({
        topic_slug: topicSlug,
        label_slug: labelSlug,
        page,
        per_page: 20,
    });
    const articles = articlesData?.data?.articles ?? [];
    const total = articlesData?.data?.total ?? 0;

    const { data: topicsData } = useListTopics();
    const topics = topicsData?.data?.topics ?? [];

    const { data: labelsData } = useListEditorialLabels();
    const labels = labelsData?.data?.labels ?? [];

    const totalPages = Math.ceil(total / 20);

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
                                setLabelSlug(
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
                        onClick={() => setTopicSlug(undefined)}
                        sx={{ fontSize: "0.75rem" }}
                    />
                    {topics.map((t) => (
                        <Chip
                            key={t.id}
                            label={t.name}
                            size="small"
                            variant={
                                topicSlug === t.slug ? "filled" : "outlined"
                            }
                            onClick={() =>
                                setTopicSlug(
                                    topicSlug === t.slug ? undefined : t.slug,
                                )
                            }
                            sx={{ fontSize: "0.75rem" }}
                        />
                    ))}
                </div>
            )}

            {isLoading && <p className="text-sm text-stone-400">Loading...</p>}

            {!isLoading && articles.length === 0 && (
                <p className="text-sm text-stone-400">
                    No published articles yet.
                </p>
            )}

            <div className="space-y-4">
                {articles.map((article) => (
                    <ArticleCard key={article.id} article={article} />
                ))}
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
                <div className="flex justify-center gap-2 mt-8">
                    {Array.from({ length: totalPages }, (_, i) => i + 1).map(
                        (p) => (
                            <button
                                key={p}
                                type="button"
                                onClick={() => setPage(p)}
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
        </div>
    );
}
