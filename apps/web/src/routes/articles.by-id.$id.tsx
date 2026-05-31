import { createFileRoute, Navigate } from "@tanstack/react-router";
import { ArticlePageUI } from "#/modules/article";
import {
    getGetArticleByIdSuspenseQueryOptions,
    useGetArticleByIdSuspense,
} from "../api/articles/articles";

export const Route = createFileRoute("/articles/by-id/$id")({
    loader: ({ context, params }) => {
        context.queryClient.prefetchQuery(
            getGetArticleByIdSuspenseQueryOptions(params.id),
        );
    },
    component: ArticleByIdRedirect,
    pendingComponent: () => <ArticlePageUI kind="loading" />,
    errorComponent: () => <ArticlePageUI kind="error" />,
});

function ArticleByIdRedirect() {
    const { id } = Route.useParams();
    const { data } = useGetArticleByIdSuspense(id);
    const article = data?.data;

    return (
        <Navigate
            to="/articles/$slug"
            params={{ slug: article.slug }}
            replace
        />
    );
}
