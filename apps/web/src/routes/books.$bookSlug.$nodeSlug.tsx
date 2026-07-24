import { createFileRoute } from "@tanstack/react-router";
import { getGetBookSuspenseQueryOptions } from "#/api/books/books";
import { getGetNodeMetaSuspenseQueryOptions } from "../api/nodes/nodes";
import { getGetTocSuspenseQueryOptions } from "../api/toc/toc";
import {
    decode,
    getNodePageSuspenseQueryOptions,
    ReaderLayout,
    validateSearch,
} from "../modules/reader";
import {
    breadcrumbJsonLd,
    chapterJsonLd,
    findTocTrail,
    metaDescription,
    SEO_COPY,
    seoHead,
    toBookMeta,
} from "../modules/seo";

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch,
    loader: async ({ context, params }) => {
        // Heavy chapter content: awaited so the reading column renders
        // synchronously into the SSR HTML. Fire-and-forget streaming made
        // the column a late Suspense boundary whose $RC reveal script races
        // client hydration — with a warm bundle cache the client hydrates
        // first and the late reveal throws React #418. Costs first-byte
        // (~the chapter query), buys structural hydration safety; revisit
        // if TanStack Start fixes late-boundary hydration coordination.
        await context.queryClient.prefetchInfiniteQuery(
            getNodePageSuspenseQueryOptions({
                bookSlug: params.bookSlug,
                showOriginal: false,
                targetNodeSlug: params.nodeSlug,
            }),
        );
        // Book + TOC are light, proxy-cached, and needed by the page
        // anyway; awaited so head() has them for title/meta. Only the
        // scalars head() needs are returned — the components keep
        // reading the full payloads through their suspense hooks.
        const [bookRes, tocRes, metaRes] = await Promise.all([
            context.queryClient.ensureQueryData(
                getGetBookSuspenseQueryOptions(params.bookSlug),
            ),
            context.queryClient.ensureQueryData(
                getGetTocSuspenseQueryOptions(params.bookSlug),
            ),
            context.queryClient.ensureQueryData(
                getGetNodeMetaSuspenseQueryOptions(
                    params.bookSlug,
                    params.nodeSlug,
                ),
            ),
        ]);
        const trail = findTocTrail(tocRes.data, params.nodeSlug);
        return {
            bookMeta: toBookMeta(bookRes.data),
            nodeLabel: trail?.at(-1)?.label ?? params.nodeSlug,
            // "Chapter 1" only means something next to "Genesis".
            parentLabel: trail && trail.length > 1 ? trail.at(-2)?.label : null,
            excerpt: metaRes.data.excerpt ?? null,
        };
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return {};
        const { bookMeta, nodeLabel, parentLabel, excerpt } = loaderData;
        const nodeRef = parentLabel
            ? `${parentLabel}, ${nodeLabel}`
            : nodeLabel;
        const bookPath = `/books/${params.bookSlug}`;
        const path = `${bookPath}/${params.nodeSlug}`;
        return seoHead({
            title: SEO_COPY.reader.title(nodeRef, bookMeta.title),
            description: metaDescription(
                excerpt
                    ? SEO_COPY.reader.descriptionFromExcerpt(nodeRef, excerpt)
                    : SEO_COPY.reader.description(
                          nodeRef,
                          bookMeta.title,
                          bookMeta.author,
                      ),
            ),
            path,
            ogType: "book",
            jsonLd: [
                chapterJsonLd(nodeRef, bookMeta, bookPath, path),
                breadcrumbJsonLd([
                    { name: "Library", path: "/" },
                    { name: bookMeta.title, path: bookPath },
                    { name: nodeRef, path },
                ]),
            ],
        });
    },
    component: ReaderPage,
});

function ReaderPage() {
    const { bookSlug, nodeSlug } = Route.useParams();
    const search = Route.useSearch();
    const { panels } = decode({ bookSlug, nodeSlug, search });
    return <ReaderLayout panels={panels} />;
}
