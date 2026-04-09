import { createFileRoute, Navigate } from "@tanstack/react-router";
import { useGetArticleById } from "../api/articles/articles";

export const Route = createFileRoute("/articles/by-id/$id")({
    component: ArticleByIdRedirect,
});

function ArticleByIdRedirect() {
    const { id } = Route.useParams();
    const { data, isLoading } = useGetArticleById(id);
    const article = data?.data;

    if (isLoading) {
        return (
            <div className="min-h-screen bg-white">
                <div className="max-w-3xl mx-auto px-8 py-16">
                    <p className="text-sm text-stone-400">Loading...</p>
                </div>
            </div>
        );
    }

    if (article) {
        return <Navigate to="/articles/$slug" params={{ slug: article.slug }} replace />;
    }

    return (
        <div className="min-h-screen bg-white">
            <div className="max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Article not found.</p>
            </div>
        </div>
    );
}
