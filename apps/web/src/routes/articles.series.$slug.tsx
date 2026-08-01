import { createFileRoute } from "@tanstack/react-router";
import {
    getGetSeriesSuspenseQueryOptions,
    useGetSeriesSuspense,
} from "../api/series/series";
import { ArticleCard } from "../modules/article";
import {
    breadcrumbJsonLd,
    metaDescription,
    SEO_COPY,
    seoHead,
} from "../modules/seo";

const PER_PAGE = 20;

type SeriesSearch = {
    page?: number;
};

export const Route = createFileRoute("/articles/series/$slug")({
    component: SeriesDetailPage,
    validateSearch: (search: Record<string, unknown>): SeriesSearch => {
        const parsedPage = Number(search.page);
        // Absent = page 1, keeping the default out of the URL (same
        // reasoning as /articles: a clean canonical URL serves 200).
        return {
            page:
                !Number.isNaN(parsedPage) && parsedPage > 1
                    ? parsedPage
                    : undefined,
        };
    },
    loaderDeps: ({ search: { page } }) => ({ page }),
    loader: async ({ context, params, deps }) => {
        const res = await context.queryClient.ensureQueryData(
            getGetSeriesSuspenseQueryOptions(params.slug, {
                page: deps.page,
                per_page: PER_PAGE,
            }),
        );
        const series = res.data;
        return {
            name: series.name,
            description: series.description
                ? metaDescription(series.description)
                : SEO_COPY.series.description(series.name),
        };
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return {};
        const path = `/articles/series/${params.slug}`;
        return seoHead({
            title: SEO_COPY.series.title(loaderData.name),
            description: loaderData.description,
            path,
            jsonLd: [
                breadcrumbJsonLd([
                    { name: "Articles", path: "/articles" },
                    { name: loaderData.name, path },
                ]),
            ],
        });
    },
});

function SeriesDetailPage() {
    const { slug } = Route.useParams();
    const { page } = Route.useSearch();
    const navigate = Route.useNavigate();

    const { data } = useGetSeriesSuspense(slug, {
        page,
        per_page: PER_PAGE,
    });
    const series = data.data;
    const totalPages = Math.ceil(series.total / PER_PAGE);

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-2">
                {series.name}
            </h1>
            {series.description && (
                <p className="text-base text-stone-500 mb-2">
                    {series.description}
                </p>
            )}
            <p className="text-xs text-stone-400 tabular-nums mb-8">
                {series.total} {series.total === 1 ? "article" : "articles"}
            </p>

            {series.articles.length === 0 && (
                <p className="text-sm text-stone-400">
                    No articles in this series yet.
                </p>
            )}

            <div className="space-y-4">
                {series.articles.map((article) => (
                    <ArticleCard key={article.id} article={article} />
                ))}
            </div>

            {totalPages > 1 && (
                <div className="flex justify-center gap-2 mt-8">
                    {Array.from({ length: totalPages }, (_, i) => i + 1).map(
                        (p) => (
                            <button
                                key={p}
                                type="button"
                                onClick={() =>
                                    navigate({
                                        search: () => ({
                                            page: p > 1 ? p : undefined,
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
        </div>
    );
}
