import { Chip } from "@mui/material";
import { createFileRoute, Link } from "@tanstack/react-router";
import parse, { type DOMNode, Element } from "html-react-parser";
import { useGetPublishedArticle } from "../api/articles/articles";
import { QuotationCard } from "../components/QuotationCard";

export const Route = createFileRoute("/articles/$slug")({
    component: PublishedArticlePage,
});

function PublishedArticlePage() {
    const { slug } = Route.useParams();
    const { data: articleData, isLoading } = useGetPublishedArticle(slug);
    const article = articleData?.data;

    if (isLoading) {
        return (
            <div className="max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Loading...</p>
            </div>
        );
    }

    if (!article) {
        return (
            <div className="max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Article not found.</p>
            </div>
        );
    }

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            {/* Header */}
            <header className="mb-8">
                <h1
                    className="text-3xl font-bold text-stone-900 mb-3"
                    style={{ fontFamily: "'Libre Baskerville', serif" }}
                >
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
            </header>

            {/* Article body */}
            <div
                className="prose prose-stone max-w-none"
                style={{ fontFamily: "'Libre Baskerville', serif" }}
            >
                {parse(article.html, {
                    replace: (domNode: DOMNode) => {
                        if (
                            domNode instanceof Element &&
                            domNode.attribs?.class?.includes("quotation-embed")
                        ) {
                            const attrs = domNode.attribs;
                            return (
                                <QuotationCard
                                    book={attrs["data-quotation-book"] ?? ""}
                                    node={attrs["data-quotation-node"] ?? ""}
                                    start={
                                        Number(attrs["data-quotation-start"]) ||
                                        0
                                    }
                                    end={
                                        attrs["data-quotation-end"]
                                            ? Number(
                                                  attrs["data-quotation-end"],
                                              )
                                            : undefined
                                    }
                                    kind={
                                        attrs["data-quotation-kind"] ?? "body"
                                    }
                                    mode={
                                        (attrs["data-quotation-mode"] as
                                            | "source"
                                            | "translation"
                                            | "source+translation") ??
                                        "translation"
                                    }
                                    layout={
                                        (attrs["data-quotation-layout"] as
                                            | "stacked"
                                            | "side-by-side-source-left"
                                            | "side-by-side-source-right") ??
                                        "stacked"
                                    }
                                />
                            );
                        }
                    },
                })}
            </div>
        </div>
    );
}
