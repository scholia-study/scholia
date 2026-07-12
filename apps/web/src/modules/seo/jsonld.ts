/**
 * schema.org JSON-LD builders. Each takes the scalar meta shapes the
 * route loaders already return (never full API payloads) and produces
 * a plain object; `seoHead` serializes and escapes it into a
 * <script type="application/ld+json"> tag.
 */

import { getSiteOrigin } from "../../config";
import type { BookMeta } from "./bookMeta";

const CONTEXT = "https://schema.org";

/** Drop null/undefined/empty-string entries so we never emit them. */
function compact(obj: Record<string, unknown>): Record<string, unknown> {
    return Object.fromEntries(
        Object.entries(obj).filter(
            ([, v]) => v !== null && v !== undefined && v !== "",
        ),
    );
}

export function bookJsonLd(bookMeta: BookMeta, path: string): object {
    const origin = getSiteOrigin();
    return compact({
        "@context": CONTEXT,
        "@type": "Book",
        name: bookMeta.title,
        author: bookMeta.author
            ? { "@type": "Person", name: bookMeta.author }
            : null,
        inLanguage: bookMeta.language,
        datePublished: bookMeta.publicationYear?.toString() ?? null,
        publisher: bookMeta.publisher,
        // Cross-link translated editions so engines see them as
        // translations of one work, not near-duplicate competitors.
        workTranslation: bookMeta.translations.length
            ? bookMeta.translations.map((t) => ({
                  "@type": "Book",
                  name: t.title,
                  inLanguage: t.language,
                  url: `${origin}/books/${t.slug}`,
              }))
            : null,
        // Reference-by-URL: the source page's own JSON-LD defines it.
        translationOfWork: bookMeta.sourceBookSlug
            ? {
                  "@type": "Book",
                  "@id": `${origin}/books/${bookMeta.sourceBookSlug}`,
              }
            : null,
        url: `${origin}${path}`,
    });
}

export function chapterJsonLd(
    nodeRef: string,
    bookMeta: BookMeta,
    bookPath: string,
    path: string,
): object {
    const { "@context": _, ...book } = bookJsonLd(bookMeta, bookPath) as {
        "@context": string;
        [key: string]: unknown;
    };
    return compact({
        "@context": CONTEXT,
        "@type": "Chapter",
        name: nodeRef,
        isPartOf: book,
        inLanguage: bookMeta.language,
        url: `${getSiteOrigin()}${path}`,
    });
}

export function breadcrumbJsonLd(
    items: Array<{ name: string; path: string }>,
): object {
    const origin = getSiteOrigin();
    return {
        "@context": CONTEXT,
        "@type": "BreadcrumbList",
        itemListElement: items.map((item, i) => ({
            "@type": "ListItem",
            position: i + 1,
            name: item.name,
            item: `${origin}${item.path}`,
        })),
    };
}

export interface ArticleJsonLdMeta {
    title: string;
    description: string | null;
    authorName: string;
    authorPath: string | null;
    publishedAt: string | null;
    updatedAt: string | null;
}

export function articleJsonLd(meta: ArticleJsonLdMeta, path: string): object {
    const origin = getSiteOrigin();
    return compact({
        "@context": CONTEXT,
        "@type": "Article",
        headline: meta.title,
        description: meta.description,
        author: compact({
            "@type": "Person",
            name: meta.authorName,
            url: meta.authorPath ? `${origin}${meta.authorPath}` : null,
        }),
        datePublished: meta.publishedAt,
        dateModified: meta.updatedAt,
        url: `${origin}${path}`,
    });
}

export interface ProfileJsonLdMeta {
    displayName: string;
    handle: string;
    bio: string | null;
}

export function profileJsonLd(meta: ProfileJsonLdMeta, path: string): object {
    const url = `${getSiteOrigin()}${path}`;
    return {
        "@context": CONTEXT,
        "@type": "ProfilePage",
        mainEntity: compact({
            "@type": "Person",
            name: meta.displayName,
            alternateName: `@${meta.handle}`,
            description: meta.bio,
            url,
        }),
        url,
    };
}
