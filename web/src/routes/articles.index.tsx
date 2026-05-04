import { Chip } from "@mui/material";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useListPublishedArticles } from "../api/articles/articles";
import { useListTopics } from "../api/topics/topics";
import { ArticleCard } from "../modules/article";

export const Route = createFileRoute("/articles/")({
    component: ArticlesListingPage,
});

function ArticlesListingPage() {
    const [topicSlug, setTopicSlug] = useState<string | undefined>(undefined);
    const [page, setPage] = useState(1);

    const { data: articlesData, isLoading } = useListPublishedArticles({
        topic_slug: topicSlug,
        page,
        per_page: 20,
    });
    const articles = articlesData?.data?.articles ?? [];
    const total = articlesData?.data?.total ?? 0;

    const { data: topicsData } = useListTopics();
    const topics = topicsData?.data?.topics ?? [];

    const totalPages = Math.ceil(total / 20);

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-6">Articles</h1>

            {/* Topic filters */}
            {topics.length > 0 && (
                <div className="flex flex-wrap gap-1.5 mb-6">
                    <Chip
                        label="All"
                        size="small"
                        variant={!topicSlug ? "filled" : "outlined"}
                        onClick={() => {
                            setTopicSlug(undefined);
                            setPage(1);
                        }}
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
                            onClick={() => {
                                setTopicSlug(
                                    topicSlug === t.slug ? undefined : t.slug,
                                );
                                setPage(1);
                            }}
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
