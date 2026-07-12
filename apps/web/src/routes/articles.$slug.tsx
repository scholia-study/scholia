import { Chip } from "@mui/material";
import { createFileRoute, Link } from "@tanstack/react-router";
import type { Element } from "html-react-parser";
import {
    getGetPublishedArticleSuspenseQueryOptions,
    useGetPublishedArticleSuspense,
} from "../api/articles/articles";
import { useAuth } from "../hooks/useAuth";
import {
    ArticlePageUI,
    EditorialLabelChips,
    EditorialLabelManager,
} from "../modules/article";
import {
    ArticleQuotationCard,
    ArticleSentences,
    QuotationCard,
} from "../modules/quotation";
import {
    articleJsonLd,
    breadcrumbJsonLd,
    metaDescription,
    SEO_COPY,
    seoHead,
    stripHtml,
} from "../modules/seo";

export const Route = createFileRoute("/articles/$slug")({
    loader: async ({ context, params }) => {
        const res = await context.queryClient.ensureQueryData(
            getGetPublishedArticleSuspenseQueryOptions(params.slug),
        );
        const article = res.data;
        return {
            title: article.title,
            description: article.description
                ? metaDescription(article.description)
                : metaDescription(stripHtml(article.html)),
            archived: article.status === "archived",
            authorName: article.author_display_name,
            authorHandle: article.author_handle ?? null,
            publishedAt: article.published_at ?? null,
            updatedAt: article.updated_at,
        };
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return {};
        const path = `/articles/${params.slug}`;
        return seoHead({
            title: SEO_COPY.article.title(loaderData.title),
            description: loaderData.description,
            path,
            ogType: "article",
            noindex: loaderData.archived,
            jsonLd: [
                articleJsonLd(
                    {
                        title: loaderData.title,
                        description: loaderData.description,
                        authorName: loaderData.authorName,
                        authorPath: loaderData.authorHandle
                            ? `/users/${loaderData.authorHandle}`
                            : null,
                        publishedAt: loaderData.publishedAt,
                        updatedAt: loaderData.updatedAt,
                    },
                    path,
                ),
                breadcrumbJsonLd([
                    { name: "Articles", path: "/articles" },
                    { name: loaderData.title, path },
                ]),
            ],
        });
    },
    component: PublishedArticlePage,
    pendingComponent: () => <ArticlePageUI kind="loading" />,
    errorComponent: () => <ArticlePageUI kind="error" />,
});

function replaceEmbed(domNode: Element) {
    // Article quotation embed (check first — "article-quotation-embed"
    // also matches "quotation-embed" via substring)
    if (domNode.attribs?.class?.includes("article-quotation-embed")) {
        const id = domNode.attribs["data-article-quotation-id"];
        if (id) {
            return <ArticleQuotationCard id={id} />;
        }
        return undefined;
    }

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

    return undefined;
}

function PublishedArticlePage() {
    const { slug } = Route.useParams();
    const { data: articleData } = useGetPublishedArticleSuspense(slug);
    const article = articleData.data;
    const { hasPermission } = useAuth();
    const canManageLabels = hasPermission("article_labels_manage");

    return (
        <div className="flex-1 bg-white">
            <div className="max-w-3xl mx-auto px-8 py-16">
                {/* Header */}
                <header className="mb-8">
                    <h1 className="text-3xl font-bold text-stone-900 mb-3">
                        {article.title}
                    </h1>
                    {(article.labels.length > 0 || canManageLabels) &&
                        article.status === "published" && (
                            <div className="flex flex-wrap items-center gap-1.5 mb-3">
                                <EditorialLabelChips
                                    labels={article.labels}
                                    clickable={true}
                                />
                                {canManageLabels && (
                                    <EditorialLabelManager
                                        articleSlug={article.slug}
                                        appliedLabels={article.labels}
                                    />
                                )}
                            </div>
                        )}
                    {article.description && (
                        <p className="text-lg text-stone-500 mb-4">
                            {article.description}
                        </p>
                    )}
                    <div className="flex items-center gap-2 text-sm text-stone-400 flex-wrap">
                        {article.author_handle ? (
                            <Link
                                to="/users/$handle"
                                params={{ handle: article.author_handle }}
                                className="text-stone-600 hover:underline no-underline"
                            >
                                {article.author_display_name}
                            </Link>
                        ) : (
                            <Link
                                to="/users/by-id/$id"
                                params={{ id: article.author_user_id }}
                                className="text-stone-600 hover:underline no-underline"
                            >
                                {article.author_display_name}
                            </Link>
                        )}
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
                        disabled={article.status === "archived"}
                    />
                </div>
            </div>
        </div>
    );
}
