import { Chip } from "@mui/material";
import { createFileRoute, Link } from "@tanstack/react-router";
import type { Element } from "html-react-parser";
import { useGetPublishedArticle } from "../api/articles/articles";
import { ArticleQuotationCard } from "../components/ArticleQuotationCard";
import { ArticleSentences } from "../components/ArticleSentences";
import { QuotationCard } from "../components/QuotationCard";

export const Route = createFileRoute("/articles/$slug")({
    component: PublishedArticlePage,
});

function replaceEmbed(domNode: Element) {
    // Book quotation embed
    if (domNode.attribs?.class?.includes("quotation-embed")) {
        const attrs = domNode.attribs;
        return (
            <QuotationCard
                book={attrs["data-quotation-book"] ?? ""}
                node={attrs["data-quotation-node"] ?? ""}
                start={Number(attrs["data-quotation-start"]) || 0}
                end={
                    attrs["data-quotation-end"]
                        ? Number(attrs["data-quotation-end"])
                        : undefined
                }
                kind={attrs["data-quotation-kind"] ?? "body"}
                mode={
                    (attrs["data-quotation-mode"] as
                        | "source"
                        | "translation"
                        | "source+translation") ?? "translation"
                }
                layout={
                    (attrs["data-quotation-layout"] as
                        | "stacked"
                        | "side-by-side-source-left"
                        | "side-by-side-source-right") ?? "stacked"
                }
            />
        );
    }

    // Article quotation embed
    if (domNode.attribs?.class?.includes("article-quotation-embed")) {
        const id = domNode.attribs["data-article-quotation-id"];
        if (id) {
            return <ArticleQuotationCard id={id} />;
        }
    }

    return undefined;
}

function PublishedArticlePage() {
    const { slug } = Route.useParams();
    const { data: articleData, isLoading } = useGetPublishedArticle(slug);
    const article = articleData?.data;

    if (isLoading) {
        return (
            <div className="min-h-screen bg-white">
                <div className="max-w-3xl mx-auto px-8 py-16">
                    <p className="text-sm text-stone-400">Loading...</p>
                </div>
            </div>
        );
    }

    if (!article) {
        return (
            <div className="min-h-screen bg-white">
                <div className="max-w-3xl mx-auto px-8 py-16">
                    <p className="text-sm text-stone-400">Article not found.</p>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-white">
            <div className="max-w-3xl mx-auto px-8 py-16">
                {/* Header */}
                <header className="mb-8">
                    <h1 className="text-3xl font-bold text-stone-900 mb-3">
                        {article.title}
                    </h1>
                    {article.description && (
                        <p className="text-lg text-stone-500 mb-4">
                            {article.description}
                        </p>
                    )}
                    <div className="flex items-center gap-2 text-sm text-stone-400">
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
                        {article.updated_at !== article.published_at && (
                            <>
                                <span>&middot;</span>
                                <span>
                                    Updated{" "}
                                    {new Date(
                                        article.updated_at,
                                    ).toLocaleDateString(undefined, {
                                        month: "long",
                                        day: "numeric",
                                        year: "numeric",
                                    })}
                                </span>
                            </>
                        )}
                    </div>
                    {article.topics.length > 0 && (
                        <div className="flex gap-1.5 mt-3">
                            {article.topics.map((t) => (
                                <Link
                                    key={t.id}
                                    to="/articles"
                                    search={{ topic_slug: t.slug }}
                                >
                                    <Chip
                                        label={t.name}
                                        size="small"
                                        variant="outlined"
                                        sx={{ fontSize: "0.75rem" }}
                                    />
                                </Link>
                            ))}
                        </div>
                    )}
                    {article.status === "archived" && (
                        <div className="mt-4 px-3 py-2 bg-stone-100 rounded text-sm text-stone-500">
                            This article has been archived by the author.
                        </div>
                    )}
                </header>

                {/* Article body with clickable sentences */}
                <div className="prose prose-stone max-w-none">
                    <ArticleSentences
                        html={article.html}
                        articleId={article.id}
                        replaceEmbed={replaceEmbed}
                    />
                </div>
            </div>
        </div>
    );
}
