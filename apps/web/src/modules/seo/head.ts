/**
 * Per-route SEO head builder. Every public route's `head()` composes its
 * result through `seoHead` so titles, descriptions, canonicals, Open
 * Graph tags and JSON-LD stay consistent site-wide.
 *
 * Canonical URLs use pathname only — reader panel state and list
 * filters live in search params and must not fragment the canonical.
 */

import { getSiteOrigin } from "../../config";
import { SITE_NAME } from "./copy";

export type OgType = "website" | "book" | "article" | "profile";

export interface SeoHeadArgs {
    /** Full title; callers compose e.g. "Node — Book | Scholia". */
    title: string;
    description?: string;
    /** Pathname only, no search params or hash. */
    path: string;
    ogType?: OgType;
    noindex?: boolean;
    jsonLd?: Array<object>;
}

const OG_IMAGE: string | null = "/images/og-default.png";

export function seoHead({
    title,
    description,
    path,
    ogType = "website",
    noindex,
    jsonLd,
}: SeoHeadArgs) {
    const origin = getSiteOrigin();
    const url = `${origin}${path}`;
    return {
        meta: [
            { title },
            ...(description
                ? [{ name: "description", content: description }]
                : []),
            ...(noindex ? [{ name: "robots", content: "noindex" }] : []),
            { property: "og:title", content: title },
            ...(description
                ? [{ property: "og:description", content: description }]
                : []),
            { property: "og:type", content: ogType },
            { property: "og:url", content: url },
            { property: "og:site_name", content: SITE_NAME },
            ...(OG_IMAGE
                ? [{ property: "og:image", content: `${origin}${OG_IMAGE}` }]
                : []),
            {
                name: "twitter:card",
                content: OG_IMAGE ? "summary_large_image" : "summary",
            },
            { name: "twitter:title", content: title },
        ],
        links: [{ rel: "canonical", href: url }],
        scripts: (jsonLd ?? []).map((obj) => ({
            type: "application/ld+json",
            // < guard: user-authored strings (bios, article titles)
            // must not be able to close the script tag.
            children: JSON.stringify(obj).replace(/</g, "\\u003c"),
        })),
    };
}

/** Reduce an HTML fragment to its text content (for meta descriptions). */
export function stripHtml(html: string): string {
    return html
        .replace(/<[^>]*>/g, " ")
        .replace(/&nbsp;/g, " ")
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&#39;|&apos;/g, "'")
        .replace(/&quot;/g, '"');
}

/** Collapse whitespace and clamp to a search-snippet-sized length. */
export function metaDescription(text: string, max = 160): string {
    const clean = text.replace(/\s+/g, " ").trim();
    if (clean.length <= max) return clean;
    const cut = clean.slice(0, max - 1);
    const lastSpace = cut.lastIndexOf(" ");
    return `${cut.slice(0, lastSpace > max / 2 ? lastSpace : max - 1)}…`;
}
