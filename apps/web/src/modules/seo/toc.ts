import type { TocNodeResponse } from "../../api/model";

/**
 * Depth-first lookup of a TOC node by slug, returning the ancestor
 * trail root → target. Used to give SEO titles their context (a bare
 * "Chapter 1" only means something next to its parent "Genesis") and
 * to build BreadcrumbList JSON-LD.
 */
export function findTocTrail(
    nodes: TocNodeResponse[],
    slug: string,
): TocNodeResponse[] | undefined {
    for (const node of nodes) {
        if (node.slug === slug) return [node];
        const rest = findTocTrail(node.children, slug);
        if (rest) return [node, ...rest];
    }
    return undefined;
}
