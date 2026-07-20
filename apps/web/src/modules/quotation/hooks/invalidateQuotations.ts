import type { QueryClient } from "@tanstack/react-query";

/**
 * Invalidate every cached `list_quotations_for_node` response across
 * every book the user is reading. Needed because saved quotations
 * project across translations as visual markers —
 * a save in WEB has to refresh the KJV cache too, otherwise the user
 * has to hard-refresh to see the marker on a verse they just saved.
 *
 * Targets any query whose key path starts with `/api/books/` and ends
 * with `/quotations`. The shape of the key is
 * `[`/api/books/${slug}/quotations`, params]` (orval-generated), so we
 * match on a string prefix + suffix predicate.
 */
export function invalidateAllNodeQuotations(queryClient: QueryClient) {
    queryClient.invalidateQueries({
        predicate: (query) => {
            const key = query.queryKey[0];
            return (
                typeof key === "string" &&
                key.startsWith("/api/books/") &&
                key.endsWith("/quotations")
            );
        },
    });
}
