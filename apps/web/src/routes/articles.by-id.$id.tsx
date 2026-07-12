import { createFileRoute, notFound, redirect } from "@tanstack/react-router";
import { ArticlePageUI } from "#/modules/article";
import { getArticleById } from "../api/articles/articles";
import { FetchError } from "../api/fetcher";

/**
 * Durable redirect from a UUID-keyed article URL to the slug URL.
 * Resolved in the loader so SSR answers with a real 301 (crawlers and
 * link equity follow it) instead of a client-side <Navigate> shell.
 */
export const Route = createFileRoute("/articles/by-id/$id")({
    loader: async ({ params }) => {
        let slug: string | undefined;
        try {
            const res = await getArticleById(params.id);
            if (res.status === 200) slug = res.data?.slug;
        } catch (err) {
            if (!(err instanceof FetchError && err.status === 404)) {
                throw err;
            }
        }
        if (slug) {
            throw redirect({
                to: "/articles/$slug",
                params: { slug },
                statusCode: 301,
                replace: true,
            });
        }
        throw notFound();
    },
    pendingComponent: () => <ArticlePageUI kind="loading" />,
    errorComponent: () => <ArticlePageUI kind="error" />,
    notFoundComponent: () => <ArticlePageUI kind="error" />,
});
